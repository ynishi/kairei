//! Tests for SistenceStorageService and integration with StatelessRelevantMemory.

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};

    use crate::provider::capabilities::relevant_memory::{
        AccessStats, DetailedImportanceEvaluation, DetailedMemoryItem,
    };
    use crate::provider::capabilities::sistence_memory::SistenceMemoryError;
    use crate::provider::capabilities::sistence_storage::SistenceStorageService;
    use crate::provider::capabilities::storage::StorageBackend;
    use crate::provider::llm::ProviderLLM;
    use crate::provider::plugins::memory::stateless_relevant_memory::StatelessRelevantMemory;
    use crate::provider::plugins::storage::in_memory::InMemoryStorageBackend;
    use crate::provider::plugins::storage::sistence_storage::{
        SistenceStorage, SistenceStorageConfig,
    };

    // Mock LLM provider for testing
    struct MockLLM;

    #[async_trait::async_trait]
    impl ProviderLLM for MockLLM {
        async fn request(
            &self,
            _prompt: &str,
        ) -> Result<crate::provider::llm::LLMResponse, crate::provider::types::ProviderError>
        {
            Ok(crate::provider::llm::LLMResponse {
                content: "Mock response".to_string(),
                tokens: 10,
                finish_reason: Some("stop".to_string()),
                usage: None,
            })
        }

        fn name(&self) -> &str {
            "MockLLM"
        }

        fn provider_type(&self) -> crate::provider::provider::ProviderType {
            crate::provider::provider::ProviderType::SimpleExpert
        }
    }

    // Helper to create a test memory item
    fn create_test_memory_item(id: &str) -> DetailedMemoryItem {
        DetailedMemoryItem {
            id: id.to_string(),
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            content: format!("Test content for {}", id),
            content_type: Some("text/plain".to_string()),
            structured_content: None,
            item_type: Some("note".to_string()),
            topics: vec!["test".to_string(), "example".to_string()],
            tags: HashMap::from([("key1".to_string(), "value1".to_string())]),
            source: None,
            references: Vec::new(),
            related_items: Vec::new(),
            importance: DetailedImportanceEvaluation {
                base_score: 0.5,
                context_score: 0.5,
                factor_scores: HashMap::new(),
                evaluated_at: SystemTime::now(),
            },
            access_stats: AccessStats {
                last_accessed: SystemTime::now(),
                access_count: 0,
            },
            ttl: Some(86400),
            retention_policy: None,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_basic_storage_operations() {
        // Create a storage backend
        let backend = Arc::new(InMemoryStorageBackend::new());

        // Create SistenceStorage
        let sistence_storage = Arc::new(SistenceStorage::new(
            "test".to_string(),
            backend.clone(),
            None,
            SistenceStorageConfig::default(),
        ));

        // Create test memory item
        let item_id = "test-item-1";
        let test_item = create_test_memory_item(item_id);

        // Save item
        let result = sistence_storage
            .save(
                "memory_items",
                item_id,
                &test_item,
                None,
                Some(Duration::from_secs(86400)),
                None,
            )
            .await;

        assert!(result.is_ok(), "Failed to save item: {:?}", result);

        // Retrieve item
        let retrieved = sistence_storage
            .get::<DetailedMemoryItem>("memory_items", item_id, None)
            .await;

        assert!(
            retrieved.is_ok(),
            "Failed to retrieve item: {:?}",
            retrieved
        );
        let retrieved_item = retrieved.unwrap();

        // Verify content
        assert_eq!(retrieved_item.value.id, test_item.id);
        assert_eq!(retrieved_item.value.content, test_item.content);
    }

    #[tokio::test]
    async fn test_stateless_relevant_memory_integration() {
        // Skip test if we're running a limited test suite
        if std::env::var("QUICK_TEST").is_ok() {
            return;
        }

        // Create storage backends
        let backend = Arc::new(InMemoryStorageBackend::new());
        let llm = Arc::new(MockLLM);

        // Create SistenceStorage
        let sistence_storage = Arc::new(SistenceStorage::new(
            "test".to_string(),
            backend.clone(),
            None,
            SistenceStorageConfig::default(),
        ));

        // Create StatelessRelevantMemory with SistenceStorage
        let mut srm = StatelessRelevantMemory::new(
            "test-srm".to_string(),
            backend.clone(),
            llm,
            crate::config::ProviderConfig::default(),
        );

        // Set SistenceStorageService
        srm.set_sistence_storage(sistence_storage);

        // Create test memory item
        let item_id = "test-item-2";
        let test_item = create_test_memory_item(item_id);

        // Store the item
        let store_result = srm.store_to_storage(&test_item).await;
        assert!(
            store_result.is_ok(),
            "Failed to store item: {:?}",
            store_result
        );

        // Retrieve the item
        let retrieve_result = srm.retrieve_from_storage(item_id).await;
        assert!(
            retrieve_result.is_ok(),
            "Failed to retrieve item: {:?}",
            retrieve_result
        );

        let retrieved_item = retrieve_result.unwrap();
        assert_eq!(retrieved_item.id, test_item.id);
        assert_eq!(retrieved_item.content, test_item.content);
    }

    #[tokio::test]
    async fn test_workspace_operations() {
        // Create storage backends
        let backend = Arc::new(InMemoryStorageBackend::new());

        // Create SistenceStorage
        let sistence_storage = Arc::new(SistenceStorage::new(
            "test".to_string(),
            backend.clone(),
            None,
            SistenceStorageConfig::default(),
        ));

        // Create workspaces
        let create_result = sistence_storage
            .create_workspace("memory_items", "workspace1", None)
            .await;
        assert!(
            create_result.is_ok(),
            "Failed to create workspace: {:?}",
            create_result
        );

        // Save item to workspace
        let item_id = "workspace-test-item";
        let test_item = create_test_memory_item(item_id);

        let save_result = sistence_storage
            .save(
                "memory_items",
                item_id,
                &test_item,
                None,
                None,
                Some("workspace1"),
            )
            .await;

        assert!(
            save_result.is_ok(),
            "Failed to save item to workspace: {:?}",
            save_result
        );

        // Retrieve from workspace
        let retrieve_result = sistence_storage
            .get::<DetailedMemoryItem>("memory_items", item_id, Some("workspace1"))
            .await;

        assert!(
            retrieve_result.is_ok(),
            "Failed to retrieve item from workspace: {:?}",
            retrieve_result
        );

        // Verify workspace isolation (item shouldn't be in the main workspace)
        let main_retrieve_result = sistence_storage
            .get::<DetailedMemoryItem>("memory_items", item_id, None)
            .await;

        assert!(
            main_retrieve_result.is_err(),
            "Item should not be in main workspace"
        );
    }
}
