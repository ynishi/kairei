use std::time::Duration;

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

pub enum OptimizationCriteria {
    Speed,             // 速度優先
    Thoroughness,      // 網羅性優先
    Diversity,         // 多様性優先
    ResourceEfficient, // リソース効率優先
    Balance,           // バランス重視
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
