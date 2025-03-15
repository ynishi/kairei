# Guide for Working with Devin on Kairei

This guide outlines how to effectively collaborate with Devin when working on the Kairei project. Devin functions as a proactive development partner rather than just a code generation tool, focusing on implementation quality, testing, and providing design feedback based on practical experience.

## Purpose

This document serves as a reference for developers working with Devin on the Kairei project. It provides guidance on development practices, build commands, and the overall approach to contributing to Kairei when collaborating with Devin. The guide complements the existing documentation while focusing on practical implementation aspects.

## Sistens Approach

As a Sistens-type development partner, Devin takes a proactive approach to development:

### Proactive Development
- **Anticipate challenges** before they arise
- **Suggest improvements** to code structure and design
- **Identify potential issues** in implementation
- **Maintain context** across the entire codebase

### Implementation Focus
- Prioritize writing functional, well-tested code
- Provide practical feedback based on implementation experience
- Suggest refactoring opportunities when appropriate
- Focus on code quality and maintainability

### Root Cause Analysis
- When encountering errors, investigate underlying causes
- Look beyond surface-level fixes to identify design issues
- Provide comprehensive analysis of problems
- Suggest structural improvements when appropriate

### Design Feedback Loop
- Implement according to specifications
- Provide feedback based on practical implementation experience
- Suggest design improvements when implementation reveals issues
- Balance adherence to design with practical considerations

## Build and Test Commands

### Basic Commands
- `make build` - Build the entire workspace
- `make test` - Run all tests across all crates
- `make fmt` - Format and lint all code

### Crate-Specific Commands
- `make test-core` - Run tests for kairei-core
- `make test-http` - Run tests for kairei-http
- `make test-cli` - Run tests for kairei-cli

### Detailed Testing
- `make test_v CASE=test_name` - Run a specific test with verbose output
- `make test_all` - Run all tests including API tests with all features

### Documentation
- `make doc` - Generate documentation
- `make doc_open` - Generate and open documentation in browser
- `make doc_check` - Verify documentation for warnings

### Development
- `make dev-core` - Run kairei-core in development mode
- `make dev-http` - Run kairei-http in development mode
- `make dev-cli` - Run kairei-cli in development mode

### Git Hooks
- `make setup-hooks` - Set up git hooks for pre-commit and pre-push checks
  - pre-commit: Runs `make fmt` (warning only)
  - pre-push: Runs `make test` (blocks push if tests fail)

## Development Workflow

### Issue Analysis
1. Thoroughly review the issue using:
   - `gh issue view {issue_number} --json title,body,comments`
   - `gh issue comment list {issue_number} --json body,author,createdAt`

2. For complex issues:
   - Analyze root causes
   - Document findings in `.devin_workspace/`
   - Consider breaking down into smaller tasks

### Implementation Process
1. **Pre-implementation**
   - Check existing code patterns
   - Review related tests
   - Understand the module's responsibility in the architecture

2. **Implementation**
   - Write code that follows Rust best practices
   - Maintain consistency with existing patterns
   - Add appropriate tests
   - Document public APIs

3. **Pre-PR Verification**
   - Run `make build` to verify compilation
   - Run `make fmt` to ensure proper formatting
   - Run `make test` to verify functionality

### PR Creation and Review
1. Create a branch with format: `devin/{timestamp}-{feature-name}`
2. Create PR with:
   - Clear description of changes
   - Reference to the issue
   - Mention of testing performed
   - Link to Devin run

3. Address feedback:
   - Run `make build` after implementing fixes
   - Run `make fmt` to ensure proper formatting
   - Run `make test` to verify functionality
   - Push changes to the same PR



## Reference Documentation

### Quick Reference
The [Quick Reference Guide](docs/quick_reference/index.md) provides essential information about:
- System architecture
- Key components
- DSL syntax
- Common development tasks

### Design Documentation
Detailed design documents are available in the [docs/design](docs/design/) directory:
- [Architecture](docs/design/architecture.md)
- [Event Architecture](docs/design/event_architecture.md)
- [Type Checker](docs/design/kairei_type_checker.md)
- [Plugin Layer Strategy](docs/design/plugin_layer_strategy.md)
- [Sistence Agent](docs/design/sistence_agent.md)
- [Think Instruction](docs/design/think_instruction.md)

For a complete understanding of the documentation structure, refer to the [Documentation Strategy](docs/doc_strategy.md).
