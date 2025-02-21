# Type Checker - Function Call Type Checking

## Overview

This document describes the implementation of function call type checking in the KAIREI type checker. The implementation focuses on validating function calls, their arguments, and return types.

## Implementation Details

### 1. Function Type Checker

The core functionality is implemented through the `FunctionTypeChecker` trait:

```rust
pub(crate) trait FunctionTypeChecker {
    fn check_function_call(
        &self,
        function: &str,
        arguments: &[Expression],
        ctx: &TypeContext,
    ) -> TypeCheckResult<TypeInfo>;
}
```

### 2. Type Checking Process

1. Function Signature Resolution
```rust
fn get_function_signature(&self, function: &str, ctx: &TypeContext) -> TypeCheckResult<TypeInfo>
```
- Looks up function type in the scope
- Returns error for undefined functions
- Validates function type signature format

2. Parameter Type Extraction
```rust
fn extract_parameter_types(&self, type_info: &TypeInfo) -> TypeCheckResult<(Vec<TypeInfo>, TypeInfo)>
```
- Extracts expected parameter types from function signature
- Extracts return type information
- Validates function type structure

3. Argument Type Checking
```rust
fn check_argument_types(
    &self,
    function: &str,
    arguments: &[Expression],
    expected_types: &[TypeInfo],
    ctx: &TypeContext,
) -> TypeCheckResult<()>
```
- Validates argument count matches parameter count
- Checks each argument's type against expected parameter type
- Reports detailed errors for mismatches

### 3. Error Handling

The implementation provides specific error types for:
- Undefined functions
- Wrong number of arguments
- Type mismatches in arguments
- Invalid function signatures

Example error messages:
```rust
"Function test_func requires 2 arguments, but 1 was provided"
"Invalid argument type for function test_func: expected Int, found String"
"Undefined function: unknown_func"
```

### 4. Testing

The implementation includes tests for:
- Basic function call validation
- Argument type checking
- Error cases:
  - Undefined functions
  - Wrong number of arguments
  - Invalid argument types

## Current Limitations

1. Function Types
   - Currently assumes Result<T, Error> return type
   - Limited support for complex parameter types
   - No support for generic functions

2. Type Checking
   - Only literal arguments are fully supported
   - Limited type inference for complex expressions
   - No support for function overloading

## Future Considerations

1. Type System Extensions
   - Support for generic functions
   - Function overloading
   - Named arguments
   - Optional parameters

2. Error Reporting
   - More detailed error messages
   - Suggestions for fixing type errors
   - Better location information in errors

3. Performance
   - Function signature caching
   - Optimized type checking for common cases
