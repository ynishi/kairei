use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::sistence::memory::context::{AgentProxy, WorkspaceContext};
use crate::sistence::memory::service::recollection::RecollectionRepository;
use crate::sistence::types::{ExecutionId, WorkspaceId};

use super::error::SistenceActionResult;
use super::model::{
    BestResult, CollaborationOptions, ExecutionStatus, ExplorationCriteria, OptimizationCriteria,
    ParallelExecutionOptions,
};

/// Service for sistence actions
pub trait SistenceActionService {
    /// Executes a task in parallel across multiple workspaces
    ///
    /// # Arguments
    /// * `task` - The task to execute
    /// * `workspace_count` - The number of workspaces to use (default: 3)
    /// * `options` - Options for parallel execution
    ///
    /// # Returns
    /// * `SistenceActionResult<Vec<T>>` - The results from each workspace
    fn think_in_parallel<T>(
        &self,
        task: fn() -> T,
        workspace_count: Option<usize>,
        options: Option<ParallelExecutionOptions>,
    ) -> SistenceActionResult<Vec<T>>;

    /// Explores alternative approaches to a task
    ///
    /// # Arguments
    /// * `task` - The task to execute
    /// * `count` - The number of alternatives to explore
    /// * `criteria` - Criteria for exploration
    ///
    /// # Returns
    /// * `SistenceActionResult<BestResult<T>>` - The best result
    fn explore_alternatives<T>(
        &self,
        task: fn() -> T,
        count: usize,
        criteria: ExplorationCriteria,
    ) -> SistenceActionResult<BestResult<T>>;

    /// Collaborates with agents to complete a task
    ///
    /// # Arguments
    /// * `agents` - The agents to collaborate with
    /// * `task` - The task to execute
    /// * `options` - Options for collaboration
    ///
    /// # Returns
    /// * `SistenceActionResult<T>` - The result of collaboration
    fn collaborate<T>(
        &self,
        agents: Vec<AgentProxy>,
        task: fn() -> T,
        options: CollaborationOptions,
    ) -> SistenceActionResult<T>;
}

/// Service for parallel execution
pub trait ParallelExecutionService {
    fn execute_across_workspaces<T, F>(
        &self,
        task: F,
        workspace_count: usize,
        optimization_criteria: OptimizationCriteria,
    ) -> SistenceActionResult<Vec<T>>
    where
        F: Fn(WorkspaceId) -> SistenceActionResult<T> + Send + Sync,
        T: Send + 'static;

    fn execute_with_specific_workspaces<T, F>(
        &self,
        task: F,
        workspace_ids: &[WorkspaceId],
    ) -> SistenceActionResult<Vec<T>>
    where
        F: Fn(WorkspaceId) -> SistenceActionResult<T> + Send + Sync,
        T: Send + 'static;
}

pub struct DefaultSistenceActionService {
    /// Service for parallel execution
    parallel_service: Arc<DefaultParallelExecutionService>,
    /// Service for recollections
    recollection_service: Arc<dyn RecollectionRepository + Send + Sync>,
    /// Workspace factory
    workspace_factory: Arc<dyn Fn() -> WorkspaceContext + Send + Sync>,
}

pub struct DefaultParallelExecutionService {
    /// Workspace factory
    workspace_factory: Arc<dyn Fn() -> WorkspaceContext + Send + Sync>,
    /// Active executions
    executions: Arc<Mutex<HashMap<ExecutionId, ExecutionStatus>>>,
}
