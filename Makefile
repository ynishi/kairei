.PHONY: build run test clippy fmt doc doc_check doc_open clean_doc all setup-hooks

test:
	cargo test --quiet

test_v:
	RUST_LOG=debug cargo test -p kairei $(CASE) --verbose

test_all:
	RUN_API_TESTS=true RUST_LOG=error cargo test --all-features

bench:
	cargo bench

fmt:
	cargo fmt
	cargo clippy -- -D warnings
	cargo fmt

doc:
	cargo doc --no-deps

doc_check:
	RUSTDOCFLAGS="-D warnings --cfg docsrs" cargo doc --no-deps --document-private-items --all-features

doc_open:
	cargo doc --no-deps --open

clean_doc:
	rm -rf target/doc

dev:
	RUST_LOG=kairei=debug cargo run --bin kairei

build:
	cargo build

setup-hooks:
	@echo "Setting up Git hooks..."
	@chmod +x scripts/install-hooks.sh
	@./scripts/install-hooks.sh

all: fmt test doc
