use std::{
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};
use tokio::sync::RwLock;

use kairei_core::{
    config::ProviderConfig,
    event_bus::Value,
    event_registry::EventType,
    provider::{
        capabilities::common::Capabilities,
        llm::LLMResponse,
        provider::{Provider, ProviderSecret},
        request::{ProviderContext, ProviderRequest, ProviderResponse},
        types::{ProviderError, ProviderResult},
    },
    system::System,
};
use tokio::time::sleep;
use uuid::Uuid;

use crate::{micro_agent_tests::create_request, should_run_external_api_tests};

use super::setup_system;

const ON_FAIL_DSL: &str = r#"
world ProcessWorld {}

micro SearchAgent {
    answer {
        on request Search(query: String) -> Result<String, Error> {
                result = think("Search for ${query}") with {
                    max_tokens: 100
                } onFail(err) {
                    emit SearchError(err)
                }
                return Ok(result)
        }
    }
}

micro ProcessAgent {
    answer {
        on request Proccess(data: String) -> Result<String, Error> {
            search_result = request Search to SearchAgent(query: data)
            result = think("Process ${data} with ${search_result}") onFail(err) {
                emit ProccessError(err)
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

// テスト用のモックProvider
#[derive(Default)]
#[allow(dead_code)]
struct MockProvider {
    should_fail: AtomicBool,
}

#[async_trait::async_trait]
impl Provider for MockProvider {
    async fn execute(
        &self,
        _context: &ProviderContext,
        _request: &ProviderRequest,
    ) -> ProviderResult<ProviderResponse> {
        if self.should_fail.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(ProviderError::InternalError(
                "MockProvider failed".to_string(),
            ));
        }
        Ok(ProviderResponse::from(LLMResponse {
            content: "Tokyo, in short".to_string(),
            metadata: Default::default(),
        }))
    }
    async fn capabilities(&self) -> Capabilities {
        Capabilities::default()
    }

    fn name(&self) -> &str {
        "MockProvider"
    }

    // validate the provider configuration
    async fn initialize(
        &mut self,
        _config: &ProviderConfig,
        _secret: &ProviderSecret,
    ) -> ProviderResult<()> {
        Ok(())
    }
}

async fn setup_on_fail() -> System {
    setup_system(SYSTEM_CONFIG, ON_FAIL_DSL, &["SearchAgent", "ProcessAgent"]).await
}

#[tokio::test]
async fn test_on_fail() {
    if !should_run_external_api_tests() {
        return;
    }

    let system = setup_on_fail().await;
    system.start().await.unwrap();
    sleep(Duration::from_millis(100)).await;
    let event_bus_ref = system.event_bus().clone();
    let recv_events = Arc::new(RwLock::new(vec![]));
    let recv_events_ref = recv_events.clone();
    tokio::spawn(async move {
        let (mut rx, _) = event_bus_ref.subscribe();
        while let Ok(event) = rx.recv().await {
            if matches!(event.event_type, EventType::Tick { .. }) {
                continue;
            }
            recv_events_ref.write().await.push(event);
        }
    });

    let request_data = vec![("data", Value::from("Tokyo, in short"))];
    let request_id = Uuid::new_v4();
    let request = create_request("ProcessAgent", &request_id, "Proccess", request_data, None);

    let result = system.send_request(request).await;
    // 処理は成功するが、内部でエラーが発生している
    assert!(result.is_ok());
    let events = recv_events.read().await.clone();
    assert!(
        events
            .iter()
            .any(|e| e.event_type == EventType::Custom("ProccessError".to_string()))
    );
    assert!(
        events
            .iter()
            .any(|e| e.event_type == EventType::Custom("SearchError".to_string()))
    );
}
