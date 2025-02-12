# KAIREI Type Checker Design

## 1. Architecture Design

### 1.1 Role and Responsibilities

The TypeCheck phase serves as a critical validation layer between the Parser and AST phases in the KAIREI DSL compilation pipeline. Its primary responsibilities include:

1. Type Validation
   - Validate type correctness of all language constructs
   - Ensure type safety of state definitions and their initial values
   - Verify type compatibility in request/response handlers
   - Check type correctness in think block interpolations
   - Validate plugin API type contracts

2. Error Collection
   - Collect and aggregate type errors across the entire AST
   - Follow existing error collection pattern used in Parser
   - Provide detailed error messages with source locations
   - Support early exit on critical errors while collecting non-critical ones

3. Type Resolution
   - Resolve type references across the codebase
   - Handle type inference for state variables
   - Validate generic type parameters
   - Ensure plugin API type compatibility

4. Integration Points
   - Hook into expression evaluation for type checking
   - Integrate with existing error reporting mechanisms
   - Provide type information for the runtime system
   - Support plugin system type validation

### 1.2 Component Architecture

The type checker is designed with a modular architecture that follows existing patterns in the codebase:

1. Core Components
   ```rust
   // Core type checker trait
   pub trait TypeChecker {
       fn check_types(&self, ast: &Root) -> TypeCheckResult<()>;
       fn collect_errors(&self) -> Vec<TypeCheckError>;
   }

   // Type validation context
   pub struct TypeContext {
       errors: Vec<TypeCheckError>,
       scope: TypeScope,
       plugins: Arc<DashMap<String, PluginTypeInfo>>,
   }

   // Error types
   pub enum TypeCheckError {
       TypeMismatch { expected: TypeInfo, found: TypeInfo, location: Location },
       UndefinedType { name: String, location: Location },
       InvalidTypeArguments { message: String, location: Location },
       PluginTypeError { plugin: String, message: String, location: Location },
   }
   ```

2. Visitor Components
   - AST traversal using visitor pattern
   - Separate visitors for different language constructs
   - Type validation rules implemented in visitors
   - Error collection during traversal

3. Type Resolution System
   - Type scope management
   - Generic type parameter resolution
   - Plugin type information resolution
   - Type inference engine

4. Error Reporting System
   - Error collection and aggregation
   - Source location tracking
   - Detailed error message formatting
   - Integration with existing error system

### 1.3 Interface Definitions

The type checker defines clear interfaces for integration with other components:

1. Type Checking Interface
   ```rust
   pub trait TypeCheck {
       // Main entry point for type checking
       fn check_types(&self, ast: &Root) -> TypeCheckResult<()>;
       
       // Type checking for specific constructs
       fn check_micro_agent(&self, agent: &MicroAgentDef) -> TypeCheckResult<()>;
       fn check_state(&self, state: &StateDef) -> TypeCheckResult<()>;
       fn check_handler(&self, handler: &HandlerDef) -> TypeCheckResult<()>;
       fn check_think(&self, think: &Expression) -> TypeCheckResult<()>;
       
       // Error handling
       fn collect_errors(&self) -> Vec<TypeCheckError>;
   }
   ```

2. Error Reporting Interface
   ```rust
   pub trait TypeErrorReporter {
       fn report_error(&self, error: TypeCheckError);
       fn has_errors(&self) -> bool;
       fn error_count(&self) -> usize;
       fn format_errors(&self) -> String;
   }
   ```

3. Plugin Validation Interface
   ```rust
   pub trait PluginTypeValidator {
       // Validate plugin type contracts
       fn validate_plugin_types(&self, plugin: &ProviderPlugin) -> TypeCheckResult<()>;
       
       // Check plugin-specific type rules
       fn check_plugin_request(&self, request: &ProviderRequest) -> TypeCheckResult<()>;
       fn check_plugin_response(&self, response: &ProviderResponse) -> TypeCheckResult<()>;
   }
   ```

4. Type Resolution Interface
   ```rust
   pub trait TypeResolver {
       fn resolve_type(&self, type_info: &TypeInfo) -> TypeCheckResult<ResolvedType>;
       fn resolve_generic(&self, type_params: &[TypeInfo]) -> TypeCheckResult<ResolvedType>;
       fn infer_type(&self, expr: &Expression) -> TypeCheckResult<TypeInfo>;
   }
   ```

### 1.4 Error Reporting Mechanism

The error reporting mechanism follows the existing error handling patterns in the codebase:

1. Error Collection Strategy
   - Collect all type errors during validation
   - Support both fail-fast and collect-all modes
   - Track error locations and contexts
   - Prioritize errors based on severity

2. Error Types and Structure
   ```rust
   #[derive(Debug)]
   pub enum TypeCheckError {
       // Type system errors
       TypeMismatch {
           expected: TypeInfo,
           found: TypeInfo,
           location: Location,
       },
       UndefinedType {
           name: String,
           location: Location,
       },
       
       // Generic type errors
       InvalidTypeArguments {
           message: String,
           location: Location,
       },
       
       // Plugin-specific errors
       PluginTypeError {
           plugin: String,
           message: String,
           location: Location,
       },
       
       // Think block errors
       ThinkBlockError {
           message: String,
           location: Location,
       },
   }
   ```

