#!/usr/bin/env bash
set -euo pipefail

# Keep this version in sync with the cargo-rdme provided by the flake dev shell
# (see flake.nix `shellPackages`) so that local `just check-readme` and CI agree
# on generated output. If they drift, bump whichever is behind.
version="${CARGO_RDME_VERSION:-1.5.0}"
install_dir="${CARGO_HOME:-$HOME/.cargo}/bin"
export PATH="$install_dir:$PATH"

if command -v cargo-rdme >/dev/null 2>&1 && cargo-rdme --version | grep -q "cargo-rdme $version"; then
  cargo-rdme --version
  exit 0
fi

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64)
    target="x86_64-unknown-linux-musl"
    ;;
  Darwin-x86_64)
    target="x86_64-apple-darwin"
    ;;
  Darwin-arm64)
    target="aarch64-apple-darwin"
    ;;
  *)
    echo "unsupported cargo-rdme install target: $(uname -s)-$(uname -m)" >&2
    exit 1
    ;;
esac

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

archive="$tmpdir/cargo-rdme.tar.bz2"
url="https://github.com/orium/cargo-rdme/releases/download/v${version}/cargo-rdme_v${version}_${target}.tar.bz2"

if ! curl \
  --fail \
  --location \
  --show-error \
  --silent \
  --retry 5 \
  --retry-all-errors \
  --retry-delay 5 \
  --connect-timeout 30 \
  --output "$archive" \
  "$url"; then
  echo "failed to download cargo-rdme release archive; falling back to cargo install" >&2
  cargo install cargo-rdme --version "$version" --locked
  cargo-rdme --version
  exit 0
fi

tar -xjf "$archive" -C "$tmpdir"
mkdir -p "$install_dir"
install "$tmpdir/cargo-rdme" "$install_dir/cargo-rdme"

cargo-rdme --version
