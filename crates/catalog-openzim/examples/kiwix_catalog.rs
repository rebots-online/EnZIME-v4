//! Browse actual Kiwix metadata without downloading any ZIM payload.
use catalog_openzim::{KiwixCatalogClient, KiwixQuery};
use std::sync::Arc;
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = KiwixCatalogClient::new(Arc::new(transfer_http::HttpTransferSource::new()?));
    let page = client
        .search(&KiwixQuery {
            text: Some(
                std::env::args()
                    .nth(1)
                    .unwrap_or_else(|| "wikipedia".into()),
            ),
            language: Some("eng".into()),
            limit: 2,
            ..Default::default()
        })
        .await?;
    println!("{}", serde_json::to_string_pretty(&page)?);
    if let Some(entry) = page.entries.first() {
        println!(
            "{}",
            serde_json::to_string_pretty(&client.resolve(entry).await?)?
        );
    }
    Ok(())
}
