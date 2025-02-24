use crate::provider::config::plugins::{MemoryConfig, RagConfig, SearchConfig};
use super::super::providers::{OpenAIApiConfig, OpenAIRagConfig};
use crate::provider::config::base::ConfigError;
use std::time::Duration;

#[test]
fn test_rag_config_validation() {
    // Valid configuration
    let config = RagConfig {
        chunk_size: 512,
        max_tokens: 1000,
        similarity_threshold: 0.7,
        ..Default::default()
    };
    assert!(config.validate().is_ok());

    // Invalid chunk size
    let config = RagConfig {
        chunk_size: 0,
        ..Default::default()
    };
    assert!(matches!(
        config.validate(),
        Err(ConfigError::InvalidValue { field, .. }) if field == "chunk_size"
    ));

    // Invalid similarity threshold
    let config = RagConfig {
        similarity_threshold: 1.5,
        ..Default::default()
    };
    assert!(matches!(
        config.validate(),
        Err(ConfigError::InvalidValue { field, .. }) if field == "similarity_threshold"
    ));
}

#[test]
fn test_memory_config_validation() {
    // Valid configuration
    let config = MemoryConfig {
        max_items: 1000,
        ttl: Duration::from_secs(3600),
        importance_threshold: 0.5,
        ..Default::default()
    };
    assert!(config.validate().is_ok());

    // Invalid max items
    let config = MemoryConfig {
        max_items: 0,
        ..Default::default()
    };
    assert!(matches!(
        config.validate(),
        Err(ConfigError::InvalidValue { field, .. }) if field == "max_items"
    ));

    // Invalid importance threshold
    let config = MemoryConfig {
        importance_threshold: -0.1,
        ..Default::default()
    };
    assert!(matches!(
        config.validate(),
        Err(ConfigError::InvalidValue { field, .. }) if field == "importance_threshold"
    ));
}

#[test]
fn test_search_config_validation() {
    // Valid configuration
    let config = SearchConfig {
        max_results: 10,
        search_window: Duration::from_secs(3600),
        filters: vec!["news".to_string()],
        ..Default::default()
    };
    assert!(config.validate().is_ok());

    // Invalid max results
    let config = SearchConfig {
        max_results: 0,
        ..Default::default()
    };
    assert!(matches!(
        config.validate(),
        Err(ConfigError::InvalidValue { field, .. }) if field == "max_results"
    ));
}

#[test]
fn test_openai_rag_config() {
    // Valid configuration
    let config = OpenAIRagConfig {
        base: RagConfig::default(),
        api_config: OpenAIApiConfig {
            model: "gpt-4".to_string(),
            api_version: Some("v1".to_string()),
            organization_id: None,
        },
    };
    assert!(config.validate().is_ok());

    // Invalid model name
    let config = OpenAIRagConfig {
        base: RagConfig::default(),
        api_config: OpenAIApiConfig {
            model: "".to_string(),
            api_version: None,
            organization_id: None,
        },
    };
    assert!(matches!(
        config.validate(),
        Err(ConfigError::InvalidValue { field, .. }) if field == "model"
    ));
}
