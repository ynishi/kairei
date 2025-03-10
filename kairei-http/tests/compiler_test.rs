use kairei_http::services::compiler::CompilerSystemManager;

#[tokio::test]
async fn test_compiler_system_manager() {
    // Create a new CompilerSystemManager with default configuration
    let mut manager = CompilerSystemManager::default();
    manager.initialize(false).await.unwrap();

    // Test validation with valid DSL code
    let valid_code = "micro TestAgent { }";
    let result = manager.validate_dsl(valid_code).await;
    assert!(
        result.is_ok(),
        "Valid DSL code should be parsed successfully"
    );

    // Test validation with invalid DSL code
    let invalid_code = "micro TestAgent { invalid syntax }";
    let result = manager.validate_dsl(invalid_code).await;
    assert!(result.is_err(), "Invalid DSL code should return an error");

    // Check that the error message contains useful information
    let error = result.unwrap_err();
    let error_string = error.to_string();
    assert!(
        error_string.to_lowercase().contains("failed to parse") || error_string.contains("syntax"),
        "Error message should indicate a parsing problem: {}",
        error_string
    );
}

#[tokio::test]
async fn test_compiler_system_manager_create_system() {
    // Create a new CompilerSystemManager with default configuration
    let mut manager = CompilerSystemManager::default();

    // Create a validation system
    manager.initialize(false).await.unwrap();
}
