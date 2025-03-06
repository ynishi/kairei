.PHONY: build run test test-core test-http fmt fmt-core fmt-http doc doc_check doc_open clean_doc all setup-hooks docker-build docker-run docker-push

test: test-core test-http

test-core:
	cd kairei-core && cargo test --quiet

test-http:
	cd kairei-http && cargo test --quiet


test_v:
	cd kairei-core && RUST_LOG=debug cargo test -p kairei $(CASE) --verbose

test_all: test_all-core test_all-http

test_all-core:
	cd kairei-core && RUN_API_TESTS=true RUST_LOG=error cargo test --all-features

test_all-http:
	cd kairei-http && RUN_API_TESTS=true RUST_LOG=error cargo test --all-features

bench: bench-core

bench-core:
	cd kairei-core && cargo bench

fmt: fmt-core fmt-http

fmt-core:
	cd kairei-core && cargo fmt
	cd kairei-core && cargo clippy --fix --allow-dirty
	cd kairei-core && cargo clippy -- -D warnings
	cd kairei-core && cargo fmt

fmt-http:
	cd kairei-http && cargo fmt
	cd kairei-http && cargo clippy --fix --allow-dirty
	cd kairei-http && cargo clippy -- -D warnings
	cd kairei-http && cargo fmt

doc: doc-core doc-http

doc-core:
	cd kairei-core && cargo doc --no-deps

doc-http:
	cd kairei-http && cargo doc --no-deps

doc_check: doc_check-core doc_check-http

doc_check-core:
	cd kairei-core && RUSTDOCFLAGS="-D warnings --cfg docsrs" cargo doc --no-deps --document-private-items --all-features

doc_check-http:
	cd kairei-http && RUSTDOCFLAGS="-D warnings --cfg docsrs" cargo doc --no-deps --document-private-items --all-features


doc_open:
	cd kairei-core && cargo doc --no-deps --open
	cd kairei-http && cargo doc --no-deps --open

clean_doc:
	cd kairei-core && rm -rf target/doc
	cd kairei-http && rm -rf target/doc

dev: dev-core dev-http

dev-core:
	cd kairei-core && RUST_LOG=kairei=debug cargo run --bin kairei

dev-http:
	cd kairei-http && RUST_LOG=kairei=debug cargo run

build:
	cargo build --workspace

setup-hooks:
	@echo "Setting up Git hooks..."
	@chmod +x scripts/install-hooks.sh
	@./scripts/install-hooks.sh

all: fmt test doc

# Docker build and run targets
docker-build:
	docker build -t kairei-http:latest .

docker-run:
	docker run -p 3000:3000 kairei-http:latest

docker-push:
	docker tag kairei-http:latest gcr.io/$(GCP_PROJECT_ID)/kairei-http:latest
	docker push gcr.io/$(GCP_PROJECT_ID)/kairei-http:latest
