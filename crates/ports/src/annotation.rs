use async_trait::async_trait;

use crate::zim::ArticleDocument;
use domain::annotation::{
    AnnotationAssetInput, AnnotationAssetRecord, AnnotationDraft, AnnotationError,
    AnnotationRecord, AnnotationResolution, SelectorBundle,
};
use domain::identity::{ArticleKey, PackId};

#[async_trait]
pub trait AnnotationResolver: Send + Sync {
    async fn resolve(
        &self,
        selector: &SelectorBundle,
        article: &ArticleDocument,
    ) -> Result<AnnotationResolution, AnnotationError>;
}

#[async_trait]
pub trait AnnotationService: Send + Sync {
    async fn create(
        &self,
        draft: AnnotationDraft,
    ) -> Result<AnnotationRecord, AnnotationError>;
    async fn get(
        &self,
        annotation_id: &str,
    ) -> Result<AnnotationRecord, AnnotationError>;
    async fn update(
        &self,
        annotation_id: &str,
        expected_revision: u32,
        draft: AnnotationDraft,
    ) -> Result<AnnotationRecord, AnnotationError>;
    async fn soft_delete(
        &self,
        annotation_id: &str,
        expected_revision: u32,
    ) -> Result<(), AnnotationError>;
    async fn restore(
        &self,
        annotation_id: &str,
    ) -> Result<AnnotationRecord, AnnotationError>;
    async fn list_for_article(
        &self,
        pack_id: &PackId,
        article_key: &ArticleKey,
    ) -> Result<Vec<AnnotationRecord>, AnnotationError>;
    async fn attach_asset(
        &self,
        annotation_id: &str,
        asset: AnnotationAssetInput,
    ) -> Result<AnnotationAssetRecord, AnnotationError>;
}
