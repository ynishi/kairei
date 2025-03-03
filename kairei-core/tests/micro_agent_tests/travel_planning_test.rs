use std::time::Duration;

use kairei_core::{event_bus::Value, system::System};
use tokio::time::sleep;
use tracing::debug;
use uuid::Uuid;

use crate::{micro_agent_tests::create_request, should_run_external_api_tests};

use super::setup_system;

const TRAVEL_PLANNING_DSL: &str = r#"
world TravelPlanning {
  // 共通の基本ポリシー
  policy "Consider budget constraints and optimize value for money"
  policy "Ensure traveler safety and comfort"
  policy "Provide practical and actionable information"
  policy "Consider seasonal factors in all recommendations"
}

micro TravelPlanner {
    // 旅程計画に特化したポリシー
    policy "Create balanced itineraries with appropriate time allocation"

    state {
        current_plan: String = "none";
        planning_stage: String = "none";
    }
    answer {
        // create a comprehensive travel plan
        on request PlanTrip(destination: String, start: String, end: String, budget: Float, interests: String) -> Result<String, Error> {
            (hotels, flights, attractions, local_info) = await (
                request FindHotels to HotelFinder(location: destination, start_date: start,end_date: end, budget: budget * 0.4),
                request FindFlight to FlightFinder(departure_location: "NewYork", arrival_location: destination, departure_date: start, back_date: end, budget :budget * 0.4),
                request FindAttractions to AttractionRecommender(location: destination, dates: "${start} to ${end}", interests: interests, budget: budget * 0.2),
                request GetLocalInfo to LocalExpertAgent(location: destination, season: start, specific_questions: "")
            )
            plan = think("""Create a comprehensive travel plan by combining this flight, hotels, attractions and local information:

                            Destination: ${destination}
                            Dates: ${start} to ${end}
                            Total Budget: ${budget}

                            Flight Information:
                            ${flights}

                            Hotel Information:
                            ${hotels}

                            Attraction Recommendations:
                            ${attractions}

                            Local Information:
                            ${local_info}

                            Please create a detailed itinerary that includes:
                            1. Transportation details (arrival and departure flights)
                            2. Accommodation details
                            3. Daily budget breakdown
                            4. Important logistical notes (check-in/out times, airport transfers)
                            5. Remaining budget for activities and meals

                            Format the response in clear sections with specific dates and times.""") with {
                                max_tokens: 2000
                            }
            return plan
        }
    }
}

micro HotelFinder {
    answer {
        // web search for hotels
        on request FindHotels(location: String, start_date: String, end_date: String, budget: Float) -> Result<String, Error> {
            foundHotels = think("Find suitable hotels matching criteria", location, check_in: start_date, check_out: end_date, budget) with {
                search: {
                    filters: ["hotels"]
                    // recent: "24h"
                }
            }
            return foundHotels
        }
    }
}

micro FlightFinder {
    answer {
        on request FindFlight(departure_location: String, arrival_location: String, departure_date: String, back_date: String, budget: Float) -> Result<String, Error> {
            flights = think("""Provide flight recommendations for:
                            Route: ${departure_location} to ${arrival_location}
                            Departure: ${departure_date} (must include this exact date)
                            Back: ${back_date} (must include this exact date)
                            Budget: ${budget}

                            Please provide:
                            1. Flight options with specific dates (${departure_date} to ${back_date})
                            2. Airlines and routes
                            3. Expected price ranges
                            4. Booking recommendations""")
            return flights
        }
    }
}

micro AttractionRecommender {
    answer {
        on request FindAttractions(
            location: String,
            dates: String,
            interests: String,  // 例: "culture,food,nature"
            budget: Float
        ) -> Result<String, Error> {
            recommendations = think("""Recommend tourist attractions and activities in ${location} that match:
                Dates: ${dates}
                Interests: ${interests}
                Daily budget: ${budget}

                Include:
                1. Major attractions and landmarks
                2. Suggested daily itineraries
                3. Estimated costs
                4. Travel times between locations""")

            return recommendations
        }
    }
}