3. Error Message Format
   ```
   Error[E0001]: Type mismatch
    --> file.kai:10:5
     |
   10 |     count: String = 42
     |     ^^^^^ expected String, found Integer
     |
     = help: consider using .to_string() to convert the integer to a string
   ```

4. Error Propagation Pattern
   - Errors are collected in TypeContext
   - Propagated up through Result types
   - Integrated with existing error system
   - Support for error recovery and continuation

## 2. Type System Specification

### 2.1 Type Hierarchy

The KAIREI type system is built on a hierarchical structure that supports both built-in and user-defined types:

1. Built-in Types
   ```rust
   // Primitive Types
   Int     // 64-bit signed integer
   String  // UTF-8 string
   Boolean // true/false
   Float   // 64-bit floating point
   Unit    // () - used for statements with no return value
   
   // Special Types
   Duration // Time duration
   Delay    // Retry delay specification
   ```

2. Generic Types
   ```rust
   // Container Types
   Result<T, E>  // Success type T or error type E
   Option<T>     // Some value of type T or None
   Array<T>      // List of elements of type T
   Map<K, V>     // Key-value mapping from K to V
   
   // Common Usage Examples
   Result<Int, Error>     // Integer result that may fail
   Option<String>        // Optional string value
   Array<Result<T, E>>   // List of results
   ```

3. Custom Types
   ```rust
   // User-defined structured types
   type UserProfile {
       name: String,
       age: Int,
       preferences: Map<String, String>
   }
   
   // Custom event types
   type LoginEvent {
       user_id: String,
       timestamp: Duration
   }
   ```

4. Plugin API Types
   ```rust
   // Plugin interface types
   type PluginRequest {
       method: String,
       parameters: Map<String, Value>
   }
   
   type PluginResponse {
       status: Int,
       data: Result<Value, Error>
   }
   ```

5. Type Relationships
   ```
   Value
   ├── Primitive Types
   │   ├── Int
   │   ├── String
   │   ├── Boolean
   │   └── Float
   ├── Container Types
   │   ├── Result<T, E>
   │   ├── Option<T>
   │   ├── Array<T>
   │   └── Map<K, V>
   ├── Special Types
   │   ├── Duration
   │   └── Delay
   └── Custom Types
       └── User-defined structures
   ```

### 2.2 Type Checking Rules

The type checker enforces the following rules:

1. State Variable Rules
   ```rust
   // Rule 1: Explicit type annotations must match initial values
   state {
       count: Int = 0,        // Valid: Int matches integer literal
       name: String = 42      // Error: Type mismatch
   }
   
   // Rule 2: Type inference from initial values
   state {
       count = 0,             // Inferred as Int
       name = "Alice"         // Inferred as String
   }
   
   // Rule 3: Generic type parameters must be valid
   state {
       results: Array<Int> = [],     // Valid: concrete type parameter
       data: Result<T, E>            // Error: unbound type parameters
   }
   ```

2. Request/Response Rules
   ```rust
   // Rule 1: Request parameter types must be concrete
   on request Process(
       input: String,              // Valid: concrete type
       config: Map<String, Value>  // Valid: concrete type parameters
   )
   
   // Rule 2: Return types must match handler block
   on request Calculate(x: Int) -> Result<Int, Error> {
       Ok(x + 1)     // Valid: matches return type
       x + 1         // Error: must be wrapped in Ok()
   }
   
   // Rule 3: Error types must be compatible
   on request Fetch() -> Result<String, Error> {
       Err("failed") // Valid: String can be error message
       Err(404)      // Error: Int cannot be error message
   }
   ```

3. Think Block Rules
   ```rust
   // Rule 1: String interpolation types must be stringifiable
   think("Count is ${count}")     // Valid if count is Int/String/etc
   think("Data is ${complex}")    // Error if complex lacks Display
   
   // Rule 2: Plugin configurations must match schema
   think("Query") with {
       model: "gpt-4",           // Valid: string config
       temperature: "high"       // Error: expected float
   }
   ```

4. Binary Operation Rules
   ```rust
   // Rule 1: Arithmetic operations require numeric types
   Int + Int       // Valid
   String + Int    // Error
   
   // Rule 2: Comparison operations must have compatible types
   Int == Int      // Valid
   Int < String    // Error
   
   // Rule 3: Logical operations require boolean operands
   Boolean && Boolean  // Valid
   Int || Boolean     // Error
   ```

5. Plugin API Rules
   ```rust
   // Rule 1: Plugin requests must match declared schema
   plugin.execute({
       method: "search",         // Valid: matches schema
       params: {                 // Valid: matches parameter types
           query: "test",
           limit: 10
       }
   })
   
   // Rule 2: Plugin responses must be handled appropriately
   match plugin.response {
       Ok(data: Value) => {},    // Valid: handles success case
       Err(e: Error) => {}       // Valid: handles error case
   }
   ```

