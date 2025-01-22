use kairei::config::{self, SecretConfig, SystemConfig};

pub mod travel_planning_test;

const TEST_SECRET_PATH: &str = "tests/micro_agent_tests/test_secret.json";

fn setup_secret() -> SecretConfig {
    let secret_config: SecretConfig = config::from_file(TEST_SECRET_PATH).unwrap();

    secret_config
}
