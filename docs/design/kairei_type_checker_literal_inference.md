# Type Checker - Literal Type Inference

## Overview

This document describes the implementation of literal type inference in the KAIREI type checker. The implementation focuses on providing robust type checking for literals, including containers like lists and maps.

## Implementation Details

### 1. Expression Type Checker

The core of the literal type inference is implemented through the `ExpressionTypeChecker` trait:

```rust
pub(crate) trait ExpressionTypeChecker {
    fn infer_literal_type(&self, lit: &Literal, ctx: &TypeContext) -> TypeCheckResult<TypeInfo>;
    fn infer_binary_op_type(
        &self,
        left: &TypeInfo,
        right: &TypeInfo,
        op: &BinaryOperator,
    ) -> TypeCheckResult<TypeInfo>;
}
```

### 2. Type Inference Rules

#### Basic Literals
- Integer -> Int
- Float -> Float
- String -> String
- Boolean -> Boolean
- Duration -> Duration
- Null -> Null

#### Container Types
1. Lists
   - Must have at least one element for type inference
   - All elements must have the same type
   - Results in Array<T> where T is the element type

2. Maps
   - Must have at least one entry for type inference
   - All keys must be strings
   - All values must have the same type
   - Results in Map<String, T> where T is the value type

### 3. Error Handling

The implementation provides detailed error messages for:
- Empty containers that cannot be type-inferred
- Mixed types in lists
- Mixed value types in maps
- Unsupported literal types

Example error messages:
```rust
"Cannot infer type of empty list"
"List contains mixed types: found both Int and String"
"Map contains mixed value types: found both Int and String"
```

### 4. Testing

The implementation includes comprehensive tests:
- Basic literal type inference
- List type inference with consistent types
- List type inference with mixed types (error case)
- Map type inference with consistent value types
- Map type inference with mixed value types (error case)
- Empty container error cases

## Future Considerations

1. Performance Optimizations
   - Consider caching inferred types
   - Optimize container type checking for large collections

2. Feature Extensions
   - Support for user-defined literal types
   - More flexible container type rules
   - Type coercion rules

3. Error Reporting
   - Add source location information
   - Provide more detailed error messages
   - Include suggestions for fixing type errors
