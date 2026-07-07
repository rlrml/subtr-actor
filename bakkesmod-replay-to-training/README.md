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

Notifiers (bindable console commands — all zero-arg by design except
`open_pack`/`target`; every tunable lives in a persisted cvar edited in the
capture window):

| Notifier | Effect |
| --- | --- |
| `replay_to_training_capture_shot` | Capture the current replay frame as an OFFENSIVE (striker) shot |
| `replay_to_training_capture_save` | Capture the current replay frame as a DEFENSIVE (goalie) save |
| `replay_to_training_save_pack` | Save the in-memory pack as `<GUID>.Tem` |
| `replay_to_training_new_pack` | Start a fresh pack (new GUID, pack type unset) |
| `replay_to_training_open_pack <path>` | Open an existing `.Tem` to append shots to |
| `replay_to_training_target [<name>]` | Set (or show) the persistent default-save target, e.g. `MyTraining\<name>` |
| `replay_to_training_list_targets` | List local `.Tem` targets under `MyTraining\` and `Downloaded\` |
| `replay_to_training_window` | Toggle the standalone capture window (same as `togglemenu replaytotraining`) |
| `replay_to_training_version` | Log the loaded plugin and Rust core build identifiers |

Suggested binding, so single keys capture shots/saves while scrubbing a
replay:

```
bind F6 replay_to_training_capture_save
bind F7 replay_to_training_capture_shot
bind F8 replay_to_training_save_pack
bind F9 replay_to_training_window
```

Cvars (all persisted): `replay_to_training_pack_name`,
`replay_to_training_creator_name`, `replay_to_training_time_limit` (seconds
per shot, default 8), `replay_to_training_mirror_by_team` (default on; see
below), `replay_to_training_capture_momentum` (default on; see below),
`replay_to_training_autosave` (default on),
`replay_to_training_output_dir`, and `replay_to_training_target_save_name`
(persisted target).

## Capture window

`togglemenu replaytotraining` (or `replay_to_training_window`, bindable)
opens a standalone in-game window — a compact one-stop capture HUD usable
while watching a replay. It contains: the pack type display + manual
override dropdown, the per-shot time limit, the mirror-by-team /
capture-momentum / autosave toggles, pack name / creator fields, the active
target display with set/clear and the discovered-target picker, capture
shot / capture save / save / new-pack buttons, the captured-shot list with
per-shot Remove, and the status line. Every control is backed by its
persisted cvar. The **F2 > Plugins > replay-to-training** settings page
keeps the same controls (plus the output-directory and open-path fields)
for parity.

## Striker vs goalie captures and pack type

The `.tem` format tags the training type (`ETrainingType`) at the PACK
level — rounds cannot carry their own type. The plugin therefore expresses
the mode through which zero-arg capture command is used:

- A fresh pack's type is **unset**; the first capture assigns it
  (`capture_shot` → Striker, `capture_save` → Goalie).
- Later captures whose mode conflicts with the assigned type (a save into a
  Striker pack or vice versa) are still recorded — the format cannot
  distinguish them — but the status line WARNS.
- The capture window's dropdown can override the type manually, including
  Aerial and None for publishing metadata.
- `replay_to_training_new_pack` resets the type to unset.

## Auto-mirroring by team (`replay_to_training_mirror_by_team`, default on)

Training scenarios live in a fixed field frame. Derived from the decoded
Psyonix striker pack in the corpus ("Diamond Pack May 2023", 9 rounds):
striker scenarios attack the **+Y** goal (ball spawns bias toward +Y — 7/9
rounds, deepest +4502uu vs the +Y goal line at 5120uu — with the car placed
on the −Y side of the ball in 8/9 rounds, and near-goal serves aimed
straight at +Y). With the standard replay convention that blue / team 0
defends −Y and attacks +Y, the training player is blue-oriented, so
captures where the spectated player is on ORANGE are mirrored 180° about
field center (X/Y locations negated, yaws + half turn, ball velocity yaw
flipped with pitch preserved) — the whole scenario flips together, so an
orange breakaway becomes the same breakaway attacking the training goal.

No goalie pack exists in the corpus, so save captures use the natural
choice that the training player occupies the same field end in both modes
(goalie defends the −Y goal). **TODO(in-game): validate the goalie
orientation** — if the game expects goalie packs mirrored the other way,
only `should_mirror` in `rust/src/mirror.rs` needs to flip.

## Car momentum capture (`replay_to_training_capture_momentum`, default on)

The spawn-point mesh's `VelocityStartSpeed` field is the editor's
car-start-speed feature (a facing plus a scalar speed; `0.0` everywhere in
the corpus). With the toggle on, the plugin writes the captured car's
forward speed — its velocity projected onto its facing — so the training
car starts moving like it did in the replay. Lateral drift and reverse
motion are not representable and clamp to `0.0`. Starting boost is
TODO(in-game), pending discovery of the archetype key from a user-authored
editor pack.

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
