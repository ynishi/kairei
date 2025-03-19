use kairei_core::config::{self, SecretConfig, SystemConfig};

pub mod openai_test;
pub mod provider_test;
pub mod serper_test;
pub mod sistence_integration_test;
pub mod validation_e2e_test;
pub mod validation_integration_test;

const TEST_CONFIG_PATH: &str = "tests/provider_tests/test_config.json";
const TEST_SECRET_PATH: &str = "tests/provider_tests/test_secret.json";

fn setup_config() -> (SystemConfig, SecretConfig) {
    let config: SystemConfig = config::from_file(TEST_CONFIG_PATH).unwrap();
    let secret_config: SecretConfig = config::from_file(TEST_SECRET_PATH).unwrap();

    (config, secret_config)
}
