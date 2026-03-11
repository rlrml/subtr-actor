#!/usr/bin/env bash

set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
js_dir="$(cd "$script_dir/.." && pwd)"
repo_root="$(cd "$js_dir/.." && pwd)"

cargo_bin="$(rustup which cargo)"
rustc_bin="$(rustup which rustc)"
wasm_file="$repo_root/target/wasm32-unknown-unknown/release/rl_replay_subtr_actor.wasm"

RUSTC="$rustc_bin" "$cargo_bin" build \
  --manifest-path "$js_dir/Cargo.toml" \
  --target wasm32-unknown-unknown \
  --release

nix run nixpkgs#wasm-bindgen-cli -- \
  "$wasm_file" \
  --out-dir "$js_dir/pkg" \
  --target web
