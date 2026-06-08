#!/usr/bin/env bash
set -euo pipefail

readonly installer_url="https://rustwasm.github.io/wasm-pack/installer/init.sh"
readonly installer_path="${RUNNER_TEMP:-/tmp}/wasm-pack-init.sh"

for attempt in 1 2 3 4 5; do
  echo "Installing wasm-pack (attempt ${attempt}/5)"

  if curl -fsSL "${installer_url}" -o "${installer_path}" && sh "${installer_path}"; then
    wasm-pack --version
    exit 0
  fi

  if [[ "${attempt}" == "5" ]]; then
    break
  fi

  sleep_seconds=$((attempt * 10))
  echo "wasm-pack install failed; retrying in ${sleep_seconds}s"
  sleep "${sleep_seconds}"
done

echo "Failed to install wasm-pack after 5 attempts" >&2
exit 1
