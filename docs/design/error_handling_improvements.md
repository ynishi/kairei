# KAIREI Parser Error Handling Improvements

## Problem Statement

The current error handling in the KAIREI parser system has several limitations:

1. **Silent Error Discarding**: The `optional` and `many` combinators silently discard errors, making it difficult to debug parsing issues.
2. **Limited Error Context**: Error messages lack context about where in the parsing process the error occurred.
3. **Incomplete Error Information**: When parsing complex structures like agent definitions, errors in optional or repeated elements are not reported.

These limitations make it challenging for developers to identify and fix issues in their KAIREI DSL code, leading to a poor developer experience.

## Root Cause Analysis

### 1. Parser Combinator Design

The current parser combinator design follows a traditional approach where:

- `optional` returns `None` on failure without preserving error information
- `many` stops collecting items on the first error without reporting it

This design prioritizes simplicity and composability but sacrifices error reporting capabilities.

```rust
// Current implementation (simplified)
impl<I, O, P> Parser<I, Option<O>> for Optional<P, I, O>
where
    P: Parser<I, O>,
{
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<Option<O>> {
        match self.parser.parse(input, pos) {
            Ok((new_pos, value)) => Ok((new_pos, Some(value))),
            Err(_) => Ok((pos, None)), // Error is discarded
        }
    }
}
```

### 2. Error Propagation Gaps

The error propagation chain has gaps, particularly in the AST registry, where errors from the parser are converted to higher-level errors without preserving detailed information.

```rust
// Current error conversion in ast_registry.rs
let (pos, mut root) = analyzer::parsers::world::parse_root()
    .parse(tokens.as_slice(), 0)
    .map_err(|e: analyzer::ParseError| ASTError::ParseError {
        message: format!("failed to parse DSL {}", e),
        target: "root".to_string(),
    })?;
```

### 3. Limited Error Type System

The `ParseError` enum lacks variants for representing errors in optional or repeated parsing contexts, making it difficult to provide meaningful error messages.

## Solution Design

### 1. Error Collection Mechanism

We implemented a thread-local error collector to aggregate errors across different stages of parsing:

```rust
thread_local! {
    pub static ERROR_COLLECTOR: RefCell<ParseErrorCollector> = RefCell::new(ParseErrorCollector::new());
}
```

This collector stores errors along with context information, allowing us to provide more detailed error messages.

### 2. Enhanced Parser Combinators

We created new parser combinators that preserve error information while maintaining the original behavior:

```rust
// Error-collecting optional combinator
pub struct ErrorCollectingOptional<P, I, O> {
    parser: P,
    context: String,
    _phantom: PhantomData<(I, O)>,
}

impl<I, O, P> Parser<I, Option<O>> for ErrorCollectingOptional<P, I, O>
where
    P: Parser<I, O>,
{
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<Option<O>> {
        match self.parser.parse(input, pos) {
            Ok((new_pos, value)) => Ok((new_pos, Some(value))),
            Err(err) => {
                // Store the error in the thread-local collector
                ERROR_COLLECTOR.with(|collector| {
                    let mut collector = collector.borrow_mut();
                    collector.add_error(ParseErrorInfo {
                        error: err,
                        context: self.context.clone(),
                        is_optional: true,
                    });
                });
                // Still return Ok with None, but we've preserved the error
                Ok((pos, None))
            }
        }
    }
}
```

Similarly, we implemented an `ErrorCollectingMany` combinator that collects errors while parsing repeated elements.

### 3. Improved Error Reporting

We enhanced the error reporting in the AST registry to include collected errors:

