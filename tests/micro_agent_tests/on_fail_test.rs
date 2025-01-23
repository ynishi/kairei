use std::time::Duration;

use kairei::{
    config::{self, SystemConfig},
    event_bus::{Event, Value},
    system::System,
};
use tokio::time::sleep;
use tracing::debug;
use uuid::Uuid;

use crate::{
    micro_agent_tests::{create_request, setup_secret},
    should_run_external_api_tests,
};

use super::setup_system;

const ON_FAIL_DSL: &str = r#"
world ProcessWorld {}

micro SearchAgent {
    answer {
        on request Search(query: String) -> Result<String, Error> {
                result = think("Search for ${query}") with {
                    max_tokens: 100
                } onFail(err) {
                    emit Error(err)
                }
                return Ok(result)
        }
    }
}

micro ProcessAgent {
    answer {
        on request Proccess(data: String) -> Result<String, Error> {
            {
                search_result = request Search to SearchAgent(query: data)
                result = think("Process ${data} with ${search_result}")
            } onFail(err) {
                emit Error(err)
            }
            return Ok("ok")
        }
    }
}
"#;

const SYSTEM_CONFIG: &str = r#"
{
  "provider_configs": {
    "primary_provider": "on_fail_provider",
    "providers": {
      "on_fail_provider": {
        "name": "on_fail_provider",
        "provider_type": "OpenAIChat",
        "provider_specific": {},
        "common_config": {
          "model": "gpt-4o-mini",
          "temperature": 0.7,
          "max_tokens": 500
        },
        "plugin_configs": {}
      }
    }
  }
}
"#;

async fn setup_on_fail() -> System {
    setup_system(SYSTEM_CONFIG, ON_FAIL_DSL, &["SearchAgent", "ProcessAgent"]).await
}

#[tokio::test]
async fn test_on_fail() {
    if !should_run_external_api_tests() {
        // return;
    }

    let system = setup_on_fail().await;
    system.start().await.unwrap();
    sleep(Duration::from_millis(100)).await;

    let request_data = vec![("data", Value::from("Tokyo, in short"))];
    let request_id = Uuid::new_v4();
    let request = create_request("ProcessAgent", &request_id, "Proccess", request_data, None);

    let result = system.send_request(request).await.unwrap();
    debug!("Result: {:?}", result);
    assert!(format!("{:?}", result).contains("Tokyo"));
}
