mod catalog;
mod catalog_budget;
mod catalog_discovery;
mod catalog_schema;
pub use catalog::{DiskEntryCatalog, OfflineCatalogProgress};
pub use catalog_discovery::discover_offline_entries;
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
    let catalog = DiskEntryCatalog::new(store.clone(), index.header().clone())?;
    let mut seed = store.manifest()?.metadata;
    let progress = store.catalog_progress()?;
    seed.article_count = progress.readable_articles;
    seed.media_count = progress.readable_resources;
    let catalog = if store.manifest()?.policy.offline {
        catalog
    } else {
        catalog.with_remote(index)
    };
    Ok(Arc::new(
        RangeZimArchive::open_index(Arc::new(catalog), store, seed, RangeReadLimits::default())
            .await
            .map_err(|e| e.to_string())?,
    ))
}
