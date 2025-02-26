# KAIREI Think Instruction Design

## Purpose
This document clarifies how the `think` instruction works in KAIREI, especially its integration with the type system, policy system, and error handling. It serves as a reference for developers using the `think` instruction in their KAIREI applications.

## Overview
The `think` instruction is designed for LLM integration and does not enforce strict type safety. It provides a flexible interface for KAIREI agents to leverage large language models for various cognitive tasks.

The `think` instruction serves as the primary mechanism for KAIREI agents to utilize LLM capabilities, enabling them to:
- Generate creative content
- Analyze data and provide insights
- Make decisions based on context
- Process natural language inputs
- Integrate with external knowledge sources

Unlike other KAIREI instructions that operate within a strict type system, `think` is intentionally designed with flexibility to accommodate the inherently probabilistic and creative nature of LLM outputs.

## Type System Interaction

### Limited Type Checking
The `think` instruction exists partially outside KAIREI's strict type checking system for several reasons:

1. **LLM Output Flexibility**: LLM responses are inherently variable and cannot be fully predicted at compile time.

2. **Runtime Interpretation**: The actual content and structure of LLM responses are determined at runtime, making static type checking impractical.

3. **String-Based Interface**: The primary interface with LLMs is string-based, which doesn't align with KAIREI's rich type system.

### Type Checking Behavior
While the `think` instruction itself has limited type checking, KAIREI does perform the following validations:

1. **Argument Validation**: The arguments passed to `think` are type-checked to ensure they can be properly serialized.

2. **Variable Interpolation**: When using variable interpolation (e.g., `${variable}`), the system verifies that the referenced variables exist and can be converted to strings.

3. **Return Type**: The `think` instruction always returns a `Result<String, Error>` type, allowing for error handling.

4. **With Block Validation**: The optional `with` block parameters are validated for correct structure and types.

### Type System Integration
Despite limited type checking, `think` integrates with the type system in the following ways:

1. **Result Type**: The `think` instruction returns a `Result` type, consistent with KAIREI's error handling patterns.

2. **Context Access**: The `think` instruction has access to typed state variables and can incorporate them into prompts.

3. **Plugin Configuration**: Type checking is applied to plugin configurations in the `with` block.

## Policy Integration

### Policy Application
Policies in KAIREI provide guidance to LLMs on how to generate responses. The `think` instruction integrates with the policy system in the following ways:

1. **Automatic Policy Collection**: When a `think` instruction is executed, the system automatically collects relevant policies from:
   - World-level policies
   - Agent-specific policies
   - Think-specific policies (defined in the `with` block)

2. **Policy Injection**: Collected policies are injected into the prompt sent to the LLM, influencing the generated response.

3. **Policy Prioritization**: Policies are applied based on their scope and specificity, with more specific policies taking precedence.

### Policy Scope
Policies can be defined at different levels:

```rust
world NewsAnalysis {
    // World-level policy (applies to all agents)
    policy "Ensure factual accuracy with multiple sources"
    policy "Use recent information, prefer within 24 hours"

    micro NewsAnalyst {
        // Agent-level policy (applies to this agent only)
        policy "Prioritize official and reliable sources"

        answer {
            on request AnalyzeNews(topic: String) -> Result<Analysis> {
                // Think-specific policy (applies to this think call only)
                think("Analyze current news about ${topic}") with {
                    policies: ["Focus on economic impact"]
                }
            }
        }
    }
}
```

### Policy Enforcement
Policies in KAIREI are not strictly enforced constraints but rather guidance for the LLM. The effectiveness of policies depends on:

1. **LLM Capabilities**: More advanced LLMs can better understand and follow complex policies.

2. **Policy Clarity**: Clear, specific policies are more likely to be followed accurately.

3. **Policy Consistency**: Consistent policies across different levels lead to more predictable results.

## Error Handling

### Error Types
The `think` instruction can produce several types of errors:

1. **Provider Errors**: Issues with the LLM provider, such as API failures, rate limiting, or authentication problems.

2. **Content Errors**: Issues with the generated content, such as inappropriate responses, hallucinations, or incomplete answers.

3. **Context Errors**: Issues with the context provided to the LLM, such as exceeding token limits or invalid prompt structure.

4. **Plugin Errors**: Issues with plugins used in the `with` block, such as search failures or memory retrieval problems.

