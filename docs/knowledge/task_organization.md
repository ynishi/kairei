# Task Organization and Scope Expansion Detection

## Overview
A feature to detect changes across multiple components and scope expansion, suggesting task splitting when appropriate.

## Detection Targets
- Simultaneous changes to multiple components
  - tokenizer
  - analyzer/parser
  - type_checker
  - other core components
- Design document reference verification

## Operation
1. PR Change Analysis:
   - Identify affected components
   - Verify docs/design references
   - Analyze change volume

2. Multi-component Change Response:
   - Suggest task splitting
   - Record in knowledge base
   - Add labels and comments to PR

## Best Practices
1. Create independent PRs per component
2. Reference design documents explicitly
3. Clarify dependencies

## Implementation Details
- Implemented in `.github/workflows/task-organization.yml`
- Implemented in `.github/workflows/scope-expansion-detection.yml`
- Uses GitHub Actions pull_request trigger
