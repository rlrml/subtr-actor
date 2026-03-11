# Releasing subtr-actor

This document describes how to release new versions of subtr-actor to GitHub Releases, crates.io, PyPI, and npm.

## Overview

The project publishes three packages:
- **Rust**: `subtr-actor` on [crates.io](https://crates.io/crates/subtr-actor)
- **Python**: `subtr-actor-py` on [PyPI](https://pypi.org/project/subtr-actor-py/)
- **JavaScript**: `rl-replay-subtr-actor` on [npm](https://www.npmjs.com/package/rl-replay-subtr-actor)

## Automated Releases (GitHub Actions)

When you push a tag starting with `v` (e.g., `v0.1.11`), the following workflows run automatically:

### GitHub (`release-github.yml`)
Creates or updates the GitHub Release page for the tag using the matching
section from `CHANGELOG.md`.

### Rust (`release-rust.yml`)
Publishes the Rust crate to crates.io.

### Python (`release-python.yml`)
Builds wheels for multiple platforms:
- Linux x86_64 (manylinux) - compatible with AWS Lambda, Docker, etc.
- Linux aarch64 (ARM)
- Windows x86_64
- macOS x86_64 and arm64
- Source distribution (sdist)

All artifacts are published to PyPI.

### JavaScript (`release-js.yml`)
Builds WebAssembly package and publishes to npm.

## Setup Required (One-Time)

### PyPI Publishing
Option A: **Trusted Publishing** (Recommended)
1. Go to PyPI → Account Settings → Publishing
2. Add a new pending publisher:
   - Owner: `rlrml`
   - Repository: `subtr-actor`
   - Workflow: `release-python.yml`
   - Environment: `pypi`

Option B: **API Token**
1. Create a PyPI API token at https://pypi.org/manage/account/token/
2. Add it as a GitHub secret named `PYPI_TOKEN`
3. Update the workflow to use the token instead of trusted publishing

### npm Publishing
1. Create an npm access token at https://www.npmjs.com/settings/~/tokens
2. Add it as a GitHub secret named `NPM_TOKEN`

## How to Release

### 1. Update Version Numbers

Update the version everywhere release metadata is sourced:

```bash
# Cargo.toml (workspace.package.version)
sed -i 's/version = "0.1.10"/version = "0.1.11"/' Cargo.toml

# python/pyproject.toml
sed -i 's/version = "0.1.10"/version = "0.1.11"/' python/pyproject.toml

# python/Cargo.toml and js/Cargo.toml dependency constraints
sed -i '/\\[dependencies\\.subtr-actor\\]/,/^\\[/{s/version = "0.1.10"/version = "0.1.11"/}' python/Cargo.toml
sed -i '/\\[dependencies\\.subtr-actor\\]/,/^\\[/{s/version = "0.1.10"/version = "0.1.11"/}' js/Cargo.toml

# js/package.json
sed -i 's/"version": "0.1.8"/"version": "0.1.11"/' js/package.json

# Cargo.lock
cargo metadata --format-version 1 >/dev/null
```

Or use the justfile helper:
```bash
just bump 0.1.11
```

### 2. Commit and Tag

```bash
git add Cargo.toml Cargo.lock python/pyproject.toml python/Cargo.toml js/package.json js/Cargo.toml
git commit -m "Release v0.1.11"
git tag v0.1.11
git push origin master --tags
```

### 3. Monitor the Release

- Go to the repository's Actions tab
- Watch the `Release GitHub`, `Release Rust`, `Release Python`, and `Release JavaScript` workflows
- Once complete, verify packages are available:
  - https://github.com/rlrml/subtr-actor/releases
  - https://crates.io/crates/subtr-actor
  - https://pypi.org/project/subtr-actor-py/
  - https://www.npmjs.com/package/rl-replay-subtr-actor

## Manual Release (Alternative)

If you need to release manually:

### Python
```bash
cd python
maturin build --release
twine upload target/wheels/*
```

### JavaScript
```bash
cd js
wasm-pack build --target bundler --out-dir pkg
cd pkg && npm publish
```

## Troubleshooting

### Python wheel build fails
- Ensure Rust toolchain is installed
- Check maturin version compatibility in `pyproject.toml`

### Linux wheel missing
- The `manylinux: auto` setting should produce compatible wheels
- If issues persist, try `manylinux: 2014` or `manylinux: 2_28`

### npm publish fails
- Verify `NPM_TOKEN` secret is set
- Check that the package name isn't taken
- Ensure version number is incremented
