use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use thiserror::Error;
use uuid::Uuid;

use crate::sistence::memory::context::{AgentProxy, WorkspaceContext};
use crate::sistence::memory::error::SistenceMemoryError;
use crate::sistence::memory::service::recollection::RecollectionService;
use crate::sistence::types::WorkspaceId;

/// Error type for sistence action operations
#[derive(Error, Debug)]
pub enum SistenceActionError {
    /// Memory-related error
    #[error("Memory error: {0}")]
    MemoryError(#[from] SistenceMemoryError),

    /// Execution timeout
    #[error("Execution timed out after {0:?}")]
    Timeout(Duration),

    /// Execution was cancelled
    #[error("Execution was cancelled")]
    Cancelled,

    /// Invalid execution ID
    #[error("Invalid execution ID: {0}")]
    InvalidExecutionId(String),

    /// No workspaces available
    #[error("No workspaces available")]
    NoWorkspacesAvailable,

    /// No agents available
    #[error("No agents available")]
    NoAgentsAvailable,

    /// Other errors
    #[error("Action error: {0}")]
    Other(String),
}

/// Result type for sistence action operations
pub type SistenceActionResult<T> = Result<T, SistenceActionError>;

/// Unique identifier for an execution
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExecutionId(String);

impl Default for ExecutionId {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionId {
    /// Creates a new random execution ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl fmt::Display for ExecutionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Status of an execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionStatus {
    /// Execution is queued
    Queued,
    /// Execution is running
    Running,
    /// Execution completed successfully
    Completed,
    /// Execution failed
    Failed(String),
    /// Execution was cancelled
    Cancelled,
    /// Execution timed out
    TimedOut,
}

/// Criteria for execution
#[derive(Debug, Clone)]
pub struct ExecutionCriteria {
    /// Maximum time to run
    pub max_time: Option<Duration>,
    /// Maximum cost to incur
    pub max_cost: Option<f64>,
    /// Whether to continue on error
    pub continue_on_error: bool,
}

impl Default for ExecutionCriteria {
    fn default() -> Self {
        Self {
            max_time: Some(Duration::from_secs(60)),
            max_cost: None,
            continue_on_error: false,
        }
    }
}

/// Options for parallel execution
#[derive(Debug, Clone)]
pub struct ParallelExecutionOptions {
    /// Timeout for execution
    pub execution_timeout: Option<Duration>,
    /// Criteria for execution
    pub execution_criteria: ExecutionCriteria,
}

impl Default for ParallelExecutionOptions {
    fn default() -> Self {
        Self {
            execution_timeout: Some(Duration::from_secs(300)),
            execution_criteria: ExecutionCriteria::default(),
        }
    }
}

/// Criteria for exploration
#[derive(Debug, Clone)]
pub struct ExplorationCriteria {
    /// Maximum number of iterations
    pub max_iterations: usize,
    /// Maximum time to run
    pub max_time: Duration,
    /// Maximum cost to incur
    pub max_cost: f64,
}

impl Default for ExplorationCriteria {
    fn default() -> Self {
        Self {
            max_iterations: 5,
            max_time: Duration::from_secs(300),
            max_cost: 1.0,
        }
    }
}

/// Options for collaboration
#[derive(Debug, Clone)]
pub struct CollaborationOptions {
    /// Maximum time for collaboration
    pub max_time: Option<Duration>,
    /// Whether to use consensus
    pub use_consensus: bool,
    /// Minimum number of agents required
    pub min_agents: Option<usize>,
}

impl Default for CollaborationOptions {
    fn default() -> Self {
        Self {
            max_time: Some(Duration::from_secs(300)),
            use_consensus: true,
            min_agents: Some(2),
        }
    }
}

/// Result of exploration
#[derive(Debug, Clone)]
pub struct BestResult<T> {
    /// The best result
    pub result: T,
    /// Score of the result
    pub score: f64,
    /// Iteration that produced the result
    pub iteration: usize,
    /// Time taken to produce the result
    pub time_taken: Duration,
    /// Cost incurred to produce the result
    pub cost: f64,
}

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
    /// Executes a task across multiple workspaces
    ///
    /// # Arguments
    /// * `task` - The task to execute
    /// * `workspace_count` - The number of workspaces to use
    /// * `criteria` - Criteria for execution
    ///
    /// # Returns
    /// * `SistenceActionResult<Vec<T>>` - The results from each workspace
    fn execute_across_workspaces<T>(
        &self,
        task: fn() -> T,
        workspace_count: usize,
        criteria: ExecutionCriteria,
    ) -> SistenceActionResult<Vec<T>>;

    /// Executes a task with specific workspaces
    ///
    /// # Arguments
    /// * `task` - The task to execute
    /// * `workspace_ids` - The IDs of the workspaces to use
    ///
    /// # Returns
    /// * `SistenceActionResult<Vec<T>>` - The results from each workspace
    fn execute_with_specific_workspaces<T>(
        &self,
        task: fn() -> T,
        workspace_ids: Vec<WorkspaceId>,
    ) -> SistenceActionResult<Vec<T>>;

    /// Cancels an execution
    ///
    /// # Arguments
    /// * `execution_id` - The ID of the execution to cancel
    ///
    /// # Returns
    /// * `SistenceActionResult<()>` - Success or failure
    fn cancel_execution(&self, execution_id: ExecutionId) -> SistenceActionResult<()>;

    /// Gets the status of an execution
    ///
    /// # Arguments
    /// * `execution_id` - The ID of the execution to check
    ///
    /// # Returns
    /// * `SistenceActionResult<ExecutionStatus>` - The status of the execution
    fn get_execution_status(
        &self,
        execution_id: ExecutionId,
    ) -> SistenceActionResult<ExecutionStatus>;
}

pub struct DefaultSistenceActionService {
    /// Service for parallel execution
    parallel_service: Arc<DefaultParallelExecutionService>,
    /// Service for recollections
    recollection_service: Arc<dyn RecollectionService + Send + Sync>,
    /// Workspace factory
    workspace_factory: Arc<dyn Fn() -> WorkspaceContext + Send + Sync>,
}

pub struct DefaultParallelExecutionService {
    /// Workspace factory
    workspace_factory: Arc<dyn Fn() -> WorkspaceContext + Send + Sync>,
    /// Active executions
    executions: Arc<Mutex<HashMap<ExecutionId, ExecutionStatus>>>,
}
