# Provider Configuration Validation Quick Reference

This quick reference guide provides essential information for validating provider configurations in KAIREI, including common validation patterns, best practices, and examples of valid configurations.

## Provider Types and Requirements

### Memory Provider

**Required Fields**:
- `type`: Must be "memory"

**Optional Fields**:
- `ttl`: Time-to-live in seconds (number, must be > 0)

**Capabilities**:
- Requires `memory` capability

**Example**:
```json
{
  "type": "memory",
  "ttl": 3600,
  "capabilities": {
    "memory": true
  }
}
```

### RAG Provider

**Required Fields**:
- `type`: Must be "rag"
- `chunk_size`: Size of text chunks (number, must be > 0)
- `max_tokens`: Maximum tokens to process (number, must be > 0)

**Optional Fields**:
- `similarity_threshold`: Threshold for similarity matching (number, between 0.0 and 1.0)

**Capabilities**:
- Requires `rag` capability

**Example**:
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

### Search Provider

**Required Fields**:
- `type`: Must be "search"
- `max_results`: Maximum number of search results (number, must be > 0)

**Capabilities**:
- Requires `search` capability

**Example**:
```json
{
  "type": "search",
  "max_results": 50,
  "capabilities": {
    "search": true
  }
}
```

## Validation Process

### Two-Phase Validation

1. **Compile-Time Validation** (`TypeCheckerValidator`):
   - Schema structure and types
   - Required fields
   - Deprecated fields (warnings)

2. **Runtime Validation** (`EvaluatorValidator`):
   - Provider-specific constraints
   - Capability requirements
   - Dependencies
   - Suboptimal configurations (warnings)

### Validation Code Example

```rust
use kairei::provider::config::validator::ProviderConfigValidator;
use kairei::provider::config::validators::{TypeCheckerValidator, EvaluatorValidator};
use std::collections::HashMap;
use serde_json::json;

// Create validators
let type_checker = TypeCheckerValidator;
let evaluator = EvaluatorValidator;

// Create a configuration
let config = serde_json::from_value(json!({
    "type": "memory",
    "ttl": 3600,
    "capabilities": {
        "memory": true
    }
})).unwrap();

// Validate the configuration
match type_checker.validate(&config) {
    Ok(()) => println!("Type check passed"),
    Err(error) => println!("Type check failed: {}", error),
}

match evaluator.validate(&config) {
    Ok(()) => println!("Evaluation passed"),
    Err(error) => println!("Evaluation failed: {}", error),
}

// Collect all validation errors
let type_check_errors = type_checker.validate_collecting(&config);
let evaluation_errors = evaluator.validate_collecting(&config);
```

## Common Validation Errors and Fixes

### Missing Required Field

**Error**:
```json
{
  "ttl": 3600
  // Missing required "type" field
}
```

**Fix**: Add the missing required field: `"type": "memory"`

### Invalid Type

**Error**:
```json
{
  "type": "memory",
  "ttl": "3600" // String instead of number
}
```

**Fix**: Change the value to a number: `"ttl": 3600`

### Invalid Value

**Error**:
```json
{
  "type": "memory",
  "ttl": 0 // Must be greater than 0
}
```

**Fix**: Use a positive value: `"ttl": 3600`

### Missing Capability

**Error**:
```json
{
  "type": "memory",
  "ttl": 3600,
  "capabilities": {
    // Missing "memory" capability
  }
}
```

**Fix**: Add the required capability: `"memory": true`

## Validation Best Practices

1. **Use Both Validators**:
   - Always use both `TypeCheckerValidator` and `EvaluatorValidator`
   - Type checker validates structure, evaluator validates constraints

2. **Collect All Errors**:
   - Use `validate_collecting` to get all validation errors at once
   - Fix all errors before proceeding

3. **Pay Attention to Warnings**:
   - Warnings indicate non-critical issues that should be addressed
   - Addressing warnings can improve performance and quality

4. **Use Templates**:
   - Start with the example configurations as templates
   - Modify them to meet your specific requirements

5. **Validate Incrementally**:
   - Validate your configuration after each significant change
   - This makes it easier to identify the source of errors

## Recommended Configuration Values

### Memory Provider

- `ttl`: 60-86400 seconds (1 minute to 1 day)
  - < 60 seconds: May impact performance (warning)
  - > 30 days: May impact resource usage (warning)

### RAG Provider

- `chunk_size`: 100-1000
  - < 100: May impact quality (warning)
  - > 1000: May impact performance (warning)
- `similarity_threshold`: 0.3-0.9
  - < 0.3: May impact result quality (warning)
  - > 0.9: May exclude relevant results (warning)

### Search Provider

- `max_results`: 10-100
  - > 100: May impact performance (warning)

## Further Reading

For more detailed information, refer to:

- [Provider Configuration Validation Reference](/docs/reference/provider_config_validation.md)
- API Documentation:
  - `kairei::provider::config::validator`
  - `kairei::provider::config::validators::type_checker`
  - `kairei::provider::config::validators::evaluator`
  - `kairei::provider::config::errors`
