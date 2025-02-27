# KAIREI DSL Syntax Reference

## Introduction

KAIREI DSL (Domain Specific Language) is a purpose-built language for defining AI agent systems. It provides a structured, type-safe, and intuitive syntax for constructing multi-agent AI systems with event-driven architecture. This reference document covers the complete syntax, usage patterns, and best practices for KAIREI DSL.

The DSL consists of two primary components:

1. **World DSL**: Defines the environment and global context where agents operate
2. **MicroAgent DSL**: Defines individual agents, their state, and behavior

## Table of Contents

- [KAIREI DSL Syntax Reference](#kairei-dsl-syntax-reference)
  - [Introduction](#introduction)
  - [Table of Contents](#table-of-contents)
  - [World Definition](#world-definition)
    - [World Declaration](#world-declaration)
    - [Policy Definition](#policy-definition)
    - [Configuration Block](#configuration-block)
    - [Events Block](#events-block)
    - [Handlers Block](#handlers-block)
  - [MicroAgent Definition](#microagent-definition)
    - [MicroAgent Declaration](#microagent-declaration)
    - [Policy Definition](#policy-definition-1)
    - [Lifecycle Block](#lifecycle-block)
    - [State Block](#state-block)
    - [Observe Block](#observe-block)
    - [Answer Block](#answer-block)
    - [React Block](#react-block)
  - [Common Syntax Elements](#common-syntax-elements)
    - [Identifiers](#identifiers)
    - [Literals](#literals)
    - [Type Annotations](#type-annotations)
    - [Expressions](#expressions)
    - [Statements](#statements)
  - [Type System](#type-system)
    - [Built-in Types](#built-in-types)
    - [Complex Types](#complex-types)
    - [Custom Types](#custom-types)
  - [Think Expression](#think-expression)
    - [Basic Syntax](#basic-syntax)
    - [Think Attributes](#think-attributes)
    - [Plugin Integration](#plugin-integration)
  - [Error Handling](#error-handling)
    - [Result Type](#result-type)
    - [Error Propagation](#error-propagation)
    - [On-Fail Handling](#on-fail-handling)
  - [Best Practices](#best-practices)
    - [Naming Conventions](#naming-conventions)
    - [State Management](#state-management)
    - [Event Design](#event-design)
    - [Error Handling](#error-handling-1)
  - [Cheat Sheet](#cheat-sheet)

## World Definition

The World DSL defines the environment where MicroAgents operate. It includes global configuration, event definitions, and handlers that apply system-wide.

### World Declaration

```kairei
world WorldName {
    // World contents
}
```

**Example**:

```kairei
world TravelPlanningWorld {
    // World contents
}
```

### Policy Definition

Policies define high-level guidelines for the World and its agents. They are expressed in natural language and are interpreted during processing.

```kairei
world WorldName {
    policy "Policy statement in natural language"
    // Additional policies
}
```

**Example**:

```kairei
world TravelPlanningWorld {
    policy "Ensure data freshness within 24 hours"
    policy "Verify travel availability before confirmation"
    policy "Maintain user privacy standards"
}
```

### Configuration Block

The config block defines global settings for the World.

```kairei
world WorldName {
    config {
        tick_interval: Duration = "1s"
        max_agents: Int = 100
        event_buffer_size: Int = 500
        // Additional configuration
    }
}
```

**Parameters**:

- `tick_interval`: Duration between tick events (e.g., "1s", "100ms", "1m")
- `max_agents`: Maximum number of agents allowed in the World
- `event_buffer_size`: Size of the event queue buffer

### Events Block

The events block defines custom events that can be emitted and handled within the World.

```kairei
world WorldName {
    events {
        EventName(param1: Type1, param2: Type2)
        SimpleEvent
        // Additional events
    }
}
```

**Example**:

```kairei
world TravelPlanningWorld {
    events {
        UserRequestedItinerary(user_id: String)
        TripStarted
        LocationUpdated(latitude: Float, longitude: Float)
    }
}
```

### Handlers Block

The handlers block defines how the World responds to events.

```kairei
world WorldName {
    handlers {
        on EventName(param1: Type1, param2: Type2) {
            // Event handling statements
        }
    }
}
```

**Example**:

```kairei
world TravelPlanningWorld {
    handlers {
        on Tick(delta_time: Float) {
            emit NextTick(delta_time)
        }
        
        on TravellerJoined(user_id: String) {
            // Handle traveller joining
        }
    }
}
```

## MicroAgent Definition

MicroAgents are the autonomous entities in KAIREI that encapsulate state and behavior. They observe events, respond to requests, and take actions.

### MicroAgent Declaration

```kairei
micro AgentName {
    // Agent contents
}
```

**Example**:

```kairei
micro TravelAgent {
    // Agent contents
}
```

### Policy Definition

Similar to World policies, agent policies define high-level guidelines for the specific agent.

```kairei
micro AgentName {
    policy "Policy statement in natural language"
    // Additional policies
}
```

**Example**:

```kairei
micro TravelAgent {
    policy "Prioritize user preferences when suggesting itineraries"
    policy "Consider budget constraints in all recommendations"
}
```

### Lifecycle Block

The lifecycle block defines handlers for agent initialization and cleanup.

```kairei
micro AgentName {
    lifecycle {
        on_init {
            // Initialization code
        }
        
        on_destroy {
            // Cleanup code
        }
    }
}
```

**Example**:

```kairei
micro TravelAgent {
    lifecycle {
        on_init {
            self.active_trips = 0
            emit AgentReady(self.name)
        }
        
        on_destroy {
            emit AgentShutdown(self.name)
        }
    }
}
```

### State Block

The state block defines internal state variables for the agent.

```kairei
micro AgentName {
    state {
        variable_name: Type = initial_value;
        another_variable: Type;  // No initial value
    }
}
```

**Example**:

```kairei
micro TravelAgent {
    state {
        active_trips: Int = 0;
        user_preferences: Map<String, String> = {};
        last_update: DateTime;
    }
}
```

### Observe Block

The observe block defines handlers for monitoring events. Handlers in this block can modify agent state.

```kairei
micro AgentName {
    observe {
        on EventName(param1: Type1, param2: Type2) {
            // Event handling statements
        }
    }
}
```

**Example**:

```kairei
micro TravelAgent {
    observe {
        on Tick {
            self.check_trip_updates()
        }
        
        on UserPreferenceChanged(user_id: String, preferences: Map<String, String>) {
            self.user_preferences = preferences
        }
    }
}
```

### Answer Block

The answer block defines handlers for responding to requests. These handlers have read-only access to state and must return a Result.

```kairei
micro AgentName {
    answer {
        on request RequestName(param1: Type1) -> Result<ResponseType, Error> {
            // Request handling with return
        }
    }
}
```

**Example**:

```kairei
micro TravelAgent {
    answer {
        on request GetItinerary(user_id: String, destination: String) -> Result<Itinerary, Error> {
            with {
                model: "gpt-4"
                temperature: 0.7
            }
            
            // Generate itinerary
            let itinerary = think("Generate a travel itinerary to ${destination} based on user preferences ${self.user_preferences[user_id]}")
            
            return Ok(itinerary)
        }
    }
}
```

### React Block

The react block defines handlers for implementing proactive behaviors in response to events. Handlers in this block can modify agent state.

```kairei
micro AgentName {
    react {
        on EventName(param1: Type1) {
            // Proactive behavior implementation
        }
    }
}
```

**Example**:

```kairei
micro TravelAgent {
    react {
        on WeatherAlert(destination: String, alert_type: String) {
            // Update affected itineraries
            self.update_affected_itineraries(destination, alert_type)
            
            // Notify users with trips to the affected destination
            emit UserNotification(destination, alert_type)
        }
    }
}
```

## Common Syntax Elements

### Identifiers

Identifiers in KAIREI follow these rules:

- Must start with a letter or underscore
- Can contain letters, numbers, and underscores
- Are case-sensitive
- Cannot be reserved keywords

**Valid identifiers**:
```
user_id
TravelAgent
count123
_internalVariable
```

### Literals

KAIREI supports the following literal types:

1. **Integer**:
   ```kairei
   42
   -7
   1000000
   ```

2. **Float**:
   ```kairei
   3.14
   -0.5
   1.0e6
   ```

3. **String**:
   ```kairei
   "Hello, world!"
   "Line 1\nLine 2"
   "${variable} interpolation"
   ```

4. **Boolean**:
   ```kairei
   true
   false
   ```

5. **Duration**:
   ```kairei
   "1s"    // 1 second
   "500ms" // 500 milliseconds
   "2m"    // 2 minutes
   "1h"    // 1 hour
   ```

6. **List**:
   ```kairei
   [1, 2, 3]
   ["a", "b", "c"]
   []  // Empty list
   ```

7. **Map**:
   ```kairei
   {"key1": "value1", "key2": "value2"}
   {"name": "Alice", "age": 30}
   {}  // Empty map
   ```

8. **Null**:
   ```kairei
   null
   ```

### Type Annotations

Type annotations follow identifiers, separated by a colon:

```kairei
variable_name: Type
```

**Examples**:

```kairei
user_id: String
count: Int
is_active: Boolean
items: List<String>
user_data: Map<String, Any>
result: Result<Int, Error>
```

### Expressions

Expressions in KAIREI include:

1. **Variable References**:
   ```kairei
   variable_name
   ```

2. **State Access**:
   ```kairei
   self.variable_name
   ```

3. **Function Calls**:
   ```kairei
   function_name(arg1, arg2)
   ```

4. **Binary Operations**:
   ```kairei
   a + b
   x * y
   flag && condition
   value == expected
   ```

5. **Think Expressions** (LLM integration):
   ```kairei
   think("Generate content based on ${input}")
   ```

6. **Request Expressions**:
   ```kairei
   request agent.RequestName(param1, param2)
   ```

7. **Result Construction**:
   ```kairei
   Ok(value)
   Err(error)
   ```

### Statements

KAIREI supports the following statement types:

1. **Assignment**:
   ```kairei
   variable = expression
   self.state_var = new_value
   ```

2. **If Statement**:
   ```kairei
   if condition {
       // Then statements
   } else {
       // Else statements
   }
   ```

3. **Return Statement**:
   ```kairei
   return expression
   ```

4. **Emit Statement**:
   ```kairei
   emit EventName(param1, param2)
   ```

5. **Block Statement**:
   ```kairei
   {
       statement1
       statement2
   }
   ```

6. **Error Handling**:
   ```kairei
   statement on_fail {
       // Error handling code
   }
   ```

## Type System

KAIREI implements a static type system that ensures type safety across the DSL.

### Built-in Types

- `String`: Text values
- `Int`: Integer numbers (i64)
- `Float`: Floating point numbers (f64)
- `Boolean`: True/false values
- `Duration`: Time intervals
- `Date`: Calendar dates
- `DateTime`: Date and time values
- `Json`: JSON data
- `Any`: Dynamic type for flexibility

### Complex Types

- `List<T>`: Lists of type T
  ```kairei
  user_ids: List<String> = ["user1", "user2"]
  ```

- `Map<K, V>`: Key-value maps
  ```kairei
  user_data: Map<String, Any> = {"name": "Alice", "age": 30}
  ```

- `Result<T, E>`: Success/failure results
  ```kairei
  result: Result<Int, Error> = Ok(42)
  error_result: Result<String, Error> = Err("Something went wrong")
  ```

- `Option<T>`: Optional values
  ```kairei
  maybe_value: Option<String> = Some("value")
  no_value: Option<Int> = None
  ```

### Custom Types

KAIREI allows defining custom types for complex data structures:

```kairei
type UserProfile {
    id: String
    name: String
    age: Int
    email: String
    preferences: Map<String, String>
}
```

Usage:

```kairei
user: UserProfile = {
    id: "user123",
    name: "Alice",
    age: 30,
    email: "alice@example.com",
    preferences: {"theme": "dark", "notifications": "enabled"}
}
```

## Think Expression

The `think` expression is a core feature of KAIREI that integrates with Language Models (LLMs) for generating content.

### Basic Syntax

```kairei
think("Prompt with ${variable} interpolation")
```

**Example**:

```kairei
let summary = think("Summarize the following text: ${text}")
```

### Think Attributes

The `think` expression can be customized with attributes:

```kairei
think("Prompt text") with {
    provider: "provider_name"
    model: "model_name"
    temperature: 0.7
    max_tokens: 100
    // Additional attributes
}
```

**Attributes**:

- `provider`: The LLM provider to use (defaults to system default)
- `model`: The specific model to use
- `temperature`: Controls randomness (0.0 - 1.0)
- `max_tokens`: Maximum number of tokens to generate
- `policies`: Array of policy strings to guide generation

**Example**:

```kairei
let itinerary = think("Generate a travel itinerary for ${destination} considering the budget of ${budget}") with {
    model: "gpt-4"
    temperature: 0.5
    max_tokens: 2000
    policies: [
        "Focus on budget-friendly options",
        "Include local experiences"
    ]
}
```

### Plugin Integration

The `think` expression can be extended with plugins:

```kairei
think("Prompt text") with {
    // Plugin configuration
    plugins: {
        "memory": {
            "ttl": 3600
        },
        "rag": {
            "chunk_size": 512,
            "max_tokens": 1000
        }
    }
}
```

**Example**:

```kairei
let answer = think("Answer the question: ${question}") with {
    plugins: {
        "memory": {
            "ttl": 3600,
            "key": "user-${user_id}"
        },
        "web_search": {
            "max_results": 5
        }
    }
}
```

## Error Handling

KAIREI provides robust error handling mechanisms.

### Result Type

The `Result<T, E>` type is used for operations that might fail:

```kairei
on request GetData() -> Result<Data, Error> {
    // On success
    return Ok(data)
    
    // On failure
    return Err("Error message")
}
```

### Error Propagation

The `?` operator can be used for error propagation:

```kairei
on request ProcessData() -> Result<ProcessedData, Error> {
    let data = request otherAgent.GetData()?
    // Continues if GetData() returned Ok, otherwise returns the error
    
    return Ok(process(data))
}
```

### On-Fail Handling

The `on_fail` syntax provides a way to handle errors inline:

```kairei
statement on_fail {
    // Error handling code
    emit ErrorOccurred(error)
}
```

**Example**:

```kairei
request dataAgent.FetchData() on_fail {
    emit DataFetchFailed("Could not fetch data")
    return Err("Data fetch failed")
}
```

## Best Practices

### Naming Conventions

- Use `PascalCase` for agent names, event types, and custom types
- Use `camelCase` for variables, functions, and parameters
- Use `snake_case` for internal implementation details

### State Management

- Keep state minimal and focused
- Avoid shared mutable state between agents
- Use events for communication and state updates
- Initialize state variables with sensible defaults

### Event Design

- Design events as notifications, not commands
- Use meaningful names that describe what happened
- Include relevant data as parameters
- Keep events focused on a single concern

### Error Handling

- Use appropriate error types for different failure modes
- Provide meaningful error messages
- Handle errors at the appropriate level
- Use the `on_fail` syntax for localized error handling

## Cheat Sheet

### World Syntax

```kairei
world WorldName {
    policy "Policy statement"
    
    config {
        tick_interval: Duration = "1s"
        max_agents: Int = 100
        event_buffer_size: Int = 500
    }
    
    events {
        CustomEvent(param: Type)
    }
    
    handlers {
        on EventName(param: Type) {
            // Handler code
        }
    }
}
```

### MicroAgent Syntax

```kairei
micro AgentName {
    policy "Policy statement"
    
    lifecycle {
        on_init {
            // Initialization
        }
        
        on_destroy {
            // Cleanup
        }
    }
    
    state {
        variable: Type = initial_value;
    }
    
    observe {
        on EventName(param: Type) {
            // Event observation
        }
    }
    
    answer {
        on request RequestName(param: Type) -> Result<ResponseType, Error> {
            // Request handling
            return Ok(result)
        }
    }
    
    react {
        on EventName(param: Type) {
            // Proactive behavior
        }
    }
}
```

### Think Expression

```kairei
think("Prompt text") with {
    model: "model_name"
    temperature: 0.7
    plugins: {
        "plugin_name": {
            "option": value
        }
    }
}
```

### Error Handling

```kairei
// Result construction
Ok(value)
Err("error message")

// Error propagation
let result = risky_operation()?

// On-fail handling
statement on_fail {
    // Error handling
}
```