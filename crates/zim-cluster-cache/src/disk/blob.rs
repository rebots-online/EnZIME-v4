use super::DiskPackStore;
use anzimmerman::{
    cluster_stream::{decode_cluster_to, read_cluster_blob, ClusterDecodeLimits},
    model::ZimError,
    range::{ClusterBlobSource, ClusterByteRange},
};
use async_trait::async_trait;
use bytes::Bytes;
#[async_trait]
impl ClusterBlobSource for DiskPackStore {
    async fn read_blob(
        &self,
        index: u32,
        range: ClusterByteRange,
        blob: u32,
        maximum_bytes: u64,
    ) -> Result<Bytes, ZimError> {
        let gate = self.cluster_gate.clone().lock_owned().await;
        let path = self
            .ensure_inner(index, range, false)
            .await
            .map_err(ZimError::Io)?;
        let workspace = self.root.join("workspace").join(format!("{index}.decoded"));
        let budget = self
            .manifest()
            .map_err(ZimError::Io)?
            .policy
            .workspace_budget_bytes;
        tokio::task::spawn_blocking(move || {
            let _gate = gate;
            let run = || {
                let mut input =
                    std::fs::File::open(path).map_err(|e| ZimError::Io(e.to_string()))?;
                let mut output = std::fs::OpenOptions::new()
                    .create(true)
                    .truncate(true)
                    .read(true)
                    .write(true)
                    .open(&workspace)
                    .map_err(|e| ZimError::Io(e.to_string()))?;
                let limits = ClusterDecodeLimits {
                    maximum_output_bytes: budget.min(2147483648),
                    maximum_blob_bytes: maximum_bytes.min(33554432),
                    maximum_decoder_window_bytes: 67108864,
                };
                let decoded = decode_cluster_to(&mut input, &mut output, limits)?;
                read_cluster_blob(&mut output, decoded, blob, limits.maximum_blob_bytes)
            };
            let result = run();
            let _ = std::fs::remove_file(workspace);
            result
        })
        .await
        .map_err(|e| ZimError::Io(e.to_string()))?
    }
}
