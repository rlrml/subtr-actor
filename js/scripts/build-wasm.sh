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

# wasm-bindgen (unlike wasm-pack) emits no package.json; write one so js/pkg is
# installable as a `file:../pkg` dependency (node/tsx resolution, not just the
# vite alias).
cat > "$js_dir/pkg/package.json" <<'EOF'
{
  "name": "@rlrml/subtr-actor",
  "version": "0.0.0-local",
  "description": "Local wasm-bindgen build of the subtr-actor WASM bindings (js/scripts/build-wasm.sh)",
  "type": "module",
  "main": "rl_replay_subtr_actor.js",
  "module": "rl_replay_subtr_actor.js",
  "types": "rl_replay_subtr_actor.d.ts",
  "exports": {
    ".": {
      "types": "./rl_replay_subtr_actor.d.ts",
      "import": "./rl_replay_subtr_actor.js"
    },
    "./rl_replay_subtr_actor_bg.wasm": "./rl_replay_subtr_actor_bg.wasm"
  },
  "files": [
    "rl_replay_subtr_actor.js",
    "rl_replay_subtr_actor.d.ts",
    "rl_replay_subtr_actor_bg.wasm",
    "rl_replay_subtr_actor_bg.wasm.d.ts"
  ],
  "sideEffects": false
}
EOF
