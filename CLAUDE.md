# Kairei Development Guide

## Build Commands
- Build: `cargo build`
- Run: `RUST_LOG=kairei=debug cargo run --bin kairei`
- Format & Lint: `cargo fmt && cargo clippy -- -D warnings`
- Test: `cargo test`
- Test single case: `RUST_LOG=debug cargo test -p kairei <test_name> --verbose`
- API Tests: `RUN_API_TESTS=true RUST_LOG=error cargo test --all-features`
- Benchmark: `cargo bench`
- Documentation: `cargo doc --no-deps --open`
- Documentation Check: `RUSTDOCFLAGS="-D warnings --cfg docsrs" cargo doc --no-deps --document-private-items --all-features`

## Code Style Guidelines
- **Naming**: Use snake_case for functions/variables, CamelCase for types/traits
- **Imports**: Group and organize imports by external crates first, then internal modules
- **Error Handling**: Use Result<T, Error> with thiserror for custom errors, use tracing for logging errors
- **Documentation**: Add docstrings for all public APIs (structs, traits, functions)
- **Types**: Prefer strong typing with custom types over primitives for domain concepts
- **Formatting**: Run `cargo fmt` before committing to ensure consistent style
- **Testing**: Write unit tests for core functionality, integration tests for features
- **Error Messages**: Make error messages clear and actionable
- **Event-Driven**: Follow event-driven architecture patterns in agent communication

## Repository Structure
- **src/**: Core source code
  - **analyzer/**: DSL parsing and analysis
  - **eval/**: Code execution engine
  - **tokenizer/**: Lexical analysis
  - **type_checker/**: Type validation
  - **provider/**: LLM and plugin interfaces
  - **event/**: Event-driven communication
- **docs/**: Design documentation
- **examples/**: Example KAIREI applications