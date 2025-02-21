# Type Checker - Event Handler Type Checking

## Overview

This document describes the type checking rules and implementation details for Event Handlers in the KAIREI type system. Event Handlers are a core component of the system, requiring specific type checking considerations due to their unique role in handling various types of events and maintaining state.

## Event Handler Types

### 1. Answer Handlers
```rust
on answer {
    // Must return Result<String, Error>
    return Ok("Response text");
}
```
- Return type must be Result<String, Error>
- String responses for direct communication
- Error handling for response failures

### 2. Observe Handlers
```rust
on observe {
    // Can return Result<Any, Error>
    return Ok(observed_data);
}
```
- Return type can be Result<Any, Error>
- Flexible return types for different observation scenarios
- Error handling for observation failures

### 3. React Handlers
```rust
on react {
    // Typically return Result<Unit, Error>
    perform_action();
    return Ok(());
}
```
- Usually return Result<Unit, Error>
- Focus on side effects rather than return values
- Error handling for action failures

### 4. Lifecycle Handlers
```rust
lifecycle {
    on_init {
        // Initialization logic
        return Ok(());
    }
    on_destroy {
        // Cleanup logic
        return Ok(());
    }
}
```
- Return Result<Unit, Error>
- Specific to initialization and cleanup
- Error handling for lifecycle operations

## Type Checking Rules

### 1. Return Type Validation

All handlers must follow specific return type rules:

```rust
// Basic structure
Result<T, Error>

// Handler-specific types
Answer   -> Result<String, Error>
Observe  -> Result<Any, Error>
React    -> Result<Unit, Error>
Lifecycle-> Result<Unit, Error>
```

Key validation points:
- All handlers must return a Result type
- The error type must be Error
- The success type must match the handler type
- Proper wrapping with Ok() or Err() is required

### 2. State Access Rules

Event Handlers have specific rules for state access:

```rust
state {
    counter: Int,
    config: Config,
    data: CustomType,
}

on answer {
    // State access validation
    state.counter += 1;
    let cfg = state.config;
    return Ok(state.data.to_string());
}
```

Validation requirements:
- State variables must be defined in the state block
- Access paths must be valid
- Type compatibility must be maintained
- Mutations must respect type constraints

### 3. Error Handling

Error handling must follow these rules:

```rust
// Error propagation
with_error {
    risky_operation()?;
} handle_error {
    return Err(error_message);
}
```

Requirements:
- Error types must be convertible to Error
- Error handlers must maintain return type consistency
- Proper error propagation using ? operator

## Implementation Details

### 1. Type Checking Process

The type checker performs these validations:

```rust
fn check_handler(&self, handler: &HandlerDef) -> TypeCheckResult<()> {
    // 1. Validate return type
    self.check_return_type(handler.block)?;
    
    // 2. Validate state access
    self.check_state_access(handler.block)?;
    
    // 3. Validate error handling
    self.check_error_handling(handler.block)?;
    
    Ok(())
}
```

### 2. Context Management

The type checker maintains context for each handler:

```rust
pub struct HandlerContext {
    // Handler-specific information
    handler_type: HandlerType,
    return_type: TypeInfo,
    
    // State access information
    state_vars: HashMap<String, TypeInfo>,
    
    // Error handling context
    in_error_handler: bool,
}
```

### 3. Error Reporting

The type checker provides detailed error messages:

```rust
// Return type errors
InvalidEventHandlerReturn {
    message: String,
    expected: TypeInfo,
    found: TypeInfo,
    help: Option<String>,
    suggestion: Option<String>,
}

// State access errors
InvalidEventHandlerStateAccess {
    message: String,
    help: Option<String>,
    suggestion: Option<String>,
}
```

## Future Considerations

1. Type System Extensions
- Support for generic event handlers
- Custom return type constraints
- Advanced state access patterns

2. Error Handling Improvements
- More detailed error messages
- Context-aware suggestions
- Better error recovery strategies

3. Performance Optimizations
- Caching of type information
- Efficient state access validation
- Optimized error handling paths
