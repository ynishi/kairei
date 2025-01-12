mod provider_tests;

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
