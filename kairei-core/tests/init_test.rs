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