### 2.3 Error Cases

The type checker handles the following error scenarios:

1. Type Mismatch Errors
   ```rust
   // Case 1: Incompatible assignment
   let x: String = 42;
   // Error: Cannot assign Int to String
   // Help: Consider using .to_string() to convert Int to String
   
   // Case 2: Invalid function arguments
   calculate(x: String, y: Int)
   calculate("hello", "world")
   // Error: Expected Int for parameter 'y', found String
   // Help: The second argument must be a number
   ```

2. Generic Type Errors
   ```rust
   // Case 1: Invalid type parameters
   let data: Result<T>;
   // Error: Type parameter T is not bound
   // Help: Specify a concrete type for T
   
   // Case 2: Mismatched generic arguments
   let nums: Array<Int> = ["1", "2"];
   // Error: Cannot assign Array<String> to Array<Int>
   // Help: Consider parsing strings to integers
   ```

3. Plugin API Errors
   ```rust
   // Case 1: Invalid plugin configuration
   plugin.configure({
       model: 123,  // should be string
       temp: "hot"  // should be float
   })
   // Error: Invalid type for plugin config 'model'
   // Help: 'model' must be a string
   
   // Case 2: Invalid response handling
   plugin.execute()
       .map(|x: String| x.len())
   // Error: Plugin response type mismatch
   // Help: Plugin returns Value, not String
   ```

4. Think Block Errors
   ```rust
   // Case 1: Invalid interpolation
   think("Data: ${complex_obj}")
   // Error: Type ${complex_obj} cannot be interpolated
   // Help: Implement Display for ComplexObj
   
   // Case 2: Invalid attribute types
   think("Query") with {
       temperature: "warm"
   }
   // Error: Invalid temperature value
   // Help: temperature must be a float between 0 and 1
   ```

5. State Definition Errors
   ```rust
   // Case 1: Invalid initial values
   state {
       count: Int = "zero"
   }
   // Error: Cannot initialize Int with String
   // Help: Use a numeric value for count
   
   // Case 2: Invalid type references
   state {
       data: InvalidType = 42
   }
   // Error: Unknown type 'InvalidType'
   // Help: Use a valid type like Int, String, etc.
   ```

6. Request Handler Errors
   ```rust
   // Case 1: Return type mismatch
   on request Process() -> Result<Int> {
       "not a number"
   }
   // Error: Return type mismatch
   // Help: Return Ok(number) or Err(error)
   
   // Case 2: Invalid error handling
   on request Fetch() -> Result<String> {
       if error {
           return 404  // wrong error type
       }
   }
   // Error: Invalid error type
   // Help: Wrap error in Err(error.to_string())
   ```

### 2.4 Type Inference

The type system supports limited type inference with clear rules and limitations:

1. State Variable Inference
   ```rust
   state {
       // Basic literal inference
       count = 0            // Inferred as Int
       name = "Alice"       // Inferred as String
       active = true        // Inferred as Boolean
       
       // Container type inference
       list = [1, 2, 3]    // Inferred as Array<Int>
       map = {             // Inferred as Map<String, String>
           "key": "value"
       }
   }
   ```

2. Expression Type Inference
   ```rust
   // Binary operation inference
   let sum = x + y         // Inferred from operand types
   let concat = s1 + s2    // String if both are strings
   
   // Function return inference
   fn process() {
       if condition {
           return 42       // Infers return type as Int
       }
       return 0
   }
   ```

3. Generic Type Inference
   ```rust
   // Container type inference
   let opt = Some(42)      // Option<Int>
   let res = Ok("success") // Result<String, _>
   
   // Map/Array inference
   let items = []          // Error: Cannot infer element type
   let items: Array<Int> = [] // Valid: Type explicitly specified
   ```

4. Inference Limitations
   ```rust
   // Case 1: Ambiguous types
   let x = None           // Error: Cannot infer Option type
   let x: Option<Int> = None  // Valid: Type explicitly specified
   
   // Case 2: Complex expressions
   let result = if cond {
       "string"
   } else {
       42
   }
   // Error: Conflicting types in branches
   
   // Case 3: Plugin interactions
   plugin.execute(data)   // Error: Cannot infer plugin types
   plugin.execute(data: RequestType)  // Valid: Type specified
   ```

5. Type Inference Rules
   - Inference flows from values to variables
   - Explicit types take precedence over inference
   - Generic types require sufficient context
   - Plugin API types must be explicit
   - Think block interpolation types must be explicit

## 3. Implementation Guide

### 3.1 Core Module Design
- TypeChecker implementation
- Visitor pattern for AST traversal
- Error collection implementation

### 3.2 Class/Interface Definitions
- TypeValidator trait
- ErrorCollector interface
- PluginTypeValidator interface

### 3.3 Error Handling Strategy
- Error collection pattern
- Error message formatting
- Error propagation flow

### 3.4 Performance Requirements
- Single-pass validation
- Error collection optimization
- Memory usage considerations
