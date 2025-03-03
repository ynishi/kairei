# Provider Configuration Validation Reference

This document provides a comprehensive reference for provider configuration validation in KAIREI, including an overview of provider configurations, the validation process, error handling, common validation scenarios, and troubleshooting.

## 1. Provider Configuration Overview

### 1.1 Purpose and Structure

Provider configurations define how providers operate within the KAIREI system. They specify:

- Provider type and identity
- Provider-specific parameters
- Required capabilities
- Dependencies on other components

A provider configuration is represented as a key-value map, typically serialized as JSON:

```json
{
  "type": "memory",
  "ttl": 3600,
  "capabilities": {
    "memory": true
  }
}
```

### 1.2 Provider Types and Requirements

KAIREI supports several provider types, each with specific configuration requirements:

#### Memory Provider

- **Required Fields**:
  - `type`: Must be "memory"
- **Optional Fields**:
  - `ttl`: Time-to-live in seconds (number, must be > 0)
- **Capabilities**:
  - Requires `memory` capability
- **Deprecated Fields**:
  - `legacy_mode`: Use standard configuration instead

#### RAG (Retrieval-Augmented Generation) Provider

- **Required Fields**:
  - `type`: Must be "rag"
  - `chunk_size`: Size of text chunks for processing (number, must be > 0)
  - `max_tokens`: Maximum tokens to process (number, must be > 0)
- **Optional Fields**:
  - `similarity_threshold`: Threshold for similarity matching (number, between 0.0 and 1.0)
- **Capabilities**:
  - Requires `rag` capability
- **Deprecated Fields**:
  - `use_legacy_chunking`: Use `chunking_strategy` instead
  - `similarity_method`: Use `similarity_strategy` instead

#### Search Provider

- **Required Fields**:
  - `type`: Must be "search"
  - `max_results`: Maximum number of search results (number, must be > 0)
- **Capabilities**:
  - Requires `search` capability
- **Deprecated Fields**:
  - `use_fuzzy`: Use `search_strategy` with value `fuzzy` instead

## 2. Validation Process

### 2.1 Validation Phases

Provider configuration validation in KAIREI occurs in two distinct phases:

1. **Compile-Time Validation (Type Checking)**:
   - Performed by the `TypeCheckerValidator`
   - Validates schema structure and types
   - Checks for required fields
   - Generates warnings for deprecated fields

2. **Runtime Validation (Evaluation)**:
   - Performed by the `EvaluatorValidator`
   - Validates provider-specific constraints
   - Verifies capability requirements
   - Validates dependencies
   - Generates warnings for suboptimal configurations

### 2.2 Validation Stages

Each validation phase includes multiple stages:

1. **Schema Validation**:
   - Ensures the configuration has the correct structure
   - Verifies that required fields are present
   - Checks that field values have the correct types

2. **Provider-Specific Validation**:
   - Validates constraints specific to each provider type
   - Ensures numeric values are within valid ranges
   - Verifies that string values match expected formats

3. **Capability Validation**:
   - Ensures the provider has the required capabilities
   - Verifies capability configuration is correct

4. **Dependency Validation**:
   - Validates that dependencies are properly configured
   - Checks dependency version formats
   - Ensures required dependencies are available

### 2.3 Validation Workflow

The typical workflow for validating provider configurations is:

1. **Create Configuration**: Define the provider configuration as a key-value map
2. **Type Check**: Validate the schema structure and types using `TypeCheckerValidator`
3. **Evaluate**: Validate provider-specific constraints, capabilities, and dependencies using `EvaluatorValidator`
4. **Handle Errors**: Process any validation errors or warnings
5. **Initialize Provider**: If validation passes, initialize the provider with the validated configuration

## 3. Error Handling Guide

### 3.1 Error Hierarchy

KAIREI uses a comprehensive error hierarchy for provider configuration validation:

- `ProviderConfigError`: Top-level error type
  - `SchemaError`: Errors related to schema validation
  - `ValidationError`: Errors related to value validation
  - `ProviderError`: Provider-specific errors

### 3.2 Error Types

#### Schema Errors

- `MissingField`: A required field is missing
- `InvalidType`: A field has an incorrect type
- `InvalidStructure`: The overall structure is invalid

#### Validation Errors

- `InvalidValue`: A field has an invalid value
- `ConstraintViolation`: A constraint was violated
- `DependencyError`: A dependency requirement was not satisfied

#### Provider Errors

- `Initialization`: Error during provider initialization
- `Capability`: Error related to provider capabilities
- `Configuration`: Error in provider configuration

### 3.3 Error Context

Each error includes rich contextual information:

- **Source Location**: File, line, column, and field name
- **Severity Level**: Critical, Error, Warning, or Info
- **Documentation Reference**: Link to relevant documentation
- **Suggestion**: Recommended fix for the error
- **Error Code**: Unique identifier for the error
- **Additional Context**: Extra information about the error

### 3.4 Error Severity Levels

