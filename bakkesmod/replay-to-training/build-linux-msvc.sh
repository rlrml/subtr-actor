#!/usr/bin/env bash
# Linux MSVC-ABI cross build of the replay-to-training plugin (clang-cl +
# lld-link + xwin sysroot), mirroring bakkesmod/subtr-actor/build-linux-msvc.sh
# and reusing the shared CMake toolchain file at bakkesmod/toolchains/.
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "$script_dir/../.." && pwd)"
toolchain_file="$repo_root/bakkesmod/toolchains/linux-msvc-clang.cmake"

build_dir="${BUILD_DIR:-$script_dir/build-linux-msvc}"
configuration="${CONFIGURATION:-Release}"
xwin_cache_dir="${XWIN_CACHE_DIR:-$repo_root/.xwin-cache}"
xwin_sysroot="${XWIN_SYSROOT:-$repo_root/.xwin-msvc}"
target="x86_64-pc-windows-msvc"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

require_command cargo
require_command cmake
require_command ninja
require_command python3
require_command clang-cl
require_command lld-link
require_command llvm-lib

if [[ ! -d "$xwin_sysroot/VC/Tools/MSVC" || ! -d "$xwin_sysroot/Windows Kits/10" ]]; then
  require_command xwin
  xwin \
    --accept-license \
    --cache-dir "$xwin_cache_dir" \
    --arch x86_64 \
    --variant desktop \
    splat \
    --output "$xwin_sysroot" \
    --use-winsysroot-style \
    --preserve-ms-arch-notation \
    --copy
fi

vc_tools_dir="$(find "$xwin_sysroot/VC/Tools/MSVC" -mindepth 1 -maxdepth 1 -type d | sort -V | tail -n 1)"
windows_sdk_lib_dir="$(find "$xwin_sysroot/Windows Kits/10/Lib" -mindepth 1 -maxdepth 1 -type d | sort -V | tail -n 1)"
vc_lib_dir="$vc_tools_dir/lib/x64"
ucrt_lib_dir="$windows_sdk_lib_dir/ucrt/x64"
um_lib_dir="$windows_sdk_lib_dir/um/x64"

if [[ -z "${BAKKESMOD_SDK_DIR:-}" && -n "${BAKKESMODSDK_DIR:-}" ]]; then
  export BAKKESMOD_SDK_DIR="$BAKKESMODSDK_DIR"
fi

if [[ -z "${BAKKESMOD_SDK_DIR:-}" ]]; then
  echo "BAKKESMOD_SDK_DIR is not set; CMake will fetch the pinned SDK if needed." >&2
else
  sdk_overlay_dir="$build_dir/bakkesmod-sdk-case-overlay"
  python3 - "$BAKKESMOD_SDK_DIR" "$sdk_overlay_dir" "$script_dir" <<'PY'
import re
import shutil
import os
import sys
from pathlib import Path

source = Path(sys.argv[1])
dest = Path(sys.argv[2])
plugin_dir = Path(sys.argv[3])

def make_writable(function, path, _error):
    Path(path).chmod(Path(path).stat().st_mode | 0o700)
    function(path)

if dest.exists():
    shutil.rmtree(dest, onerror=make_writable)
shutil.copytree(source, dest, symlinks=True)
for item in dest.rglob("*"):
    item.chmod(item.stat().st_mode | 0o700)
dest.chmod(dest.stat().st_mode | 0o700)

include_root = dest / "include"
include_pattern = re.compile(r'^\s*#\s*include\s+[<"]([^">]+)[">]', re.MULTILINE)
scan_roots = [include_root, plugin_dir]
skip_dirs = {"build", "build-linux-msvc", "__pycache__"}

for directory in sorted((item for item in dest.rglob("*") if item.is_dir()), key=lambda item: len(item.parts)):
    for child in list(directory.iterdir()):
        alias = directory / child.name.lower()
        if alias == child or alias.exists():
            continue
        alias.symlink_to(child.name, target_is_directory=child.is_dir())


def resolve_case_insensitive(root: Path, relative: str) -> Path | None:
    current = root
    for part in Path(relative).parts:
        exact = current / part
        if exact.exists():
            current = exact
            continue
        if not current.is_dir():
            return None
        matches = [child for child in current.iterdir() if child.name.lower() == part.lower()]
        if not matches:
            return None
        current = matches[0]
    return current


