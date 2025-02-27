//! # Event-Driven Architecture
//!
//! KAIREI's event-driven architecture is the core mechanism for agent communication and system coordination.
//! It enables loosely coupled interactions between components through a centralized event bus and typed events.
//!
//! ## Architecture Overview
//!
//! The event system consists of the following key components:
//!
//! - **EventBus**: Central hub for publishing and subscribing to events using a broadcast channel
//! - **EventRegistry**: Registry of event types with parameter validation
//! - **RequestManager**: Manages request-response patterns with timeout handling
//!
//! ## Event Flow
//!
//! ```text
//! ┌──────────┐     ┌──────────┐     ┌──────────┐
//! │Publisher │────▶│ EventBus │────▶│Subscriber│
//! └──────────┘     └──────────┘     └──────────┘
//!                       │
//!                       │
//!                  ┌────▼────┐
//!                  │EventType│
//!                  └─────────┘
//! ```
//!
//! 1. Publishers create and publish events to the EventBus
//! 2. The EventBus broadcasts events to all subscribers
//! 3. Subscribers receive and process events based on event type
//!
//! ## Request-Response Pattern
//!
//! For synchronous communication patterns, the system provides a request-response mechanism:
//!
//! ```text
//! ┌─────────┐     ┌──────────┐     ┌──────────┐
//! │Requester│────▶│EventBus  │────▶│Responder │
//! └────┬────┘     └──────────┘     └────┬─────┘
//!      │                                │
//!      │            Response            │
//!      └────────────────────────────────┘
//! ```
//!
//! The RequestManager coordinates this pattern with automatic timeout handling and response matching.
//!
//! ## Usage Examples
//!
//! ### Publishing an Event
//!
//! ```rust,no_run
//! # use kairei::event_bus::{EventBus, Event, Value};
//! # use kairei::event_registry::EventType;
//! # use std::collections::HashMap;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let event_bus = EventBus::new(10);
//! let event = Event {
//!     event_type: EventType::Custom("user_logged_in".to_string()),
//!     parameters: {
//!         let mut params = HashMap::new();
//!         params.insert("user_id".to_string(), Value::String("12345".to_string()));
//!         params
//!     },
//! };
//! event_bus.publish(event).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Subscribing to Events
//!
//! ```rust,no_run
//! # use kairei::event_bus::{EventBus, Event, Value};
//! # use kairei::event_registry::EventType;
//! # fn example() {
//! let event_bus = EventBus::new(10);
//! let (mut event_rx, _) = event_bus.subscribe();
//!
//! tokio::spawn(async move {
//!     while let Ok(event) = event_rx.recv().await {
//!         match event.event_type {
//!             EventType::Custom(name) if name == "user_logged_in" => {
//!                 // Process user login event
//!                 if let Some(Value::String(user_id)) = event.parameters.get("user_id") {
//!                     println!("User logged in: {}", user_id);
//!                 }
//!             },
//!             _ => {} // Ignore other events
//!         }
//!     }
//! });
//! # }
//! ```
//!
//! ### Request-Response Example
//!
//! ```rust,no_run
//! # use kairei::event_bus::{EventBus, Event, Value};
//! # use kairei::event_registry::EventType;
//! # use kairei::event::request_manager::{RequestManager, RequestError};
//! # use std::sync::Arc;
//! # use std::time::Duration;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let event_bus = Arc::new(EventBus::new(10));
//! let request_manager = Arc::new(RequestManager::new(
//!     event_bus.clone(),
//!     Duration::from_secs(5)
//! ));
//!
//! // Create request
//! let request = Event::request_buidler()
//!     .request_type("get_user_info")
//!     .requester("client")
//!     .responder("user_service")
//!     .request_id("request-123".to_string())
//!     .parameter("user_id", &Value::String("12345".to_string()))
//!     .build()?;
//!
//! // Send request and await response
//! match request_manager.request(&request).await {
//!     Ok(response) => {
//!         println!("Received response: {:?}", response.response_value());
//!     },
//!     Err(e) => println!("Request failed: {}", e)
//! }
//! # Ok(())
//! # }
//! ```

pub mod event_bus;
pub mod event_registry;
pub mod request_manager;
