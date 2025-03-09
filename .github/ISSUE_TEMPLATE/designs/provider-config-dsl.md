# Provider Configuration DSL Design

## Overview
This design issue explores the feasibility and value of implementing a Domain-Specific Language (DSL) or YAML-based configuration for Provider LLM integration in KAIREI. The goal is to make it easier for contributors to extend KAIREI with new provider implementations without modifying core code, similar to browser extensions or GitHub Actions ecosystem.

## Current Architecture
The current Provider configuration system in KAIREI is implemented as a Rust native module with the following characteristics:

- Well-defined Provider trait interface with lifecycle methods
- Support for multiple provider types (OpenAIAssistant, SimpleExpert, OpenAIChat)
- Plugin architecture for Memory, RAG, and Search capabilities
- Configuration through nested Rust structs with Serde serialization
- Extensive validation mechanisms for type safety and configuration correctness

### Key Components

1. **Provider Trait**
   - Defines the interface for all providers
   - Includes lifecycle methods (init, send_message, etc.)
   - Supports capability reporting and validation

2. **Configuration System**
   - Hierarchical configuration with base and provider-specific options
   - Plugin configurations (Memory, RAG, Search)
   - Validation through the `ConfigValidation` trait
   - Dynamic configuration via `HashMap<String, serde_json::Value>`

3. **Plugin Architecture**
   - Modular extension of provider capabilities
   - Priority-based execution
   - Capability matching for plugin selection

## Problem Statement
While the current system is powerful and type-safe, it presents a high barrier to entry for contributors who want to extend KAIREI with new provider implementations. Contributors currently need to:

1. Understand the Rust Provider trait interface
2. Implement provider-specific configuration structures
3. Write validation logic for their configurations
4. Integrate with the provider registry
5. Modify core code to add their provider

This complexity limits the potential for a vibrant ecosystem of provider implementations and extensions.

## Proposed Solution
Implement a DSL or YAML-based configuration system for Provider LLM integration with the following features:

### Option 1: YAML-based Configuration
```yaml
provider:
  type: openai_chat
  name: my_openai_provider
  config:
    model: gpt-4
    temperature: 0.7
  plugins:
    memory:
      enabled: true
      max_tokens: 1000
    search:
      enabled: true
      max_results: 5
```

### Option 2: Simplified Domain-Specific Language
```
provider OpenAIChat MyProvider {
  model: "gpt-4"
  temperature: 0.7
  
  memory {
    enabled: true
    max_tokens: 1000
  }
  
  search {
    enabled: true
    max_results: 5
  }
}
```

## Implementation Approach

### Parser Implementation
1. Create a parser that converts DSL/YAML to Rust configuration structures
   - For YAML: Use serde_yaml for deserialization
   - For custom DSL: Implement a custom parser (potentially using nom or pest)

2. Configuration Mapping
   - Map DSL/YAML fields to corresponding Rust structs
   - Handle nested configurations for plugins
   - Support dynamic fields through HashMaps

### Validation Integration
1. Maintain the existing validation mechanisms
   - Apply ConfigValidation trait methods to parsed configurations
   - Collect and report validation errors with source locations
   - Support both compile-time and runtime validation

### Registry Integration
1. Extend the provider registry to support DSL-based providers
   - Add methods to load providers from DSL/YAML files
   - Support dynamic provider registration
   - Maintain backward compatibility with native providers

### Migration Path
1. Provide a clear migration path for existing configurations
   - Generate DSL/YAML from existing Rust configurations
   - Support both native and DSL-based providers simultaneously
   - Document the migration process

## Complexity Assessment

| Aspect | Complexity | Notes |
|--------|------------|-------|
| Parser Implementation | Moderate | YAML parsing is straightforward with serde_yaml; custom DSL would require more effort |
| Type Safety | Low | Can leverage existing validation mechanisms |
| Configuration Mapping | Low | Straightforward with Serde |
| Validation | Low | Can reuse existing validation logic |
| Integration | Moderate | Requires changes to provider registry |
| Documentation | Moderate | Comprehensive documentation needed for contributors |

## Value Assessment

| Aspect | Value | Notes |
|--------|-------|-------|
| Contributor Experience | High | Significantly lowers barrier to entry |
| Extensibility | High | Enables easier creation of new providers |
| Ecosystem Potential | High | Similar to browser extensions or GitHub Actions |
| Maintenance | Medium | Slightly increased complexity but manageable |
| Alignment | High | Consistent with KAIREI's architectural goals |