def source_files(root: Path):
    for path in root.rglob("*"):
        if any(part in skip_dirs for part in path.relative_to(root).parts):
            continue
        if path.suffix.lower() in {".h", ".hpp", ".hh", ".cpp", ".cxx", ".cc"}:
            yield path


created = True
while created:
    created = False
    for root in scan_roots:
        for path in source_files(root):
            text = path.read_text(encoding="utf-8", errors="ignore")
            for include in include_pattern.findall(text):
                for base in (path.parent, include_root):
                    requested = base / include
                    if requested.exists():
                        continue
                    resolved = resolve_case_insensitive(base, include)
                    if resolved is None or not resolved.exists():
                        continue
                    requested.parent.mkdir(parents=True, exist_ok=True)
                    requested.symlink_to(Path(os.path.relpath(resolved, requested.parent)))
                    created = True
PY
  export BAKKESMOD_SDK_DIR="$sdk_overlay_dir"
fi

export XWIN_SYSROOT="$xwin_sysroot"
export CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER="${CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER:-lld-link}"
unit_separator=$'\x1f'
rust_link_flags=(
  "-Clink-arg=/libpath:$vc_lib_dir"
  "-Clink-arg=/libpath:$ucrt_lib_dir"
  "-Clink-arg=/libpath:$um_lib_dir"
)
encoded_link_flags="$(IFS="$unit_separator"; echo "${rust_link_flags[*]}")"
if [[ -n "${CARGO_ENCODED_RUSTFLAGS:-}" ]]; then
  export CARGO_ENCODED_RUSTFLAGS="${CARGO_ENCODED_RUSTFLAGS}${unit_separator}${encoded_link_flags}"
else
  export CARGO_ENCODED_RUSTFLAGS="$encoded_link_flags"
fi

pushd "$repo_root" >/dev/null
cargo build -p subtr-actor-replay-to-training --release --target "$target"
popd >/dev/null

cmake_args=(
  -S "$script_dir"
  -B "$build_dir"
  -G Ninja
  -DCMAKE_BUILD_TYPE="$configuration"
  -DCMAKE_TOOLCHAIN_FILE="$toolchain_file"
  -DXWIN_SYSROOT="$xwin_sysroot"
  -DXWIN_VC_LIB_DIR="$vc_lib_dir"
  -DXWIN_UCRT_LIB_DIR="$ucrt_lib_dir"
  -DXWIN_UM_LIB_DIR="$um_lib_dir"
)

if [[ -n "${BAKKESMOD_SDK_DIR:-}" ]]; then
  cmake_args+=("-DBAKKESMOD_SDK_DIR=$BAKKESMOD_SDK_DIR")
fi

cmake "${cmake_args[@]}"
cmake --build "$build_dir" --config "$configuration"

plugin_out_dir="$build_dir/$configuration"
mkdir -p "$plugin_out_dir"
if [[ -f "$build_dir/ReplayToTrainingPlugin.dll" ]]; then
  cp -f "$build_dir/ReplayToTrainingPlugin.dll" "$plugin_out_dir/ReplayToTrainingPlugin.dll"
elif [[ ! -f "$plugin_out_dir/ReplayToTrainingPlugin.dll" ]]; then
  echo "missing built plugin DLL: $build_dir/ReplayToTrainingPlugin.dll" >&2
  exit 1
fi
cp \
  "$repo_root/target/$target/release/replay_to_training.dll" \
  "$plugin_out_dir/replay_to_training.dll"

install_layout_dir="$plugin_out_dir/bakkesmod-install"
mkdir -p "$install_layout_dir/plugins" "$install_layout_dir/data/replay-to-training"
cp -f \
  "$plugin_out_dir/ReplayToTrainingPlugin.dll" \
  "$install_layout_dir/plugins/ReplayToTrainingPlugin.dll"
cp -f \
  "$plugin_out_dir/replay_to_training.dll" \
  "$install_layout_dir/data/replay-to-training/replay_to_training.dll"

echo "Built Linux MSVC-ABI replay-to-training artifacts in $plugin_out_dir"
echo "Prepared install layout in $install_layout_dir"
