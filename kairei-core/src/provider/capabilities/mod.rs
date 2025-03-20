//! Provider capabilities for the Kairei system.
//!
//! This module contains the capability traits that define different functionalities
//! that provider plugins can implement. These capabilities form the interface between
//! the Kairei runtime and provider-specific implementations.
//!
//! # Available Capabilities
//!
//! - **SharedMemory**: High-performance key-value storage for sharing data between agents
//! - **PersistentSharedMemory**: Shared memory with persistence to storage backends
//! - **SistenceMemory**: Metadata-enriched middle layer of the 3-layer memory architecture
//! - **RAGMemory**: Semantic search and memory organization using lightweight LLMs
//! - **Storage**: Storage operations for provider plugins
//! - **WillAction**: Will action resolution for provider plugins
//! - ... and more
//!
//! # Capability Architecture
//!
//! Each capability is defined as a trait that extends the `ProviderPlugin` trait.
//! This design allows plugins to implement multiple capabilities while maintaining
//! a common plugin interface.
//!
//! ```text
//! ProviderPlugin (base trait)
//!  ├── SharedMemoryCapability
//!  ├── SistenceMemoryCapability
//!  ├── RAGMemoryCapability
//!  └── StorageCapability
//! ```
//!
//! # Usage Example
//!
//! ```ignore,no_run
//! use kairei_core::provider::capabilities::shared_memory::SharedMemoryCapability;
//! use kairei_core::provider::capabilities::sistence_memory::{SistenceMemoryCapability, MemoryItem, ItemType};
//! use serde_json::json;
//!
//! # async fn example(
//! #     shared_memory: &impl SharedMemoryCapability,
//! #     sistence_memory: &impl SistenceMemoryCapability,
//! # ) -> Result<(), Box<dyn std::error::Error>> {
//! // Store data in working memory
//! shared_memory.set("user_123", json!({"name": "Alice"})).await?;
//!
//! // Store information with rich metadata
//! let item = MemoryItem::new(
//!     "Alice is working on the new AI project and needs access to the research database.",
//!     ItemType::Information,
//!     vec!["access request", "research", "project"],
//! );
//! let item_id = sistence_memory.store(item).await?;
//!
//! # Ok(())
//! # }
//! ```

pub mod common;
pub mod relevant_memory;
pub mod shared_memory;
pub mod sistence_memory;
pub mod storage;
pub mod will_action;
