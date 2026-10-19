# Releasing subtr-actor

This document describes how to release new versions of subtr-actor to GitHub Releases, crates.io, PyPI, and npm.

## Overview

The project publishes five packages:

- **Rust**: `subtr-actor` on [crates.io](https://crates.io/crates/subtr-actor)
- **Python**: `subtr-actor-py` on [PyPI](https://pypi.org/project/subtr-actor-py/)
- **JavaScript bindings**: `@rlrml/subtr-actor` on [npm](https://www.npmjs.com/package/@rlrml/subtr-actor)
- **JavaScript player**: `@rlrml/player` on [npm](https://www.npmjs.com/package/@rlrml/player)
- **JavaScript stats player**: `@rlrml/stats-player` on [npm](https://www.npmjs.com/package/@rlrml/stats-player)

## Automated Releases (GitHub Actions)

When you push a tag starting with `v` (for example, `v<version>`), the following workflows run automatically:

### GitHub (`release-github.yml`)

Creates or updates the GitHub Release page for the tag using the matching
section from [`CHANGELOG.md`](../CHANGELOG.md).

### Rust (`release-rust.yml`)

Publishes the Rust crate to crates.io.

### Python (`release-python.yml`)

Builds abi3 wheels (one per platform, covering CPython 3.9+):

- Linux x86_64 and aarch64 (manylinux2014)
- Windows x86_64
- macOS universal2 (x86_64 and arm64)
- Source distribution (sdist)

All artifacts are published to PyPI.

### JavaScript (`release-js.yml`)

Builds the WebAssembly bindings package, builds the player package, smoke-tests
the packed player artifact in a fresh consumer app, builds the stats player
package, smoke-tests that packed artifact in a fresh consumer app, and publishes
all three npm packages. Without usable npm credentials the packages are still
built and smoke-tested, but a run on a release tag then fails instead of passing
with nothing published; runs on a branch act as a dry run.

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

Merging a version bump to `master` is enough: `auto-release.yml` notices that
the workspace version has no matching tag, creates `v<version>`, and the tag
triggers the release workflows above. To release from a local checkout
instead, use `just bump`, which also commits, tags, and pushes.

### 1. Update Version Numbers

Set the Cargo workspace version and synchronize every duplicated release
metadata file from that source of truth:

```bash
VERSION=${VERSION:?set VERSION to the target release}
python3 scripts/sync_release_versions.py "$VERSION"
python3 scripts/check_release_versions.py
```

Or use the justfile helper:

```bash
VERSION=${VERSION:?set VERSION to the target release}
just bump "$VERSION"
```

### 2. Commit and Tag

`just bump` already does this. If you ran the sync script by hand, commit the
version changes and either merge them (auto-release tags for you) or tag and
push yourself:

```bash
VERSION=${VERSION:?set VERSION to the target release}
git commit -am "Bump version to $VERSION"
git tag -a "v$VERSION" -m "Release v$VERSION"
git push origin master "v$VERSION"
```

### 3. Monitor the Release

- Go to the repository's Actions tab
- Watch the `Release GitHub`, `Release Rust`, `Release Python`, and `Release JavaScript` workflows
- Once complete, verify packages are available:
  - https://github.com/rlrml/subtr-actor/releases
  - https://crates.io/crates/subtr-actor
  - https://pypi.org/project/subtr-actor-py/
  - https://www.npmjs.com/package/@rlrml/subtr-actor
  - https://www.npmjs.com/package/@rlrml/player
  - https://www.npmjs.com/package/@rlrml/stats-player

## Manual Release (Alternative)

If you need to release manually:

### Python

```bash
cd python
maturin build --release
twine upload ../target/wheels/*
```

### JavaScript

```bash
cd js
npm run publish

cd ../player
package_dir="$(npm run --silent prepare:package)"
(cd "$package_dir" && npm publish --access public)

cd ../stat-evaluation-player
package_dir="$(npm run --silent prepare:package)"
(cd "$package_dir" && npm publish --access public)
```

## Troubleshooting

### Python wheel build fails

- Ensure Rust toolchain is installed
- Check maturin version compatibility in `pyproject.toml`

### Linux wheel missing

- Linux wheels are built with `--zig --compatibility manylinux2014`; check that
  the zig toolchain installed by `maturin[zig]` supports the target

### npm publish fails

- Verify `NPM_TOKEN` secret is set
- Check that the package name isn't taken
- Ensure version number is incremented
