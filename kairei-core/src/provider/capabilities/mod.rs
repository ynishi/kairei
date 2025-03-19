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
//!  ├── RAGMemoryCapability
//!  └── StorageCapability
//! ```
//!
//! # Usage Example
//!
//! ```no_run
//! use kairei_core::provider::capabilities::shared_memory::SharedMemoryCapability;
//! use kairei_core::provider::capabilities::common::rag_memory::RAGMemoryCapability;
//! use serde_json::json;
//!
//! # async fn example(
//! #     shared_memory: &impl SharedMemoryCapability,
//! #     rag_memory: &impl RAGMemoryCapability
//! # ) -> Result<(), Box<dyn std::error::Error>> {
//! // Store data in working memory
//! shared_memory.set("user_123", json!({"name": "Alice"})).await?;
//!
//! // Record information for semantic search
//! let content = "Alice is working on the new AI project and needs access to the research database.";
//! rag_memory.record(content, "conversation", None).await?;
//!
//! // Search for relevant information
//! let context = rag_memory::SearchContext::default();
//! let results = rag_memory.search("research database access", &context).await?;
//! # Ok(())
//! # }
//! ```

pub mod common;
pub mod shared_memory;
pub mod storage;
pub mod will_action;
