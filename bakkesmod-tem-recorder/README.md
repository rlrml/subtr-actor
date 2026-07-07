# tem-recorder BakkesMod plugin

Capture shots from an in-game Rocket League replay into a custom training
pack and save it as an encrypted `.Tem` file the game can load.

While watching a replay, press a keybind (or run the notifier) to snapshot
the current frame's ball position/velocity and car positions/rotations as a
new shot in an in-memory pack; save the pack when you are done. Pack
serialization is done by the [`subtr-actor-training`](../crates/subtr-actor-training)
crate through the thin C ABI crate in [`rust/`](./rust).

This plugin is separate from the [`bakkesmod/`](../bakkesmod) live-analysis
plugin and can be installed independently.

## Build

From a Windows machine with Rust, CMake, and Visual Studio 2022:

```powershell
.\bakkesmod-tem-recorder\build-windows.ps1
```

On Linux, the same DLLs cross-compile with clang-cl + lld-link against an
xwin MSVC sysroot (mirroring the `bakkesmod/` live plugin's build):

```sh
nix build .#bakkesmod-tem-recorder   # hermetic; artifacts in ./result
# or, inside `nix develop .#bakkesmod`:
bakkesmod-tem-recorder/build-linux-msvc.sh
```

Built artifacts can be checked with
`python3 bakkesmod-tem-recorder/verify-dll-exports.py --rust-dll <tem_recorder.dll> --plugin-dll <TemRecorderPlugin.dll>`
(also run by CI).

Either path builds two DLLs and prepares an install layout under
`.../Release/bakkesmod-install/`:

| File | Destination under `%APPDATA%\bakkesmod\bakkesmod\` |
| --- | --- |
| `TemRecorderPlugin.dll` (C++ plugin) | `plugins\TemRecorderPlugin.dll` |
| `tem_recorder.dll` (Rust ABI) | `data\tem-recorder\tem_recorder.dll` |

Run with `-Install` to copy both into a local BakkesMod installation
(add `-EnableAutoload` to append `plugin load TemRecorderPlugin` to
`cfg\plugins.cfg`), or load manually from the BakkesMod console:

```
plugin load TemRecorderPlugin
```

The plugin loads `tem_recorder.dll` at runtime (next to the plugin DLL,
then `data\tem-recorder\`); if the DLL is missing the plugin stays loaded
but capture is disabled.

## Usage

Notifiers (bindable console commands):

| Notifier | Effect |
| --- | --- |
| `tem_recorder_capture_shot` | Capture the current replay frame as a new shot |
| `tem_recorder_save_pack` | Save the in-memory pack as `<GUID>.Tem` |
| `tem_recorder_new_pack` | Start a fresh pack (new GUID) |
| `tem_recorder_open_pack <path>` | Open an existing `.Tem` to append shots to |

Suggested binding, so a single key captures a shot while scrubbing a replay:

```
bind F7 tem_recorder_capture_shot
bind F8 tem_recorder_save_pack
```

Cvars: `tem_recorder_pack_name`, `tem_recorder_creator_name`,
`tem_recorder_time_limit` (seconds per shot, default 8), and
`tem_recorder_output_dir`.

A settings page under **F2 > Plugins > tem-recorder** provides the same
controls plus the captured-shot list (with per-shot Remove buttons) and a
status/error line.

## Where packs land

With `tem_recorder_output_dir` unset, packs save to
`%USERPROFILE%\Documents\My Games\Rocket League\TAGame\Training\<GUID>.Tem`.

**Caveat:** the game actually lists local training packs from a per-account
subfolder, `Training\<online-id>\MyTraining\`. Set
`tem_recorder_output_dir` to that folder (it exists once you have saved any
pack from the in-game editor) to have recorded packs show up under
Training > Custom Training.

## What a captured shot contains

The `.tem` archetype format stores: ball location + initial velocity
(direction/speed), and one player car location + rotation. Car velocity,
boost, and ball/car angular velocity are **not representable** in the
current format; the plugin still captures them across the ABI so they can
be serialized once the format work lands (see below). Only the primary car
(the replay camera's view target, falling back to the first car) is written
into the round.

## Phase-3 integration

Archetype strings are currently hand-built in
[`rust/src/archetypes.rs`](./rust/src/archetypes.rs), which is the **single
replacement point**: when the typed `BallSpawn`/`CarSpawn` constructors
land in `subtr-actor-training`, swap the string building there and delete
the local formatting helpers. All unit-conversion assumptions and dropped
fields are marked with `TODO(phase-3)` comments in that module and in
`plugin/capture.cpp`.
