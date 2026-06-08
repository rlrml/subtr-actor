#!/usr/bin/env bash
set -euo pipefail

readonly wasm_pack_version="0.13.1"
readonly installer_url="https://rustwasm.github.io/wasm-pack/installer/init.sh"
readonly installer_path="${RUNNER_TEMP:-/tmp}/wasm-pack-init.sh"

for attempt in 1 2 3; do
  echo "Installing wasm-pack from binary release (attempt ${attempt}/3)"

  if curl -fsSL "${installer_url}" -o "${installer_path}" && sh "${installer_path}"; then
    wasm-pack --version
    exit 0
  fi

  if [[ "${attempt}" == "3" ]]; then
    break
  fi

  sleep_seconds=$((attempt * 10))
  echo "wasm-pack binary install failed; retrying in ${sleep_seconds}s"
  sleep "${sleep_seconds}"
done

for attempt in 1 2; do
  echo "Installing wasm-pack from crates.io (attempt ${attempt}/2)"

  if cargo install wasm-pack --version "${wasm_pack_version}" --locked; then
    wasm-pack --version
    exit 0
  fi

  if [[ "${attempt}" == "2" ]]; then
    break
  fi

  sleep_seconds=$((attempt * 10))
  echo "wasm-pack cargo install failed; retrying in ${sleep_seconds}s"
  sleep "${sleep_seconds}"
done

echo "Failed to install wasm-pack" >&2
exit 1
