pub mod common;
pub mod default;

pub use common::PluginVisitor;
pub use common::TypeVisitor;

// Re-export common visitor traits and implementations
pub mod prelude {
    pub use super::{PluginVisitor, TypeVisitor};
}