### Error Handling Strategies
KAIREI provides several strategies for handling `think` instruction errors:

1. **Basic Error Handling**:
   ```rust
   let result = think("Generate content");
   match result {
       Ok(content) => use_content(content),
       Err(error) => handle_error(error)
   }
   ```

2. **Retry Logic**:
   ```rust
   think("Analyze market data") with {
       search: {
           recent: "24h",
           filter: "financial"
       }
   } on fail {
       retry: {
           max_attempts: 3,
           delay: "exponential"
       }
   }
   ```

3. **Fallback Mechanisms**:
   ```rust
   let result = think("Generate complex analysis");
   let content = match result {
       Ok(content) => content,
       Err(_) => think("Generate simple analysis").unwrap_or_default()
   };
   ```

4. **Error Logging and Monitoring**:
   ```rust
   let result = think("Generate content");
   if let Err(error) = &result {
       log_error(error);
       emit ErrorOccurred(error.to_string());
   }
   ```

### Best Practices
When using the `think` instruction, follow these best practices for error handling:

1. **Always Check Results**: Never assume a `think` instruction will succeed; always handle the Result type.

2. **Implement Retries**: For critical operations, implement retry logic with appropriate backoff.

3. **Provide Fallbacks**: Have alternative strategies when LLM responses fail or are inadequate.

4. **Monitor and Log**: Track error patterns to identify recurring issues and improve prompts.

5. **Validate Outputs**: When possible, validate LLM outputs before using them in critical operations.

## Example Usage

### Basic Usage
The simplest form of the `think` instruction takes a string prompt:

```rust
// Basic form
think("Create a travel plan to Tokyo")

// Variable interpolation
think("Analyze market trends for ${product}")

// With context
think("Create comprehensive analysis", previous_data)
```

### With Block Customization
The `with` block allows customization of the `think` instruction:

```rust
think("Analyze recent AI developments") with {
    // Search settings
    search: {
        recent: "24h",    // Time period
        filter: "news",   // Filter type
        limit: 5          // Result limit
    },
    
    // Provider settings
    provider: "gpt-4",    // LLM provider
    
    // LLM parameters
    params: {
        temperature: 0.7,
        max_tokens: 1000
    }
}
```

### Policy-Guided Thinking
Policies can guide the `think` instruction:

```rust
world NewsAnalysis {
    // Intent expressed in natural language
    policy "Ensure factual accuracy with multiple sources"
    policy "Use recent information, prefer within 24 hours"
    policy "Prioritize official and reliable sources"

    micro NewsAnalyst {
        answer {
            on request AnalyzeNews(topic: String) -> Result<Analysis> {
                // Policies are automatically considered
                think("Analyze current news about ${topic}")
            }
        }
    }
}
```

### Parallel Thinking
Multiple `think` operations can be executed in parallel:

```rust
micro NewsAnalyst {
    answer {
        on request MarketReport(industry: String) -> Result<Report> {
            // Parallel news and analysis retrieval
            let [news, stats, trends] = await [
                think("Find latest news about ${industry}") with {
                    search: { filter: "news" }
                },
                think("Get market statistics for ${industry}") with {
                    search: { filter: "financial" }
                },
                think("Identify emerging trends in ${industry}") with {
                    search: { recent: "7d" }
                }
            ];

            // Integrate results
            think("Create comprehensive market report", {
                news: news,
                stats: stats,
                trends: trends
            })
        }
    }
}
```

### Error Handling Examples
Examples of error handling with the `think` instruction:

```rust
// Basic error handling
let result = think("Generate content");
match result {
    Ok(content) => use_content(content),
    Err(error) => handle_error(error)
}

// Retry logic
think("Analyze market data") with {
    search: {
        recent: "24h",
        filter: "financial"
    }
} on fail {
    retry: {
        max_attempts: 3,
        delay: "exponential"
    }
}

// Fallback mechanism
let result = think("Generate complex analysis");
let content = match result {
    Ok(content) => content,
    Err(_) => think("Generate simple analysis").unwrap_or_default()
};
```

## Conclusion

The `think` instruction is a powerful feature in KAIREI that enables agents to leverage LLM capabilities. While it operates with limited type safety, it integrates with KAIREI's policy system and provides robust error handling mechanisms.

By understanding how `think` interacts with the type system, how policies influence responses, and how to handle errors effectively, developers can create more reliable and effective KAIREI applications.
