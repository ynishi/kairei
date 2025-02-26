# KAIREI Quick Reference Guide

This guide provides a quick overview of the KAIREI AI Agent Orchestration Platform, designed to help developers rapidly understand the system's key components, architecture, and usage patterns.

## Documentation Strategy

KAIREI follows a layered documentation approach to minimize cognitive load while providing comprehensive information:

1. **Quick Reference Docs** (You are here)
   - Essential concepts, patterns, and relationships
   - High-level overview for rapid onboarding
   - Usage examples and common development tasks
   - Entry point for new developers

2. **Architecture in lib.rs**
   - High-level system architecture documentation
   - Cross-module relationships and pipeline flows
   - Core design principles and architectural patterns
   - Available through `cargo doc` or source code

3. **Detailed Implementation in mod.rs files**
   - Implementation details in module-level RustDocs
   - Focused on "how" rather than "why"
   - Specific to each module's functionality
   - Technical documentation for active development

This layered approach allows developers to start with quick concepts, understand the architecture, and then dive into implementation details as needed.

## What is KAIREI?

KAIREI is an AI Agent Orchestration Platform leveraging LLMs. It provides a flexible and scalable development and execution environment for AI agents using an intuitive DSL (Domain Specific Language) and event-driven architecture.

## System Architecture

KAIREI follows a layered architecture with clear separation of concerns:

```
┌─────────────────────────────────────────────┐
│                 User Layer                  │
│  (DSL Code, CLI Interface, Applications)    │
└───────────────────┬─────────────────────────┘
                    │
┌───────────────────▼─────────────────────────┐
│               Processing Pipeline           │
│  ┌─────────┐  ┌───────────┐  ┌───────────┐  │
│  │ Lexical │  │ Syntactic │  │ Semantic  │  │
│  │ Analysis│→ │ Analysis  │→ │ Analysis  │  │
│  │(Tokenizer) │ (Parser)  │  │(TypeCheck)│  │
│  └─────────┘  └───────────┘  └───────────┘  │
└───────────────────┬─────────────────────────┘
                    │
┌───────────────────▼─────────────────────────┐
│              Runtime Environment            │
│ ┌─────────────┐  ┌─────────────┐            │
│ │Event System │  │    AST      │            │
│ │   Bus       │←→│  Evaluator  │            │
│ └─────────────┘  └─────────────┘            │
└───────────────────┬─────────────────────────┘
                    │
┌───────────────────▼─────────────────────────┐
│              Provider Layer                 │
│  (LLM Integration, Plugins, Extensions)     │
└─────────────────────────────────────────────┘
```

## Key Modules Quick Reference

| Module | Purpose | Key Components |
|--------|---------|----------------|
| `tokenizer` | Lexical analysis | `Tokenizer`, `Token`, various token types |
| `preprocessor` | Token normalization | `TokenPreprocessor`, `StringPreprocessor` |
| `analyzer` | Syntactic analysis | `Parser` trait, combinators, specialized parsers |
| `ast` | Abstract Syntax Tree | Agent/world definitions, statements, expressions |
| `type_checker` | Semantic analysis | Type validation, scope management, interfaces |
| `eval` | Runtime execution | Statement/expression evaluation, context management |
| `event` | Event distribution | `EventBus`, event registration, event handling |
| `provider` | LLM integration | Service providers, plugins, capability interfaces |
| `runtime` | Agent orchestration | Agent lifecycle, state management, coordination |

## DSL Quick Reference

KAIREI DSL has two main components:

### 1. World Definition

```kairei
world ExampleWorld {
    config {
        tick_interval: Duration = "1s"
    }
    
    events {
        CustomEvent(param: String)
    }
    
    handlers {
        on CustomEvent(param: String) {
            // Handle event
        }
    }
}
```

### 2. Agent Definition

```kairei
micro ExampleAgent {
    policy "Provide helpful responses with accurate information"
    
    state {
        counter: i64 = 0;
    }
    
    lifecycle {
        on_init {
            // Initialization code
        }
    }
    
    observe {
        on CustomEvent(param: String) {
            self.counter += 1;
        }
    }
    
    answer {
        on request GetCount() -> Result<i64, Error> {
            return Ok(self.counter);
        }
    }
    
    react {
        // Reactive behaviors
    }
}
```

## Common Development Tasks

### Building the Project

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Running KAIREI

```bash
RUST_LOG=kairei=debug cargo run --bin kairei
```

### Format & Lint

```bash
cargo fmt && cargo clippy -- -D warnings
```

## Error Handling Patterns

KAIREI uses result-based error handling with specialized error types for different stages:

- `ASTError` - Parsing and AST creation errors
- `TypeError` - Type checking errors
- `EvalError` - Runtime evaluation errors

## Type System

KAIREI implements a static type system with support for:

- Basic types: `String`, `i64`, `f64`, `Boolean`, etc.
- Complex types: `List<T>`, `Map<K,V>`, `Result<T,E>`
- Custom types: User-defined structured types
- Plugin types: Extension-provided types

## Event-Driven Architecture

The system uses an event-driven architecture with:

- Event emission and subscription
- Event-based communication between agents
- Asynchronous event processing

## Further Resources

- [KAIREI Documentation](../README.md)
- [Type System Details](../design/kairei_type_checker.md)
- [API Documentation](https://your-documentation-link.com) (generated with `cargo doc`)