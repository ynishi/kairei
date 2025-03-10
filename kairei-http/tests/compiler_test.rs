use kairei_http::services::compiler::CompilerSystemManager;

#[tokio::test]
async fn test_compiler_system_manager() {
    // Create a new CompilerSystemManager with default configuration
    let manager = CompilerSystemManager::default();

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
        error_string.contains("failed to parse") || error_string.contains("syntax"),
        "Error message should indicate a parsing problem: {}",
        error_string
    );
}

#[tokio::test]
async fn test_compiler_system_manager_create_system() {
    // Create a new CompilerSystemManager with default configuration
    let manager = CompilerSystemManager::default();

    // Create a validation system
    let system = manager.create_validation_system().await;

    // Verify that the system was created successfully
    assert!(
        system
            .ast_registry()
            .read()
            .await
            .list_agent_asts()
            .await
            .is_empty()
    );
}
