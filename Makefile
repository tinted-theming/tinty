publish: publish_dry
	@echo "---------------"
	@echo "Running publish"
	@echo "---------------"
	cargo publish

publish_dry: test_docker
	@echo "-------------------"
	@echo "Running publish_dry"
	@echo "-------------------"
	cargo publish --dry-run

test_docker: setup_tests
	@echo "-------------------"
	@echo "Running test_docker"
	@echo "-------------------"
	docker build --target tests -t tinty-clippy .

test: setup_tests
	@echo "------------"
	@echo "Running test"
	@echo "------------"
	RUST_TEST_THREADS=1 cargo test --release

setup_tests: build
	@echo "-----------------"
	@echo "Creating fixtures"
	@echo "-----------------"
	./scripts/create_fixtures

build:
	@echo "-------------"
	@echo "Running build"
	@echo "-------------"
	cargo build --release
	cargo deny check

install:
	@echo "---------------"
	@echo "Installing deps"
	@echo "---------------"
	@if [ -z "$$(command -v cargo)" ]; then \
		echo "Installing rustup"; \
		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; \
	else \
		echo "rustup already installed"; \
	fi
	@if [ ! "$$(cargo about --version &>/dev/null)" ]; then \
		echo "Installing cargo about"; \
		cargo install --locked cargo-about; \
	else \
		echo "cargo-about already installed"; \
	fi
	@if [ ! "$$(cargo deny --version &>/dev/null)" ]; then \
		echo "Installing cargo deny"; \
		cargo install --locked cargo-deny; \
	else \
		echo "cargo-deny already installed"; \
	fi
