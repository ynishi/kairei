use crate::sistence::{memory::model::recollection::MergeStrategy, types::WorkspaceId};

use super::{
    error::SistenceWorkspaceResult,
    model::{MergeResult, WorkspaceInfo},
};

pub trait WorkspaceService {
    fn create_workspace(
        &self,
        name: String,
        purpose: Option<String>,
    ) -> SistenceWorkspaceResult<WorkspaceId>;
    fn get_active_workspace(&self) -> SistenceWorkspaceResult<WorkspaceId>;
    fn list_workspaces(&self) -> SistenceWorkspaceResult<Vec<WorkspaceInfo>>;

    // フォーク&マージ操作
    fn fork_workspace(
        &self,
        base_id: WorkspaceId,
        name: String,
    ) -> SistenceWorkspaceResult<WorkspaceId>;
    fn merge_workspace(
        &self,
        source_id: WorkspaceId,
        target_id: WorkspaceId,
        strategy: MergeStrategy,
    ) -> SistenceWorkspaceResult<MergeResult>;

    // ワークスペース状態管理
    fn activate_workspace(&self, id: WorkspaceId) -> SistenceWorkspaceResult<()>;
    fn archive_workspace(&self, id: WorkspaceId) -> SistenceWorkspaceResult<()>;
}
