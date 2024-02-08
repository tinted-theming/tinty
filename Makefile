publish: publish_dry
	@echo "---------------"
	@echo "Running publish"
	@echo "---------------"
	cargo publish

publish_dry: test_docker
	@echo "-------------------"
	@echo "Running publish_dry"
	@echo "-------------------"
	@if [ -n "$(git status --porcelain)" ]; then \
		echo "There are changes." && exit 1; \
	fi
	cargo publish --dry-run

test_docker: setup_tests
	@echo "-------------------"
	@echo "Running test_docker"
	@echo "-------------------"
	docker build -t tinty . && docker run tinty

test: setup_tests
	@echo "------------"
	@echo "Running test"
	@echo "------------"
	RUST_TEST_THREADS=1 cargo test

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

