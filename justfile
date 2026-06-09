# justfile for subtr-actor

# Rust and Python tooling should come from the flake dev shell rather than the
# caller's ambient PATH.
nix_develop := "nix develop -c"
nix_shell_bash := "nix develop -c bash -lc"

# Default recipe to list available commands
default:
    @just --list

# Build all packages
build:
    {{nix_develop}} cargo build --release

# Build Python package
build-python:
    {{nix_shell_bash}} 'cd python && cargo build --release'

# Build JavaScript/WASM package
build-js:
    cd js && npm exec wasm-pack -- build --target bundler

# Run the stats evaluation player dev server
dev-stats:
    cd js/stat-evaluation-player && npm run dev

# Run tests
test:
    {{nix_develop}} cargo test

# Download replay metadata/stats JSON from ballchasing.com for a specific replay id
ballchasing-replay-json replay_id output='':
    #!/usr/bin/env bash
    set -euo pipefail
    target_path="{{output}}"
    if [[ -z "${BALLCHASING_API_KEY:-}" ]]; then
        echo "BALLCHASING_API_KEY is not set. Run 'direnv allow' or export it manually." >&2
        exit 1
    fi
    if [[ -z "$target_path" ]]; then
        target_path="ballchasing-{{replay_id}}.json"
    fi
    curl --fail --silent --show-error \
        -H "Authorization: ${BALLCHASING_API_KEY}" \
        "https://ballchasing.com/api/replays/{{replay_id}}" \
        -o "$target_path"
    echo "Wrote $target_path"

# Download the raw replay file from ballchasing.com for a specific replay id
ballchasing-replay-file replay_id output='':
    #!/usr/bin/env bash
    set -euo pipefail
    target_path="{{output}}"
    if [[ -z "${BALLCHASING_API_KEY:-}" ]]; then
        echo "BALLCHASING_API_KEY is not set. Run 'direnv allow' or export it manually." >&2
        exit 1
    fi
    if [[ -z "$target_path" ]]; then
        target_path="ballchasing-{{replay_id}}.replay"
    fi
    curl --fail --silent --show-error \
        -H "Authorization: ${BALLCHASING_API_KEY}" \
        "https://ballchasing.com/api/replays/{{replay_id}}/file" \
        -o "$target_path"
    echo "Wrote $target_path"

# Download both the replay file and ballchasing JSON into flat asset files
ballchasing-fixture replay_id name:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p assets
    just ballchasing-replay-json {{replay_id}} "assets/{{name}}.ballchasing.json"
    just ballchasing-replay-file {{replay_id}} "assets/{{name}}.replay"
    printf '%s\n' '{{replay_id}}' > "assets/{{name}}.replay_id.txt"
    echo "Prepared fixture assets/{{name}}.*"

# Run Python tests
test-python:
    {{nix_shell_bash}} 'cd python && pytest'

# Publish main Rust crate to crates.io
publish-rust:
    {{nix_develop}} cargo publish -p subtr-actor

# Publish Python package to PyPI (builds sdist for cross-platform compatibility)
publish-python:
    {{nix_shell_bash}} 'cd python && maturin build --release --sdist'
    {{nix_shell_bash}} 'cd python && TWINE_USERNAME=__token__ TWINE_PASSWORD=$(pass show pypi.org | grep token: | awk '"'"'{print $2}'"'"') twine upload ../target/wheels/*'

# Publish JavaScript package to npm
publish-js: build-js
    cd js/pkg && npm publish

# Publish all packages in correct order (Rust first, then bindings)
publish-all: publish-rust publish-python publish-js
    @echo "All packages published successfully!"

# Clean build artifacts
clean:
    {{nix_develop}} cargo clean
    rm -rf python/dist
    rm -rf js/pkg
    rm -f python/*.so

# Format code
fmt:
    {{nix_develop}} cargo fmt

# Check formatting
fmt-check:
    {{nix_develop}} cargo fmt -- --check

# Run clippy
clippy:
    {{nix_develop}} cargo clippy -- -D warnings

# Version bump (requires version as argument)
# Updates workspace version and subtr-actor dependency in bindings, tags and pushes
bump version:
    python3 scripts/sync_release_versions.py "{{version}}"
    python3 scripts/check_release_versions.py
    git add -A
    git commit -m "Bump version to {{version}}"
    git tag -a "v{{version}}" -m "Release v{{version}}"
    git push && git push --tags