- **Critical**: Errors that prevent the system from functioning
- **Error**: Standard errors that affect functionality
- **Warning**: Issues that should be addressed but don't affect core functionality
- **Info**: Informational messages about potential issues

### 3.5 Collecting Multiple Errors

Instead of stopping at the first error, you can collect all validation errors using the `validate_collecting` method:

```rust
let errors = validator.validate_collecting(&config);
for error in errors {
    println!("Validation error: {}", error);
}
```

## 4. Common Validation Scenarios

### 4.1 Valid Configuration Examples

#### Memory Provider

```json
{
  "type": "memory",
  "ttl": 3600,
  "capabilities": {
    "memory": true
  }
}
```

#### RAG Provider

```json
{
  "type": "rag",
  "chunk_size": 512,
  "max_tokens": 1000,
  "similarity_threshold": 0.7,
  "capabilities": {
    "rag": true
  }
}
```

#### Search Provider

```json
{
  "type": "search",
  "max_results": 50,
  "capabilities": {
    "search": true
  }
}
```

### 4.2 Common Validation Errors

#### Missing Required Field

```json
{
  "ttl": 3600
  // Missing required "type" field
}
```

**Error**: `SchemaError::MissingField`  
**Fix**: Add the missing required field: `"type": "memory"`

#### Invalid Type

```json
{
  "type": "memory",
  "ttl": "3600" // String instead of number
}
```

**Error**: `SchemaError::InvalidType`  
**Fix**: Change the value to a number: `"ttl": 3600`

#### Invalid Value

```json
{
  "type": "memory",
  "ttl": 0 // Must be greater than 0
}
```

**Error**: `ValidationError::InvalidValue`  
**Fix**: Use a positive value: `"ttl": 3600`

#### Missing Capability

```json
{
  "type": "memory",
  "ttl": 3600,
  "capabilities": {
    // Missing "memory" capability
  }
}
```

**Error**: `ProviderError::Capability`  
**Fix**: Add the required capability: `"memory": true`

#### Invalid Dependency Format

```json
{
  "dependencies": [
    {
      "name": "some-lib",
      "version": "1" // Invalid format, must be x.y.z
    }
  ]
}
```

**Error**: `ValidationError::DependencyError`  
**Fix**: Use the correct version format: `"version": "1.0.0"`

## 5. Troubleshooting Guide

### 5.1 Diagnosing Validation Issues

1. **Identify the Error Type**:
   - Check the error type to understand what kind of validation failed
   - Look at the error message for specific details

2. **Check the Source Location**:
   - The error will include the field name where the error occurred
   - Use this to locate the problematic part of your configuration

3. **Review Documentation References**:
   - Follow any documentation links provided in the error
   - These will provide more context and guidance

4. **Consider Suggestions**:
   - The error will often include a suggested fix
   - Apply the suggestion to resolve the issue

### 5.2 Common Pitfalls and Solutions

#### Schema Validation Failures

**Pitfall**: Missing required fields or incorrect types  
**Solution**: Review the provider type requirements and ensure all required fields are present with the correct types

#### Provider-Specific Validation Failures

**Pitfall**: Values outside of valid ranges  
**Solution**: Check the valid ranges for numeric values and ensure they meet the requirements

#### Capability Validation Failures

**Pitfall**: Missing required capabilities  
**Solution**: Ensure the provider has all required capabilities enabled

#### Dependency Validation Failures

**Pitfall**: Invalid dependency configurations  
**Solution**: Verify that all dependencies have the required fields and use the correct version format

### 5.3 Handling Warnings

Warnings indicate non-critical issues that should be addressed but don't prevent the system from functioning:

1. **Review Warning Messages**:
   - Check the warning message to understand the issue
   - Consider the potential impact on performance or quality

2. **Apply Suggestions**:
   - Warnings include suggestions for improvement
   - Apply these suggestions to optimize your configuration

3. **Balance Trade-offs**:
   - Some warnings involve trade-offs (e.g., performance vs. quality)
   - Choose the configuration that best meets your requirements

### 5.4 Validation Best Practices

1. **Start with Templates**:
   - Use the example configurations as templates
   - Modify them to meet your specific requirements

2. **Validate Incrementally**:
   - Validate your configuration after each significant change
   - This makes it easier to identify the source of errors

3. **Collect All Errors**:
   - Use `validate_collecting` to get all validation errors at once
   - This is more efficient than fixing one error at a time

4. **Pay Attention to Warnings**:
   - Don't ignore warnings, even though they don't prevent operation
   - Addressing warnings can improve performance and quality

5. **Keep Dependencies Updated**:
   - Regularly check and update dependencies
   - This helps avoid compatibility issues

## 6. API Reference

For detailed API documentation, refer to the RustDoc documentation for the following modules:

- `kairei_core::provider::config::validator`: Core validation trait and utilities
- `kairei_core::provider::config::validators::type_checker`: Compile-time type checking
- `kairei_core::provider::config::validators::evaluator`: Runtime validation
- `kairei_core::provider::config::errors`: Error types and handling
