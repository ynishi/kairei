# KAIREI Plugin Layer Strategy

## Overview

This document outlines KAIREI's strategy for the Plugin Layer in its three-layer architecture (Native, Plugin, MicroAgent). It clarifies the current development approach and outlines future considerations for extending the system.

## Current Approach: Plugin Layer Deferral

This document explains why KAIREI does not currently implement a general Plugin Layer and how the system maintains extensibility through modular plugin interfaces. It also outlines future conditions for introducing a Plugin Layer when necessary.

### 1. Plugin Layer is NOT needed at this stage

At KAIREI's current development stage, a full Plugin Layer implementation is **intentionally deferred**:

- KAIREI's functionality is sufficiently provided by the Native and MicroAgent layers
- A formal Plugin Layer would add unnecessary complexity and maintenance overhead
- Current extension needs can be effectively managed through PR-based contributions
- The Provider-specific plugin system meets current LLM integration needs

### 2. PR-based "PluginMods" for Extensions

Instead of a dedicated Plugin Layer, KAIREI will manage extensions through PR-based "PluginMods":

- Contributors can extend the system through standard Pull Requests
- Extensions are reviewed, maintained, and integrated into the core codebase
- This approach ensures code quality and architectural consistency
- It provides flexibility without the complexity of a formal plugin system

### 3. Module-Based Plugin Systems vs System-Wide Plugin Layer

KAIREI currently implements module-specific extension points (which we call "PluginMods") rather than a system-wide Plugin Layer:

#### Existing Module-Based Plugin Systems

1. **Provider Plugins**
   - Extension mechanism for LLM integration
   - Focused solely on extending provider capabilities
   - Implemented through the `ProviderPlugin` trait
   - Allows for memory, policy, and web search extensions

   ```rust
   #[async_trait]
   pub trait ProviderPlugin: Send + Sync {
       fn priority(&self) -> i32;
       fn capability(&self) -> CapabilityType;
       async fn generate_section<'a>(&self, context: &PluginContext<'a>) -> ProviderResult<Section>;
       async fn process_response<'a>(&self, context: &PluginContext<'a>, response: &LLMResponse) -> ProviderResult<()>;
   }
   ```

2. **Type Checker Plugins**
   - Extension mechanism for the type checking system
   - Allows for custom type validation logic
   - Implemented through the `TypeCheckerPlugin` trait
   - Provides hooks into the AST visitor pattern

   ```rust
   pub trait TypeCheckerPlugin {
       fn before_root(&self, _root: &mut Root, _ctx: &mut TypeContext) -> TypeCheckResult<()> { Ok(()) }
       fn after_root(&self, _root: &mut Root, _ctx: &mut TypeContext) -> TypeCheckResult<()> { Ok(()) }
       // Additional lifecycle hooks for various AST nodes
   }
   ```

Each module in KAIREI that requires extensibility implements its own plugin interface tailored to its specific needs. This approach provides focused extension points without the overhead of a general plugin system.

#### System-Wide Plugin Layer (Deferred)

In contrast to these module-specific extensions:

- **System-wide Plugin Layer**: General extensibility layer (deferred)
  - Would provide unified system-wide extension points (events, state, functions)
  - Would require a formal plugin management system with installation and discovery
  - Would standardize plugin interfaces across all system components
  - Deferred until specific use cases justify the complexity

## Current Layer Responsibilities

### Native Layer Responsibilities

- Core system functionality and infrastructure
- Event management and distribution
- State primitives and access patterns
- Resource monitoring and control
- System lifecycle management

### MicroAgent Layer Responsibilities

- User-facing DSL interface
- Business logic implementation
- Event handling (observe, react, answer patterns)
- Agent lifecycle management
- Integration with LLM providers through the Provider subsystem

## Future Plugin Layer Considerations

### When to Revisit the Plugin Layer Concept

The Plugin Layer concept should be revisited when:

1. **Ecosystem Growth**: KAIREI's ecosystem grows to include third-party developers
2. **Installation-based Extensions**: When runtime installation of extensions becomes necessary
3. **Community Demand**: Clear community demand for a formal plugin system emerges
4. **Extension Patterns**: Common extension patterns emerge that would benefit from standardization

### Potential Future Implementation

When revisited, the Plugin Layer could include:

- **Common Plugin Interface**: `KaireiPlugin` trait with lifecycle methods
- **Plugin Types**:
  - `EventEmitterPlugin`: For extending the event system
  - `StateProviderPlugin`: For extending state management
  - `FunctionProviderPlugin`: For adding callable functions
- **Plugin Management**: Registration, discovery, and lifecycle management
- **Resource Control**: Memory, computation, and API access management
- **Versioning**: API versioning and compatibility checking

### Migration Path

When implementing a formal Plugin Layer:

1. Start with existing provider plugins as a pattern
2. Create a common plugin infrastructure
3. Gradually migrate provider plugins to the new system
4. Extend plugin capabilities to other system components

## Conclusion

KAIREI's current architecture intentionally defers the Plugin Layer in favor of simpler PR-based extensions. This pragmatic approach allows the system to evolve naturally while avoiding unnecessary complexity. As the ecosystem grows and specific needs emerge, the Plugin Layer concept can be revisited with clearer requirements and use cases.