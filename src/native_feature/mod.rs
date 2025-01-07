pub mod native_registry;
/// 1. Native Layer
/// Provides the most fundamental system functionalities. Requires high-performance and reliable implementation.
/// Main Components:
/// System Events:
/// - Tick: System heartbeat (Native implementation)
/// System State:
/// - Memory usage
/// - Number of agents
/// - Event queue status
/// - Plugin state management
pub mod ticker;
pub mod types;
