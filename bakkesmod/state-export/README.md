# state-export BakkesMod plugin

Export live Rocket League game state over a local WebSocket server, with no
in-game analysis: sampled frames (ball/car rigid bodies, controller input,
camera state, boost, match stats, platform identity) and explicit game events
(touches, boost pads, goals, shots/saves/assists, demolitions) are converted
to the owned [`subtr-actor-live`](../../crates/subtr-actor-live) model inside
`state_export.dll` and broadcast on the `subtr-actor-live` wire protocol.
Consumers run the stats graph (or anything else) out of process.

This plugin is separate from the [`bakkesmod/subtr-actor/`](../subtr-actor)
live-analysis plugin and the
[`bakkesmod/replay-to-training/`](../replay-to-training) capture plugin, and
can be installed independently.

## Wire protocol (brief)

- WebSocket server, default port **49109**
  (`SE_DEFAULT_STATE_EXPORT_PORT` / `DEFAULT_STATE_EXPORT_PORT`), bound to
  `127.0.0.1` unless `state_export_bind_all_interfaces` is set.
- Clients handshake with a JSON `Hello` text frame negotiating the encoding;
  **postcard** (compact binary) is the intended default. `GET /?format=json`
  subscribes without any Hello for zero-code JSON consumers (`websocat`,
  browsers).
- New subscribers receive a **snapshot on connect** (match meta, roster, and
  event history so far), then live frames; a slow client that overflows its
  queue is disconnected and reconnects into a fresh snapshot.
- Protocol versioning, message shapes, and the consumer-side view live in
  [`crates/subtr-actor-live`](../../crates/subtr-actor-live) and
  [`crates/subtr-actor-live-consumer`](../../crates/subtr-actor-live-consumer).

### Consumer quickstart

With the plugin loaded and a match running:

```sh
cargo run -p subtr-actor-live-consumer --example event_timeline_stream -- --url ws://<host>:49109
```

## Cvars and notifiers

| Cvar | Default | Effect |
| --- | --- | --- |
| `state_export_enabled` | `1` | Enable live game-state export |
| `state_export_port` | `49109` | Server TCP port (0 = ephemeral); applied on server restart |
| `state_export_bind_all_interfaces` | `0` | Bind `0.0.0.0` instead of `127.0.0.1` (see security note) |
| `state_export_sample_interval_ms` | `8` | Minimum elapsed game time between frame samples (clamped 1–1000) |
| `state_export_sample_when_no_clients` | `0` | Keep sampling with zero connected clients |

| Notifier | Effect |
| --- | --- |
| `state_export_restart_server` | Restart the server with the current port/bind cvars (same as the settings-page Apply) |
| `state_export_status` | Log server state, port, client count, frames sent/dropped, and both build ids |

The **F2 > Plugins > state-export** settings page exposes the same controls
plus a live status line, the last engine error, and the Rust core build id.

## Performance design

- All FFI calls are cheap and game-thread-safe; the WebSocket server runs on
  its own threads inside `state_export.dll`.
- `state_export_status` is atomics-only, so the tick polls it every pass and
  **gates sampling on `client_count > 0`** — an idle server costs one status
  read per tick and nothing else (opt out with
  `state_export_sample_when_no_clients`).
- Frame sampling reuses per-tick scratch buffers (no steady-state heap churn)
  and is rate-limited by game time via `state_export_sample_interval_ms`.
- `state_export_push_frame` never blocks beyond a short mutex hold: the
  bounded ingest queue **drops the oldest frame** on overflow and coalesces
  its explicit events forward, so backpressure can never stall the game
  thread (drops are visible as `frames_dropped` in the status).
- Match lifecycle: entering a game pushes the match context (match GUID, map
  name, playlist id); leaving the game or the `EventMatchEnded` hook
  broadcasts `MatchEnd` and resets the stream for the next match.

## Security note

`state_export_bind_all_interfaces` exposes raw live game state (including
player platform ids) to every host that can reach this machine on the LAN or
a VPN, with no authentication. Leave it off (loopback-only) unless remote
consumers need the stream, and prefer a tunnel (SSH, tailscale serve) over
binding all interfaces on untrusted networks.

## Build

From a Windows machine with Rust, CMake, and Visual Studio 2022:

```powershell
.\bakkesmod\state-export\build-windows.ps1
```

On Linux, the same DLLs cross-compile with clang-cl + lld-link against an
xwin MSVC sysroot (mirroring the other two plugins' builds):

```sh
nix build .#bakkesmod-state-export   # hermetic; artifacts in ./result
# or, inside `nix develop .#bakkesmod`:
bakkesmod/state-export/build-linux-msvc.sh
```

Built artifacts can be checked with
`python3 bakkesmod/state-export/verify-dll-exports.py --rust-dll <state_export.dll> --plugin-dll <StateExportPlugin.dll>`
(also run by CI).

Either path builds two DLLs and prepares an install layout under
`.../Release/bakkesmod-install/`:

| File | Destination under `%APPDATA%\bakkesmod\bakkesmod\` |
| --- | --- |
| `StateExportPlugin.dll` (C++ plugin) | `plugins\StateExportPlugin.dll` |
| `state_export.dll` (Rust ABI) | `data\state-export\state_export.dll` |

Run with `-Install` to copy both into a local BakkesMod installation
(add `-EnableAutoload` to append `plugin load StateExportPlugin` to
`cfg\plugins.cfg`), or load manually from the BakkesMod console:

```
plugin load StateExportPlugin
```

The plugin loads `state_export.dll` at runtime (next to the plugin DLL, then
`data\state-export\`); if the DLL is missing the plugin stays loaded but
export is disabled.

## SDK availability notes (degraded fields)

Fields the ABI carries but the BakkesMod SDK cannot populate are sent with
their `has_` flag zeroed rather than guessed:

- **`dodge_impulse`** — the SDK's `DodgeComponentWrapper` exposes the dodge
  torque vector (`GetDodgeTorque()`, which the plugin samples) but no
  applied-impulse vector getter (`GetDodgeImpulse2` computes a hypothetical
  from a caller-supplied direction), so `has_dodge_impulse` is always 0.
- **PsyNet / PlayStation identities** — the SDK reports the platform, but
  those ids carry opaque payloads boxcars cannot reconstruct, so they cross
  the ABI tagged and the Rust side maps them to "no identity" (falling back
  to `SplitScreen(player_index)`), per the table in
  [`rust/include/state_export.h`](./rust/include/state_export.h).
- **`live_play`** — like the subtr-actor live plugin, live sampling has no
  replay-style `live_play` bit; `has_live_play` is 0 and consumers derive
  play state from `game_state` / `ball_has_been_hit`.

## Build identification

Both DLLs embed the git hash, dirty flag, and commit date of the build:
CMake injects them into the C++ plugin as compile definitions, and
`rust/build.rs` embeds the same values into the Rust core. Each is derived
from git at build time, overridable via the `STATE_EXPORT_GIT_HASH` /
`STATE_EXPORT_GIT_DIRTY` / `STATE_EXPORT_COMMIT_DATE` environment variables
(which the nix build exports from the flake's source metadata, since its
sandbox has no `.git`). `state_export_status` (also logged once on load)
prints both identifiers; a hash mismatch between the two lines means the
installed `StateExportPlugin.dll` and `state_export.dll` come from different
builds.
