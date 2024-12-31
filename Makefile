.PHONY: build run test clippy fmt

test:
	cargo test --verbose

fmt:
	cargo clippy -- -D warnings
	cargo fmt
