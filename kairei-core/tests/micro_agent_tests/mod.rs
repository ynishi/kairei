use std::{collections::HashSet, time::Duration};

use kairei_core::{
    config::{self, SecretConfig, SystemConfig},
    event_bus::{Event, Value},
    system::System,
};
use tracing::debug;
use uuid::Uuid;

pub mod on_fail_test;
pub mod travel_planning_test;

const TEST_SECRET_PATH: &str = "tests/micro_agent_tests/test_secret.json";

fn setup_secret() -> SecretConfig {
    let secret_config: SecretConfig = config::from_file(TEST_SECRET_PATH).unwrap();

    secret_config
}

async fn setup_system(system_config_str: &str, dsl_str: &str, required: &[&str]) -> System {
    let system_config: SystemConfig = config::from_str(system_config_str).unwrap();
    let secret = setup_secret();
    debug!("System Config: {:?}", system_config);

    let mut system = System::new(&system_config, &secret).await;

    let root = system.parse_dsl(dsl_str).await.unwrap();
    debug!("Root: {:?}", root);
    root.micro_agent_defs
        .is_empty()
        .then(|| panic!("No micro agents found"));
    let agent_name_set: HashSet<&str> =
        HashSet::from_iter(root.micro_agent_defs.iter().map(|x| x.name.as_str()));
    let required_set = HashSet::from_iter(required.iter().cloned());
    // check diff
    let diff: HashSet<_> = required_set.difference(&agent_name_set).collect();
    if !diff.is_empty() {
        panic!("Missing required micro agents, {:?}", diff)
    }
    system.initialize(root).await.unwrap();
    system
}

fn create_request(
    agnent_name: &str,
    request_id: &Uuid,
    request_type: &str,
    requests: Vec<(&str, Value)>,
    timeout: Option<u64>,
) -> Event {
    let mut builder = Event::request_builder()
        .request_type(request_type)
        .requester("test")
        .responder(agnent_name)
        .request_id(request_id.to_string().as_str());

    for request in requests.clone() {
        builder = builder.clone().parameter(request.0, &request.1);
    }
    if let Some(timeout) = timeout {
        builder = builder.parameter("timeout", &Value::Duration(Duration::from_secs(timeout)));
    } else {
        builder = builder.parameter("timeout", &Value::Duration(Duration::from_secs(240)));
    }

    builder.build().unwrap()
}