micro LocalExpertAgent {
    policy "Provide comprehensive weather information and packing suggestions"
    policy "Include local customs, etiquette and cultural considerations"
    policy "Cover local transportation systems and tips"
    policy "Detail safety information and emergency contacts"
    policy "List relevant local festivals and events"

    answer {
        on request GetLocalInfo(
            location: String,
            season: String,  // 旅行時期
            specific_questions: String  // オプショナル
        ) -> Result<String, Error> {
            local_info = think("""Provide detailed local information for ${location}:
                Travel season: ${season}
                Specific questions: ${specific_questions}

                Cover:
                1. Weather conditions and what to pack
                2. Local customs and etiquette
                3. Transportation tips
                4. Safety information
                5. Local emergency contacts
                6. Best areas to stay
                7. Local festivals or events during the period""")

            return local_info
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

async fn setup_travel_planner() -> System {
    setup_system(
        SYSTEM_CONFIG,
        TRAVEL_PLANNING_DSL,
        &[
            "TravelPlanner",
            "HotelFinder",
            "FlightFinder",
            "AttractionRecommender",
            "LocalExpertAgent",
        ],
    )
    .await
}

#[tokio::test]
async fn test_travel_planner() {
    println!("Running test_travel_planner");
    if !should_run_external_api_tests() {
        return;
    }

    let system = setup_travel_planner().await;
    system.start().await.unwrap();
    sleep(Duration::from_millis(100)).await;

    let request_data = vec![
        ("destination", Value::from("Tokyo")),
        ("start", Value::from("2024-06-01")),
        ("end", Value::from("2024-06-07")),
        ("interests", Value::from("culture,food,nature")),
        ("budget", Value::Float(3000.0)),
    ];
    let request_id = Uuid::new_v4();
    let request = create_request("TravelPlanner", &request_id, "PlanTrip", request_data, None);

    let result = system.send_request(request).await.unwrap();
    println!("Result: {:?}", result);
    assert!(format!("{:?}", result).contains("travel"));
    assert!(format!("{:?}", result).contains("Tokyo"));
}

#[tokio::test]
async fn test_hotel_finder() {
    if !should_run_external_api_tests() {
        return;
    }

    let system = setup_travel_planner().await;
    system.start().await.unwrap();
    sleep(Duration::from_millis(100)).await;

    // リクエストデータの構築
    let request_data = vec![
        ("location", Value::from("Tokyo")),
        ("start_date", Value::from("2024-06-01")),
        ("end_date", Value::from("2024-06-07")),
        ("budget", Value::Float(3000.0)),
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
    assert!(
        result_str.contains("2024") && (result_str.contains("06") || result_str.contains("June"))
    ); // 日付の確認
    assert!(result_str.to_lowercase().contains("price")); // 価格情報の存在確認
}

#[tokio::test]
async fn test_flight_finder() {
    if !should_run_external_api_tests() {
        return;
    }

    let system = setup_travel_planner().await;
    system.start().await.unwrap();
    sleep(Duration::from_millis(100)).await;

    // リクエストデータの構築
    let request_data = vec![
        ("departure_location", Value::from("NewYork")),
        ("arrival_location", Value::from("Tokyo")),
        ("departure_date", Value::from("2024-06-01")),
        ("back_date", Value::from("2024-06-07")),
        ("budget", Value::Float(3000.0)),
    ];

    let request_id = Uuid::new_v4();
    let request = create_request(
        "FlightFinder",
        &request_id,
        "FindFlight",
        request_data,
        None,
    );
    let result = system.send_request(request).await.unwrap();

    // 結果の検証
    let result_str = format!("{:?}", result);
    debug!("result_str: {}", result_str);

    // 必須要素の確認
    assert!(result_str.contains("flight")); // フライト情報が含まれている
    assert!(result_str.contains("Tokyo")); // 場所の確認
    assert!(
        result_str.contains("2024") && (result_str.contains("06") || result_str.contains("June"))
    ); // 日付の確認
    assert!(result_str.to_lowercase().contains("price")); // 価格情報の存在確認
}
