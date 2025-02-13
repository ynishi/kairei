pub mod agent_registry;
pub mod analyzer;
pub mod ast;
pub mod ast_registry;
pub mod config;
pub mod core;
pub mod error;
pub mod eval;
pub mod event;
pub mod formatter;
pub mod gen;
pub mod native_feature;
pub mod preprocessor;
pub mod provider;
pub mod runtime;
pub mod system;
pub mod timestamp;
pub mod tokenizer;

// Re-exports
pub use ast::*;
pub use error::*;
pub use eval::*;
pub use event::*;

#[cfg(test)]
mod tests {
    use tracing_subscriber::{EnvFilter, FmtSubscriber};

    #[ctor::ctor]
    fn init_tests() {
        // テストの前に一度だけ実行したい処理
        // tracing_subscriberの初期化
        let subscriber = FmtSubscriber::builder()
            .with_env_filter(EnvFilter::from_default_env())
            .finish();
        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set tracing subscriber");
    }
}
