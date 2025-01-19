use std::{collections::HashMap, time::Duration};

use kairei::{
    config::{self, SystemConfig},
    event_bus::{Event, Value},
    system::System,
};
use tokio::time::sleep;
use tracing::debug;
use uuid::Uuid;

use crate::{micro_agent_tests::setup_secret, should_run_external_api_tests};

const TRAVEL_PLANNING_DSL: &str = r#"
world TravelPlanning {
}
micro TravelPlanner {
    state {
        current_plan: String = "none",
        planning_stage: String = "none"
    }
    answer {
        // create a comprehensive travel plan
        on request PlanTrip(destination: String, start: String, end: String, budget: Float) -> Result<String, Error> {
            plan = think("Create a comprehensive travel plan", destination, start, end, budget)
            return Ok(plan)
        }
    }
}

micro HotelFinder {
    answer {
        // web search for hotels
        on request FindHotels(location: String, start_date: String, end_date: String, budget: Float) {
            hotels = think("Find suitable hotels matching criteria", location, check_in: start_date, check_out: end_date, budget) with {
                search: {
                    filter: "hotels",
                    recent: "24h"
                }
            }
            return Ok(hotels)
        }
    }
}

micro AttractionRecommender {
    answer {
        on request RecommendAttractions(location: String, interests: [String]) {
            think("Recommend attractions based on interests") with {
                search: {
                    filter: ["attractions", "reviews"]
                }
            }
        }
    }
}

    micro LocalExpert {
    answer {
        on request GetLocalInfo(location: String) {
            think("Provide local insights") with {
                search: {
                    filter: ["news", "blogs"],
                    recent: "7d"
                }
            }
        }
    }
}

micro BudgetOptimizer {
    answer {
        on request OptimizeBudget(
            total_budget: Float,
            allocations: [Allocation]
        ) -> Result<OptimizedBudget> {
            think("Optimize budget allocation") with {
                context: {
                    total_budget,
                    allocations
                }
            }
        }
    }
}
"#;

const SYSTEM_CONFIG: &str = r#"
{
  "provider_configs": {
    "primary_provider": "travel_planner",
    "providers": {
      "travel_planner": {
        "name": "travel_planner",
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

fn create_request(
    agnent_name: &str,
    request_id: &Uuid,
    request_type: &str,
    requests: Vec<(&str, &str)>,
    timeout: Option<u64>,
) -> Event {
    let mut builder = Event::request_buidler()
        .request_type(request_type)
        .requester("test")
        .responder(agnent_name)
        .request_id(request_id.to_string().as_str());

    for request in requests.clone() {
        builder = builder
            .clone()
            .parameter(request.0, &Value::String(request.1.to_string()));
    }
    if let Some(timeout) = timeout {
        builder = builder.parameter("timeout", &Value::Duration(Duration::from_secs(timeout)));
    }

    builder.build().unwrap()
}

async fn setup_system() -> System {
    let system_config: SystemConfig = config::from_str(SYSTEM_CONFIG).unwrap();
    let secret = setup_secret();
    debug!("System Config: {:?}", system_config);

    let mut system = System::new(&system_config, &secret).await;

    let root = system.parse_dsl(TRAVEL_PLANNING_DSL).await.unwrap();
    debug!("Root: {:?}", root);
    system.initialize(root).await.unwrap();
    system
}

#[tokio::test]
async fn test_travel_planner() {
    if !should_run_external_api_tests() {
        return;
    }

    let system = setup_system().await;
    system.start().await.unwrap();
    sleep(Duration::from_millis(100)).await;

    let request_data = vec![
        ("destination", "Tokyo"),
        ("start", "2024-06-01"),
        ("end", "2024-06-07"),
        ("budget", "3000.0"),
    ];
    let request_id = Uuid::new_v4();
    let request = create_request("TravelPlanner", &request_id, "PlanTrip", request_data, None);

    let result = system.send_request(request).await.unwrap();
    assert!(format!("{:?}", result).contains("travel plan"));
    assert!(format!("{:?}", result).contains("Tokyo"));
}

#[tokio::test]
async fn test_hotel_finder() {
    if !should_run_external_api_tests() {
        return;
    }

    let system = setup_system().await;
    system.start().await.unwrap();
    sleep(Duration::from_millis(100)).await;

    // リクエストデータの構築
    let request_data = vec![
        ("location", "Tokyo"),
        ("start_date", "2024-06-01"),
        ("end_date", "2024-06-07"),
        ("budget", "3000.0"),
    ];

    let request_id = Uuid::new_v4();
    let request = create_request("HotelFinder", &request_id, "FindHotels", request_data, None);
    let result = system.send_request(request).await.unwrap();

    // 結果の検証
    let result_str = format!("{:?}", result);
    debug!("result_str: {}", result_str);

    // 必須要素の確認
    assert!(result_str.contains("hotel")); // ホテル情報が含まれている
    assert!(result_str.contains("Tokyo")); // 場所の確認
    assert!(result_str.contains("2024-06")); // 日付の確認
    assert!(result_str.contains("price")); // 価格情報の存在確認
}
