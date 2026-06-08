#!/usr/bin/env bash
set -euo pipefail

version="${WASM_PACK_VERSION:-0.13.1}"
install_dir="${CARGO_HOME:-$HOME/.cargo}/bin"
export PATH="$install_dir:$PATH"

if command -v wasm-pack >/dev/null 2>&1 && wasm-pack --version | grep -q "wasm-pack $version"; then
  wasm-pack --version
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
    echo "unsupported wasm-pack install target: $(uname -s)-$(uname -m)" >&2
    exit 1
    ;;
esac

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

archive="$tmpdir/wasm-pack.tar.gz"
url="https://github.com/rustwasm/wasm-pack/releases/download/v${version}/wasm-pack-v${version}-${target}.tar.gz"

curl \
  --fail \
  --location \
  --show-error \
  --silent \
  --retry 5 \
  --retry-all-errors \
  --retry-delay 5 \
  --connect-timeout 30 \
  --output "$archive" \
  "$url"

tar -xzf "$archive" -C "$tmpdir"
mkdir -p "$install_dir"
install "$tmpdir/wasm-pack-v${version}-${target}/wasm-pack" "$install_dir/wasm-pack"

wasm-pack --version
