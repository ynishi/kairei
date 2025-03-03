mod micro_agent_tests;
mod provider_tests;
mod type_checker_tests;

use std::collections::HashMap;

use kairei_core::{
    config::PluginConfig,
    expression,
    provider::{
        plugin::PluginContext,
        request::{ProviderContext, ProviderRequest, RequestInput},
    },
};
use lazy_static::lazy_static;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[ctor::ctor]
fn init_tests() {
    // テストの前に一度だけ実行したい処理
    // tracing_subscriberの初期化
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");
}

const RUN_API_TESTS: &str = "RUN_API_TESTS";

lazy_static! {
    pub static ref EXTERNAL_API_TESTS_ENABLED: bool = {
        match std::env::var(RUN_API_TESTS) {
            Ok(_) => true,
            Err(_) => {
                println!("Skipping API tests: RUN_API_TESTS not set");
                false
            }
        }
    };
}

pub fn should_run_external_api_tests() -> bool {
    *EXTERNAL_API_TESTS_ENABLED
}

// TODO: Unit test とまとめる
// テストコンテキストのホルダー構造体
#[derive(Clone)]
pub struct TestContextHolder {
    request: ProviderRequest,
    context: ProviderContext,
    configs: HashMap<String, PluginConfig>,
}

impl TestContextHolder {
    pub fn new(request_content: &str) -> Self {
        let input = RequestInput {
            query: expression::Value::String(request_content.to_string()),
            ..Default::default()
        };
        let request = ProviderRequest {
            input,
            ..Default::default()
        };

        Self {
            request,
            context: ProviderContext::default(),
            configs: HashMap::new(),
        }
    }

    pub fn get_plugin_context(&self) -> PluginContext<'_> {
        PluginContext {
            request: &self.request,
            context: &self.context,
            configs: &self.configs,
        }
    }
}
