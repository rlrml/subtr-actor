# replay-to-training BakkesMod plugin

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
.\bakkesmod-replay-to-training\build-windows.ps1
```

On Linux, the same DLLs cross-compile with clang-cl + lld-link against an
xwin MSVC sysroot (mirroring the `bakkesmod/` live plugin's build):

```sh
nix build .#bakkesmod-replay-to-training   # hermetic; artifacts in ./result
# or, inside `nix develop .#bakkesmod`:
bakkesmod-replay-to-training/build-linux-msvc.sh
```

Built artifacts can be checked with
`python3 bakkesmod-replay-to-training/verify-dll-exports.py --rust-dll <replay_to_training.dll> --plugin-dll <ReplayToTrainingPlugin.dll>`
(also run by CI).

Either path builds two DLLs and prepares an install layout under
`.../Release/bakkesmod-install/`:

| File | Destination under `%APPDATA%\bakkesmod\bakkesmod\` |
| --- | --- |
| `ReplayToTrainingPlugin.dll` (C++ plugin) | `plugins\ReplayToTrainingPlugin.dll` |
| `replay_to_training.dll` (Rust ABI) | `data\replay-to-training\replay_to_training.dll` |

Run with `-Install` to copy both into a local BakkesMod installation
(add `-EnableAutoload` to append `plugin load ReplayToTrainingPlugin` to
`cfg\plugins.cfg`), or load manually from the BakkesMod console:

```
plugin load ReplayToTrainingPlugin
```

The plugin loads `replay_to_training.dll` at runtime (next to the plugin DLL,
then `data\replay-to-training\`); if the DLL is missing the plugin stays loaded
but capture is disabled.

## Usage

Notifiers (bindable console commands):

| Notifier | Effect |
| --- | --- |
| `replay_to_training_capture_shot` | Capture the current replay frame as a new shot |
| `replay_to_training_save_pack` | Save the in-memory pack as `<GUID>.Tem` |
| `replay_to_training_new_pack` | Start a fresh pack (new GUID) |
| `replay_to_training_open_pack <path>` | Open an existing `.Tem` to append shots to |
| `replay_to_training_target [<name>]` | Set (or show) the persistent default-save target, e.g. `MyTraining\<name>` |
| `replay_to_training_list_targets` | List local `.Tem` targets under `MyTraining\` and `Downloaded\` |
| `replay_to_training_version` | Log the loaded plugin and Rust core build identifiers |

Suggested binding, so a single key captures a shot while scrubbing a replay:

```
bind F7 replay_to_training_capture_shot
bind F8 replay_to_training_save_pack
```

Cvars: `replay_to_training_pack_name`, `replay_to_training_creator_name`,
`replay_to_training_time_limit` (seconds per shot, default 8),
`replay_to_training_output_dir`, and `replay_to_training_target_save_name`
(persisted target).

A settings page under **F2 > Plugins > replay-to-training** provides the same
controls plus the captured-shot list (with per-shot Remove buttons), the
target field / discovered-target picker, and a status/error line.

## Target (persistent default save)

Set a **target** to accumulate captures into one specific custom-training
pack across sessions:

```
replay_to_training_target MyTraining\MyRecordedShots
```

Setting a target **opens that `.Tem` into memory** (its existing rounds show
up in the shot list), so subsequent captures **append** to it. On save the
in-memory pack is written back to the target path. Because memory is the
single source of truth — the file is read in once when the target is set,
never merged again at the I/O layer — this is inherently **non-destructive**:
the original rounds are preserved and new captures are added, with no
double-counting. A name is normalized (backslashes, a stripped `.tem`/`.Tem`
suffix); a bare stem defaults into `MyTraining\`, which is where the game
lists locally created packs. `replay_to_training_list_targets` scans
`MyTraining\` and `Downloaded\` under the training root
(`replay_to_training_output_dir`, or the default `TAGame\Training`).

Guardrails against clobbering a target:

- `replay_to_training_new_pack` clears the active target and reverts to the
  auto `<GUID>.Tem` flow, so a fresh pack can never overwrite a target file.
- Before overwriting an existing target file, the plugin compares the
  on-disk pack's GUID with the in-memory pack's. If they differ **and** the
  current pack was not loaded from that path, the save is **refused**
  (`target already contains a different pack; open/target it first to
  append`) and nothing is written. Target/open a file to append to it.

With no target set, save falls back to writing an auto `<GUID>.Tem` in
`replay_to_training_output_dir` (unchanged).

## Build identification

Both DLLs embed the git hash, dirty flag, and commit date of the build:
CMake injects them into the C++ plugin as compile definitions, and
`rust/build.rs` embeds the same values into the Rust core. Each is derived
from git at build time, overridable via the `REPLAY_TO_TRAINING_GIT_HASH` /
`REPLAY_TO_TRAINING_GIT_DIRTY` / `REPLAY_TO_TRAINING_COMMIT_DATE`
environment variables (which the nix build exports from the flake's source
metadata, since its sandbox has no `.git`). `replay_to_training_version`
(also logged once on load) prints both identifiers, e.g.:

```
replay-to-training plugin 0.1.0 build=20eb058 dirty=0 commit_date=2026-07-05T18:44:32-07:00
rust core: replay_to_training 1.1.0 build=20eb058 dirty=0 commit_date=2026-07-05T18:44:32-07:00
```

A hash mismatch between the two lines means the installed
`ReplayToTrainingPlugin.dll` and `replay_to_training.dll` come from
different builds.

## Where packs land

With no target set and `replay_to_training_output_dir` unset, packs save to
`%USERPROFILE%\Documents\My Games\Rocket League\TAGame\Training\<GUID>.Tem`.

To have recorded packs show up under **Training > Custom Training**, prefer
setting a target (`replay_to_training_target MyTraining\<name>`): targets
resolve into `MyTraining\` under the training root, which is exactly the
folder the game lists. Point `replay_to_training_output_dir` at the training
root if the game uses a non-default profile path (older builds nested it
under a per-account `<online-id>\` folder).

## What a captured shot contains

The `.tem` archetype format stores: ball location + initial velocity
(direction/speed), and one player car location + rotation. Car velocity,
boost, and ball/car angular velocity are **not representable** in the
format (confirmed by the typed archetype structs in
`subtr-actor-training`); the plugin still captures them across the ABI so
nothing needs to change if the format ever grows those fields. Only the
primary car (the replay camera's view target, falling back to the first
car) is written into the round.

## Archetype serialization

Archetype strings are built through the typed
`Archetype`/`BallSpawn`/`CarSpawn`/`PlayerCarSpawn` constructors from
`subtr-actor-training`, which own the corpus-matching formatting (fixed key
order, four-decimal floats, integer rotator units).
[`rust/src/archetypes.rs`](./rust/src/archetypes.rs) only maps captured
ABI state onto those structs — including the velocity-vector →
rotator+speed conversion for the ball. Behaviors that still need in-game
validation are marked with `TODO(in-game)` comments in that module and in
`plugin/capture.cpp`.
