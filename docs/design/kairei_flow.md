# KAIREI Flow Specification

## Overview

The KAIREI Flow system defines the transformation and execution pipeline for the KAIREI DSL, consisting of three primary layers:

1. DSL Layer
   - World DSL: Defines the environment and global configuration
   - MicroAgent DSL: Defines individual agent behaviors

2. AST Layer
   - WorldAgent (MA AST): Transformed World definition
   - Events AST: Extracted event definitions
   - MicroAgent AST: Transformed MicroAgent definitions

3. Runtime Layer
   - Runtime: Executes agent behaviors
   - EventRegistry: Manages event registration
   - EventBus: Handles event distribution

## Flow Sequence

### 1. DSL to AST Transformation
```
WorldDSL --transform--> WorldMA (AST)
WorldDSL --extract--> Events (AST)
MADSL --transform--> MA (AST)
```

### 2. AST to Runtime Registration
```
WorldMA --run--> Runtime
MA --run--> Runtime
Events --register--> EventRegistry
```

## Type Validation Flow

The type checking phase serves as a critical validation layer between parsing and AST transformation:

1. Type Validation
   - Language construct validation
   - State definition type safety
   - Request/response handler compatibility
   - Think block interpolation correctness

2. Error Collection
   - Aggregated error collection across AST
   - Parser error pattern integration
   - Detailed source location tracking
   - Critical error fast-fail support

3. Type Resolution
   - Cross-codebase type reference resolution
   - State variable type inference
   - Generic parameter validation

4. Integration Points
   - Expression evaluation type checking
   - Error reporting system integration
   - Runtime type information provision

## Error Handling Flow

The error handling system follows a structured flow:

1. Collection Phase
   ```rust
   fn check_with_collection(&self, ast: &Root) -> TypeCheckResult<()> {
       let mut collector = ErrorCollector::new();
       
       // Collect errors from all phases
       self.check_types(ast, &mut collector)?;
       self.check_think_blocks(ast, &mut collector)?;
       
       // Process collected errors
       if collector.has_critical_errors() {
           Err(collector.take_critical_errors().into())
       } else if collector.has_errors() {
           Err(collector.take_errors().into())
       } else {
           Ok(())
       }
   }
   ```

2. Error Propagation
   - Error collection in TypeContext
   - Result type propagation
   - Existing error system integration
   - Error recovery support

## Implementation Details

### 1. Type Checking Process
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

## Future Considerations

1. Flow Optimization
   - Parallel type checking
   - Incremental validation
   - Cached type resolution

2. Error Handling Improvements
   - Enhanced error messages
   - Context-aware suggestions
   - Recovery strategies

3. Type System Extensions
   - Generic flow handlers
   - Custom type constraints
   - Advanced state patterns
