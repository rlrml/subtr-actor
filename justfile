# justfile for subtr-actor

# Default recipe to list available commands
default:
    @just --list

# Build all packages
build:
    cargo build --release

# Build Python package
build-python:
    cd python && cargo build --release

# Build JavaScript/WASM package
build-js:
    cd js && wasm-pack build --target bundler

# Run tests
test:
    cargo test

# Run Python tests
test-python:
    cd python && pytest

# Publish Python package to PyPI (builds sdist for cross-platform compatibility)
publish-python:
    cd python && rm -rf dist/*
    cd python && maturin build --release --sdist
    cd python && TWINE_USERNAME=__token__ TWINE_PASSWORD=$(pass show pypi.org | grep token: | awk '{print $2}') twine upload dist/*

# Publish JavaScript package to npm
publish-js: build-js
    cd js && npm publish

# Clean build artifacts
clean:
    cargo clean
    rm -rf python/dist
    rm -rf js/pkg
    rm -f python/*.so

# Format code
fmt:
    cargo fmt

# Check formatting
fmt-check:
    cargo fmt -- --check

# Run clippy
clippy:
    cargo clippy -- -D warnings

# Version bump (requires version as argument)
bump version:
    sed -i 's/version = "[0-9]\+\.[0-9]\+\.[0-9]\+"/version = "{{version}}"/' Cargo.toml
    sed -i 's/version = "[0-9]\+\.[0-9]\+\.[0-9]\+"/version = "{{version}}"/' python/pyproject.toml
    sed -i 's/"version": "[0-9]\+\.[0-9]\+\.[0-9]\+"/"version": "{{version}}"/' js/package.json
    git add -A
    git commit -m "Bump version to {{version}}"