.PHONY: build run test clippy fmt

test:
	RUST_LOG=debug cargo test --verbose

fmt:
	cargo clippy -- -D warnings
	cargo fmt

dev:
	RUST_LOG=kairei=debug cargo run --bin kairei
