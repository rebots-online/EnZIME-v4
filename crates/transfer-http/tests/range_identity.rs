use futures_util::StreamExt;
use ports::transfer::*;
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{Arc, Mutex},
};
use transfer_http::HttpTransferSource;
fn server(responses: Vec<String>) -> (String, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/zim", listener.local_addr().unwrap());
    let requests = Arc::new(Mutex::new(vec![]));
    let captured = requests.clone();
    std::thread::spawn(move || {
        for response in responses {
            let (mut stream, _) = listener.accept().unwrap();
            let mut data = Vec::new();
            let mut byte = [0];
            while stream.read(&mut byte).unwrap_or(0) > 0 {
                data.push(byte[0]);
                if data.ends_with(b"\r\n\r\n") {
                    break;
                }
            }
            captured
                .lock()
                .unwrap()
                .push(String::from_utf8(data).unwrap());
            stream.write_all(response.as_bytes()).unwrap();
        }
    });
    (url, requests)
}
fn identity(uri: &str) -> TransferProbe {
    TransferProbe {
        uri: uri.into(),
        size_bytes: 10,
        accepts_ranges: true,
        etag: Some("\"v1\"".into()),
        last_modified: None,
        content_type: None,
    }
}
fn reply(status: u16, headers: &str, body: &str) -> String {
    format!("HTTP/1.1 {status} Test\r\nConnection: close\r\n{headers}\r\n{body}")
}
#[tokio::test]
async fn pinned_ranges_require_exact_identity_encoding_length_and_total() {
    let cases = [
        (
            206,
            "ETag: \"v1\"\r\nContent-Range: bytes 2-4/10\r\n",
            "abc",
            true,
        ),
        (206, "Content-Range: bytes 2-4/10\r\n", "abc", false),
        (
            206,
            "ETag: \"v2\"\r\nContent-Range: bytes 2-4/10\r\n",
            "abc",
            false,
        ),
        (
            206,
            "ETag: \"v1\"\r\nContent-Range: bytes 2-4/11\r\n",
            "abc",
            false,
        ),
        (
            206,
            "ETag: \"v1\"\r\nContent-Range: bytes 1-4/10\r\n",
            "abc",
            false,
        ),
        (
            206,
            "ETag: \"v1\"\r\nContent-Range: bytes 2-4/10\r\nContent-Encoding: gzip\r\n",
            "abc",
            false,
        ),
        (200, "ETag: \"v1\"\r\n", "abc", false),
        (
            206,
            "ETag: \"v1\"\r\nContent-Range: bytes 2-4/10\r\n",
            "ab",
            false,
        ),
        (
            206,
            "ETag: \"v1\"\r\nContent-Range: bytes 2-4/10\r\n",
            "abcd",
            false,
        ),
    ];
    for (status, headers, body, valid) in cases {
        let (url, captured) = server(vec![reply(status, headers, body)]);
        let result = HttpTransferSource::new()
            .unwrap()
            .open_pinned_range(
                &url,
                TransferRange {
                    start_byte: 2,
                    end_byte_exclusive: 5,
                },
                &identity(&url),
            )
            .await;
        let ok = match result {
            Ok(mut s) => {
                let mut good = true;
                let mut bytes = vec![];
                while let Some(chunk) = s.next().await {
                    match chunk {
                        Ok(b) => bytes.extend_from_slice(&b),
                        Err(_) => good = false,
                    }
                }
                good && bytes == b"abc"
            }
            Err(_) => false,
        };
        assert_eq!(ok, valid, "status={status} headers={headers} body={body}");
        let requests = captured.lock().unwrap();
        let request = requests[0].to_ascii_lowercase();
        assert!(request.contains("accept-encoding: identity"));
        assert!(request.contains("if-range: \"v1\""));
    }
}
#[tokio::test]
async fn last_modified_is_sent_and_validated() {
    let modified = "Mon, 12 Oct 2026 01:00:00 GMT";
    let (url, captured) = server(vec![reply(
        206,
        &format!("Last-Modified: {modified}\r\nContent-Range: bytes 2-4/10\r\n"),
        "abc",
    )]);
    let mut id = identity(&url);
    id.etag = None;
    id.last_modified = Some(modified.into());
    let mut stream = HttpTransferSource::new()
        .unwrap()
        .open_pinned_range(
            &url,
            TransferRange {
                start_byte: 2,
                end_byte_exclusive: 5,
            },
            &id,
        )
        .await
        .unwrap();
    while let Some(chunk) = stream.next().await {
        chunk.unwrap();
    }
    assert!(captured.lock().unwrap()[0].contains(&format!("if-unmodified-since: {modified}")));
}
#[tokio::test]
async fn probe_records_final_url_and_insecure_redirect_is_rejected() {
    let (url, _) = server(vec![
        reply(302, "Location: /final\r\nContent-Length: 0\r\n", ""),
        reply(
            206,
            "Content-Range: bytes 0-0/10\r\nContent-Length: 1\r\nETag: \"v1\"\r\n",
            "a",
        ),
    ]);
    let source = HttpTransferSource::new().unwrap();
    let probe = source
        .probe(TransferProbeRequest {
            uri: url,
            maximum_response_bytes: 1,
        })
        .await
        .unwrap();
    assert!(probe.uri.ends_with("/final"));
    let (url, _) = server(vec![reply(
        302,
        "Location: http://example.com/unsafe\r\nContent-Length: 0\r\n",
        "",
    )]);
    assert!(source
        .open_range(
            &url,
            TransferRange {
                start_byte: 0,
                end_byte_exclusive: 1
            },
            None
        )
        .await
        .is_err());
}

#[tokio::test]
async fn weak_identity_rejects_missing_or_changed_date_and_no_identity() {
    for headers in [
        "Content-Range: bytes 2-4/10\r\n",
        "Last-Modified: Tue, 13 Oct 2026 01:00:00 GMT\r\nContent-Range: bytes 2-4/10\r\n",
    ] {
        let (url, _) = server(vec![reply(206, headers, "abc")]);
        let mut id = identity(&url);
        id.etag = Some("W/\"weak\"".into());
        id.last_modified = Some("Mon, 12 Oct 2026 01:00:00 GMT".into());
        assert!(HttpTransferSource::new()
            .unwrap()
            .open_pinned_range(
                &url,
                TransferRange {
                    start_byte: 2,
                    end_byte_exclusive: 5
                },
                &id
            )
            .await
            .is_err());
    }
    let mut id = identity("https://fixture.invalid/a.zim");
    id.etag = None;
    assert!(HttpTransferSource::new()
        .unwrap()
        .open_pinned_range(
            &id.uri,
            TransferRange {
                start_byte: 0,
                end_byte_exclusive: 1
            },
            &id
        )
        .await
        .is_err());
}

#[tokio::test]
async fn probe_rejects_shifted_or_overlong_range_and_ignored_request() {
    for (status, headers) in [
        (206, "Content-Range: bytes 1-3/10\r\n"),
        (206, "Content-Range: bytes 0-4/10\r\n"),
        (200, "Accept-Ranges: bytes\r\nContent-Length: 0\r\n"),
    ] {
        let (uri, _) = server(vec![reply(status, headers, "")]);
        assert!(HttpTransferSource::new()
            .unwrap()
            .probe(TransferProbeRequest {
                uri,
                maximum_response_bytes: 3
            })
            .await
            .is_err());
    }
}
