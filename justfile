test_docker: setup_tests
    @echo "-------------------"
    @echo "Running test_docker"
    @echo "-------------------"
    DOCKER_BUILDKIT=1 docker build -t tinty-test --target tests .

test pattern: setup_tests
    @echo "------------"
    @echo "Running test"
    @echo "------------"
    RUST_TEST_THREADS=1 cargo test --release {{pattern}}

setup_tests: lint build
    @echo "-----------------"
    @echo "Creating fixtures"
    @echo "-----------------"
    ./scripts/create_fixtures

build:
    @echo "-------------"
    @echo "Running build"
    @echo "-------------"
    cargo build --release

lint:
    @echo "------------"
    @echo "Running lint"
    @echo "------------"
    cargo fmt --all --check
    cargo clippy
    cargo deny check

list:
    @just --list
