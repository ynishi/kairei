# KAIREI Documentation Strategy

## Overview
KAIREI documentation is managed in two primary locations:

1. RustDoc: "Current State of the Product"
   - Technical specifications tightly coupled with implementation
   - API documentation
   - Component design and structure

2. docs/: "Historical Context and Process"
   - Development processes
   - Design decision records
   - Tutorials and use cases
   - Visual documentation and diagrams

## RustDoc Structure

### lib.rs - Project Overview
```rust
//! # KAIREI
//! 
//! AI Agent Orchestration Platform
//! 
//! ## Architecture Overview
//! KAIREI provides an execution environment for LLM-powered AI agents...
//! 
//! ## Core Design
//! Event-driven architecture with MicroAgent model...
```

### dsl/mod.rs - DSL Specification
```rust
//! # KAIREI DSL
//! 
//! KAIREI DSL consists of the following core components:
//! - World definitions
//! - MicroAgent definitions
//! - think syntax
```

### agent/mod.rs - Agent Design
```rust
//! # MicroAgent
//! 
//! MicroAgent is an execution unit based on the Single Responsibility Principle...
```

## docs/ Structure
- design/: Architecture and design documents
- process/: Development processes and workflows
- tutorials/: Tutorials and guides
- assets/: Diagrams and visual assets

## Cross-referencing
- RustDoc to docs/ references
  - Design background and detailed explanations
  - Diagrams and visual assets

- docs/ to RustDoc references
  - API specifications and interfaces
  - Implementation details

## Maintenance
- RustDoc: Review during code reviews
- docs/: Update during design changes and feature additions
- Regular documentation review cycles