## Recommendation
Based on the analysis, implementing a **YAML-based Provider configuration system** is **recommended** for the following reasons:

1. It significantly lowers the barrier to entry for contributors
2. The implementation complexity is moderate and manageable
3. Existing type safety and validation can be maintained
4. It aligns with the project's goal of creating an extensible ecosystem
5. It provides a clear path for future WASM support

YAML is preferred over a custom DSL for the initial implementation because:
1. It leverages existing Serde serialization/deserialization
2. It has lower implementation complexity
3. It is widely understood by developers
4. It can be extended to support a custom DSL in the future if needed

## Next Steps
1. Create a prototype parser for YAML-to-Rust configuration conversion
2. Implement validation for the YAML configuration
3. Update the provider registry to support DSL-based providers
4. Create documentation and examples for contributors
5. Develop a migration guide for existing providers

## Full Provider Implementation via YAML

This section explores extending the YAML-based configuration approach to support full provider implementation, allowing contributors to add new providers (like Claude) without modifying core code.

### Implementation Approach

1. **Provider Adapter Registry**
   - Create a registry that maps provider types to adapter implementations
   - Adapters handle provider-specific API interactions
   - Registry dynamically loads adapters based on YAML configuration

```yaml
provider:
  type: claude
  name: my_claude_provider
  implementation:
    adapter: claude_adapter  # Points to a Rust adapter that handles Claude API
    capabilities:
      - Generate
      - SystemPrompt
  config:
    model: claude-3-opus
    temperature: 0.7
  plugins:
    memory:
      enabled: true
      max_tokens: 1000
```

2. **Adapter Implementation**
   - Each provider type requires a corresponding adapter implementation
   - Adapters implement the `ProviderLLM` trait
   - Adapters handle provider-specific API calls and response parsing
   - Example adapter structure:

```rust
pub struct ClaudeAdapter {
    client: Option<Client>,
    name: String,
    capabilities: Capabilities,
}

#[async_trait]
impl ProviderLLM for ClaudeAdapter {
    async fn send_message(&self, prompt: &str, config: &ProviderConfig) -> ProviderResult<LLMResponse>;
    fn capabilities(&self) -> Capabilities;
    fn name(&self) -> &str;
    async fn initialize(&mut self, config: &ProviderConfig, secret: &ProviderSecret) -> ProviderResult<()>;
}
```

3. **Dynamic Provider Registration**
   - Extend `ProviderRegistry` to support loading providers from YAML
   - Add methods to register providers from YAML files
   - Implement dynamic adapter selection based on provider type

### Integration with Existing Systems

1. **Validation Integration**
   - Reuse existing validation mechanisms for YAML configurations
   - Extend validators to support provider-specific validation
   - Implement schema validation for YAML provider configurations

2. **Plugin Integration**
   - Support plugin configuration via YAML
   - Allow plugins to be dynamically loaded and configured
   - Maintain existing plugin architecture and lifecycle

### Complexity Assessment

| Component | Complexity | Notes |
|-----------|------------|-------|
| Provider Adapter Registry | Moderate | Requires careful design for extensibility |
| YAML Parser | Low | Can leverage serde_yaml |
| Dynamic Provider Registration | Moderate | Needs to handle errors and validation |
| Validation Integration | Low | Can reuse existing validation mechanisms |
| Plugin Integration | Low | Can leverage existing plugin architecture |
| Overall | Moderate | Feasible with reasonable effort |

### Benefits

1. **Contributor Experience**
   - Significantly lowers barrier to entry for new provider implementations
   - Familiar YAML syntax for configuration
   - No need to modify core code

2. **Extensibility**
   - Enables a plugin ecosystem for providers
   - Allows for community-contributed providers
   - Simplifies adding support for new LLM APIs

3. **Maintenance**
   - Reduces core code complexity
   - Isolates provider-specific code in adapters
   - Simplifies testing and validation

### Implementation Plan

1. Create provider adapter registry
2. Implement YAML parser for provider configurations
3. Extend provider registry to support YAML-based providers
4. Create example adapter implementations
5. Implement validation for YAML provider configurations
6. Add documentation and examples

## Open Questions
1. Should we support both YAML and a custom DSL, or focus on one approach?
2. How should we handle versioning of the configuration format?
3. Should we provide tooling to validate configurations before runtime?
4. How should we handle provider-specific configuration options that may not be known at compile time?
5. What is the best approach for error reporting in the DSL/YAML parser?
6. How should we handle authentication and secrets for YAML-defined providers?
7. What is the best approach for distributing and discovering community-contributed provider adapters?
