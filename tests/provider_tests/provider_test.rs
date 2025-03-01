use std::{collections::HashMap, time::Duration};

use kairei::{
    config::{SecretConfig, SystemConfig},
    event_bus::{Event, Value},
    system::System,
};
use tokio::time::sleep;
use tracing::debug;

#[tokio::test]
async fn test_travel_agent_basic_flow() {
    // テスト用の設定とシークレット
    let config: SystemConfig = serde_json::from_str(
        r#"
        {
            "provider_configs": {
                "primary_provider": "simple_expert",
                "providers": {
                    "simple_expert": {
                        "name": "simple_expert",
                        "provider_type": "SimpleExpert",
                        "provider_specific": {
                            "type": "simple_expert",
                            "assistant_id": "assistant_id",
                            "Tokyo": "Tokyo is a great city!",
                            "Osaka": "Osaka is a great city!",
                            "Kyoto": "Kyoto is a great city!"
                        },
                        "plugin_configs": {}
                    }
                }
            }
        }
    "#,
    )
    .unwrap();

    let secret_config: SecretConfig = serde_json::from_str(
        r#"
        {
            "providers": {
                "simple_expert": {
                    "api_key": "test_key",
                    "additional_auth": {
                        "test_secret": "test_secret"
                    }
                }
            }
        }
    "#,
    )
    .unwrap();

    // テスト用のDSL
    let dsl = r#"
        micro TravelAgent {
            answer {
                on request PlanTrip() -> Result<String, Error> {
                    return think("Tokyo")
                }
            }
        }
    "#;

    debug!("Config: {:?}", config);
    debug!("Secret Config: {:?}", secret_config);

    // Initialize system
    let mut system = System::new(&config, &secret_config).await;

    let root = system.parse_dsl(&dsl).await.unwrap();

    debug!("Root: {:?}", root);

    system.initialize(root).await.unwrap();

    // Start system
    system.start().await.unwrap();

    sleep(Duration::from_millis(100)).await;

    let request_id = uuid::Uuid::new_v4();

    let response = system
        .send_request(Event {
            event_type: kairei::event_registry::EventType::Request {
                request_type: "PlanTrip".to_string(),
                requester: "test".to_string(),
                responder: "TravelAgent".to_string(),
                request_id: request_id.to_string(),
            },
            parameters: {
                let mut hashmap = std::collections::HashMap::new();
                hashmap.insert(
                    "destination".to_string(),
                    kairei::event_bus::Value::String("Tokyo".to_string()),
                );
                hashmap
            },
        })
        .await
        .unwrap();
    sleep(Duration::from_millis(100)).await;

    assert_eq!(
        response,
        Value::Map(HashMap::from_iter(vec![(
            "output".to_string(),
            Value::String("Tokyo is a great city!".to_string())
        )]))
    );
}