```rust
// Enhanced error conversion in ast_registry.rs
let parse_result = analyzer::parsers::world::parse_root()
    .parse(tokens.as_slice(), 0)
    .map_err(|e: analyzer::ParseError| {
        // Check if we have collected errors
        let collected_errors = analyzer::error_handling::ERROR_COLLECTOR.with(|collector| {
            let collector = collector.borrow();
            collector.get_errors().to_vec()
        });
        
        if !collected_errors.is_empty() {
            // Create a detailed error message including collected errors
            let detailed_message = analyzer::error_handling::format_detailed_error_message(&e, &collected_errors);
            ASTError::ParseError {
                message: detailed_message,
                target: "root".to_string(),
            }
        } else {
            // Fall back to the original error
            ASTError::ParseError {
                message: format!("failed to parse DSL {}", e),
                target: "root".to_string(),
            }
        }
    });
```

### 4. Extended Error Type System

We extended the `ParseError` enum with new variants to represent errors in optional and repeated parsing contexts:

```rust
pub enum ParseError {
    // ... existing variants ...
    
    /// Error in optional parsing
    #[error("Optional parsing failed in '{context}': {inner}")]
    OptionalFailed {
        /// Context where the error occurred
        context: String,
        /// Inner error
        inner: Box<ParseError>,
    },
    
    /// Error in repeated parsing
    #[error("Repeated parsing failed in '{context}' after {collected_count} items: {inner}")]
    ManyFailed {
        /// Context where the error occurred
        context: String,
        /// Number of items successfully parsed before the error
        collected_count: usize,
        /// Inner error
        inner: Box<ParseError>,
    },
}
```

## Implementation Details

### 1. New Module: `error_handling`

We created a new module `src/analyzer/error_handling/mod.rs` to encapsulate the error collection and reporting functionality:

- `ParseErrorInfo`: Stores information about a parse error, including context and whether it occurred in an optional parsing context.
- `ParseErrorCollector`: Collects and manages parse errors.
- `ERROR_COLLECTOR`: Thread-local storage for the error collector.
- `ErrorCollectingOptional` and `ErrorCollectingMany`: Enhanced parser combinators.
- `format_detailed_error_message`: Formats a detailed error message including collected errors.

### 2. Integration with Existing Code

We integrated the new error handling mechanism with the existing code:

- Updated `src/analyzer/parsers/world.rs` to use the new error-collecting combinators.
- Enhanced `src/ast_registry.rs` to use the collected errors for better error reporting.
- Extended `src/analyzer/core.rs` with new error variants.
- Added the new module to `src/analyzer/mod.rs`.

### 3. Testing

We created comprehensive tests in `tests/error_handling_test.rs` to verify the new error handling functionality:

- Tests for `ErrorCollectingOptional` and `ErrorCollectingMany` combinators.
- Tests for error message formatting.
- Integration tests with the AST registry.

## Benefits

The improved error handling provides several benefits:

1. **Better Developer Experience**: Developers receive more detailed and context-aware error messages, making it easier to identify and fix issues in their KAIREI DSL code.

2. **Preserved Behavior**: The enhanced combinators maintain the same behavior as the original ones, ensuring backward compatibility.

3. **Extensibility**: The error collection mechanism can be extended to other parts of the system, providing a consistent approach to error handling.

4. **Improved Debugging**: The detailed error messages make it easier to debug complex parsing issues, reducing development time.

## Example

Before:
```
failed to parse DSL EOF
```

After:
```
Parse error: EOF

Additional parsing issues:
1. Optional parsing failed in 'world definition': EOF
2. Repeated parsing failed in 'agent definitions': Expected identifier, found Keyword(If)
```

## Future Work

1. **Error Recovery**: Implement error recovery mechanisms to continue parsing after encountering errors.

2. **Source Location Information**: Enhance error messages with source location information (file, line, column).

3. **Visual Error Reporting**: Integrate with an IDE or editor to provide visual error reporting.

4. **Error Categorization**: Categorize errors to provide more targeted error messages and suggestions.

5. **Error Suggestions**: Provide suggestions for fixing common errors.

## Conclusion

The improved error handling in the KAIREI parser system significantly enhances the developer experience by providing more detailed and context-aware error messages. By preserving error information in optional and repeated parsing contexts, we enable developers to more easily identify and fix issues in their KAIREI DSL code.
