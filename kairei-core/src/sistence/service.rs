use std::sync::Arc;

use super::{
    action::service::DefaultSistenceActionService,
    error::SistenceResult,
    memory::{
        facade::SistenceMemoryFacade,
        model::recollection::{Content, Recollection, RecollectionQuery, RecollectionSource},
    },
    space::service::WorkspaceService,
    types::RecollectionId,
};

trait SistenceService {
    fn remember(
        &self,
        content: Content,
        source: RecollectionSource,
    ) -> SistenceResult<RecollectionId>;
    fn recall(&self, query: RecollectionQuery) -> SistenceResult<Vec<Recollection>>;
    fn think_in_parallel<T>(&self, task: fn() -> T, count: usize) -> SistenceResult<Vec<T>>;
}

struct SistenceServiceImpl {
    memory: Arc<SistenceMemoryFacade>,
    action: Arc<DefaultSistenceActionService>,
    workspace: Arc<dyn WorkspaceService>,
}
