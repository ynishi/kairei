pub mod agents;
pub mod compiler;
pub mod events;
pub mod system;
pub mod test_helpers;

// Re-export all handlers for easier imports
pub use agents::*;
pub use compiler::*;
pub use events::*;
pub use system::*;
