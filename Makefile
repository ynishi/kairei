.PHONY: build run test clippy fmt

test:
	RUST_LOG=debug cargo test -p kairei $(CASE) --verbose

test_all:
	RUN_API_TESTS=true RUST_LOG=error cargo test

fmt:
	cargo fmt
	cargo clippy -- -D warnings
	cargo fmt

dev:
	RUST_LOG=kairei=debug cargo run --bin kairei
