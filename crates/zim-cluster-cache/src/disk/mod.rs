mod blob;
mod manifest;
mod plan;
mod source;
mod store;
use anzimmerman::range::{RangeReadLimits, RangeZimArchive, RemoteZimIndex};
pub use manifest::{DiskPackManifest, DiskPackPolicy, SourceIdentity};
pub use plan::{install_selected, plan_disk_fit, PlanSummary};
pub use source::{DiskRangeSource, PinnedTransferSource};
use std::sync::Arc;
pub use store::DiskPackStore;
pub async fn open_disk_archive(
    store: Arc<DiskPackStore>,
) -> Result<Arc<dyn anzimmerman::model::ZimArchive>, String> {
    let index = Arc::new(
        RemoteZimIndex::open(
            Arc::new(DiskRangeSource::new(store.clone())),
            RangeReadLimits::default(),
        )
        .await
        .map_err(|e| e.to_string())?,
    );
    let seed = store.manifest()?.metadata;
    Ok(Arc::new(
        RangeZimArchive::open(index, store, seed, RangeReadLimits::default())
            .await
            .map_err(|e| e.to_string())?,
    ))
}
