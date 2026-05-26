# subtr-actor BakkesMod plugin spike

This is an early BakkesMod integration spike. It is intentionally split into:

- `crates/subtr-actor-bakkesmod`: Rust C ABI that accepts sampled live frames,
  adapts them through a `ProcessorView`, evaluates the shared `subtr-actor`
  analysis graph, drains normalized player and team events for overlay use, and
  exposes the live graph metadata, timeline, event bundle, graph-backed stats
  modules, and current frame stats snapshot as JSON.
- `bakkesmod/SubtrActorPlugin.*`: C++ BakkesMod plugin shell that samples active
  cars and the ball, calls the Rust ABI, and renders short on-screen labels.
  The `subtr_actor_dump_graph` console command writes the current graph
  metadata, full timeline payload, event bundle, graph-backed stats modules, all
  analysis-node outputs, and current frame stats snapshot to BakkesMod's
  `data/subtr-actor` directory as JSON.
  The `subtr_actor_dump_stats_module <module_name> [finish]` console command
  writes one graph-backed builtin stats module by name, using the module names
  reported in `graph-info.json`.
  The `subtr_actor_dump_stats_module_frame <module_name> [finish]` and
  `subtr_actor_dump_stats_module_config <module_name> [finish]` commands write
  the same module-keyed frame snapshot and config surfaces by name.
  The `subtr_actor_dump_graph_output <output_name> [finish]` console command
  writes one named graph output (`events`, `frame`, `timeline`, `stats`,
  `analysis_nodes`, `event_history`, or `graph_info`) using the output names
  reported in `graph-info.json`.
  The `subtr_actor_dump_analysis_node <node_name> [finish]` console command
  writes one callable analysis node by name, using the callable node-name
  registry exposed by the Rust ABI.
  The `subtr_actor_verify_graph [finish]` console command calls the fixed graph
  outputs plus every callable analysis node name from the loaded plugin
  runtime and logs byte sizes, giving a quick in-game check that the graph
  surface is callable after at least one live frame has been processed.
  The `subtr_actor_self_test_graph` console command creates a temporary Rust
  engine, feeds a synthetic live sample containing every required event family,
  and runs strict graph verification without disturbing the current live engine.

The current spike feeds active cars from BakkesMod's server car list, falling
back to the local car when that list is unavailable. That is enough to test
mechanics whose first pass can work from live kinematics, explicit BakkesMod
events, dodge-refresh transitions, and control state:

- speed flip
- half flip
- wavedash
- the normalized analysis-graph mechanics/events available through
  `subtr_actor_bakkesmod_events_json_len`,
  `subtr_actor_bakkesmod_write_events_json`,
  `subtr_actor_bakkesmod_frame_json_len`, and
  `subtr_actor_bakkesmod_write_frame_json`
- player-owned, team-owned, and goal-context drainable overlay events through
  `subtr_actor_bakkesmod_drain_events`,
  `subtr_actor_bakkesmod_drain_team_events`, and
  `subtr_actor_bakkesmod_drain_goal_context_events`
- the full live `ReplayStatsTimeline` payload through
  `subtr_actor_bakkesmod_timeline_json_len` and
  `subtr_actor_bakkesmod_write_timeline_json`
- the shared `StatsCollector` module-keyed stats surface through
  `subtr_actor_bakkesmod_stats_json_len` and
  `subtr_actor_bakkesmod_write_stats_json`
- the full callable analysis-node output map through the `analysis_nodes` named
  graph output, using `subtr_actor_bakkesmod_graph_output_json_len` and
  `subtr_actor_bakkesmod_write_graph_output_json`
- cumulative raw live event-family history through the `event_history` named
  graph output, using `subtr_actor_bakkesmod_graph_output_json_len` and
  `subtr_actor_bakkesmod_write_graph_output_json`
- the resolved graph DAG, callable node registry, builtin node registry, stats
  module registry, and Rust-declared live event-history field registry through
  `subtr_actor_bakkesmod_graph_info_json_len` and
  `subtr_actor_bakkesmod_write_graph_info_json`

## Windows build outline

The actual plugin build path is Windows/MSVC. From a Developer PowerShell with
Visual Studio 2022 Build Tools, CMake, and Rust on `PATH`:

```powershell
.\bakkesmod\build-windows.ps1
```

The CMake project fetches pinned `bakkesmodorg/BakkesModSDK` sources by default.
To test a local SDK checkout instead:

```powershell
.\bakkesmod\build-windows.ps1 -BakkesModSdkDir C:\src\BakkesModSDK
```

The script builds the Rust ABI DLL, builds the C++ plugin, and copies
`subtr_actor_bakkesmod.dll` next to the plugin DLL under the CMake build output.
It also prepares `bakkesmod\build\Release\bakkesmod-install`, whose contents
can be copied into a BakkesMod root: `plugins\SubtrActorPlugin.dll` and
`data\subtr-actor\subtr_actor_bakkesmod.dll`.

To install the built DLLs into the default local BakkesMod tree:

```powershell
.\bakkesmod\build-windows.ps1 -Install
```

To also add the plugin to BakkesMod's `cfg\plugins.cfg` autoload list:

```powershell
.\bakkesmod\build-windows.ps1 -Install -EnableAutoload
```

The install step copies `SubtrActorPlugin.dll` into BakkesMod's `plugins`
directory and `subtr_actor_bakkesmod.dll` into `data\subtr-actor`, which is
also where the plugin looks for the Rust ABI at runtime.

## In-game verification

Use this checklist after installing the DLLs into BakkesMod. It is the runtime
acceptance check for live graph callability and event-generation parity.

1. Launch Rocket League with BakkesMod and enter a freeplay, replay, or custom
   training session.
2. Wait until at least one live frame has been processed, then run:

   ```text
   subtr_actor_verify_graph
   ```

   The BakkesMod console should log `subtr-actor: graph verification passed`
   along with nonzero byte sizes for `events`, `frame`, `timeline`, `stats`,
   `analysis_nodes`, `event_history`, `graph_info`, every graph output name reported by
   `graph_info`, every builtin stats module surface (`module`, `frame`, and
   `config`), and every name reported by the callable analysis-node name
   registry. It should also log that every resolved graph node is callable by
   name, that named graph outputs match the fixed ABI outputs, that
   `analysis_nodes` contains exactly the callable analysis nodes, and that
   `frame_events_state` exposes every live event family field with an entry
   count for each field. It should also report cumulative `event_history`
   entry counts for the same event-family fields.
   To validate the loaded plugin and Rust ABI without first manually producing
   gameplay events, run:

   ```text
   subtr_actor_self_test_graph
   ```

   The console should log that the self-test derived `active_demos` from the
   synthetic demolition, fed every required event family, and then
   `subtr-actor: graph verification passed` from the strict verifier.
3. Exercise live events that should be visible to the graph: touch the ball,
   trigger a dodge refresh or flip reset setup when possible, pick up a boost
   pad, generate shot/save/assist match-stat deltas, score a goal, and trigger
   a demolition when possible. Overlay labels should appear for drainable graph
   events.
   If the console logs `subtr-actor: live frame processing failed`, the plugin
   preserves queued BakkesMod events and retries them on the next sampled frame.
   After exercising the available event families, run:

   ```text
   subtr_actor_verify_graph require_event_history finish
   ```

   This stricter verifier fails if cumulative `event_history` counts are still
   zero for touch, dodge refresh, boost pad, player stat, goal, or demolition
   event arrays. `active_demos` is current state, not cumulative history, so it
   is intentionally not required by this mode.
   To also require graph-generated timeline output after exercising the goal,
   boost pickup, and player stat/demo portions of the checklist, run:

   ```text
   subtr_actor_verify_graph require_graph_events finish
   ```

   This mode fails if the cumulative `events` graph output still has zero
   entries for the required graph event families (`timeline`, `goal_context`,
   and `boost_pickups`). You can combine both strict checks in one run:
   `subtr_actor_verify_graph require_event_history require_graph_events finish`.
4. Dump the complete live graph surface:

   ```text
   subtr_actor_dump_graph finish
   ```

   BakkesMod should write these files under `data\subtr-actor`:
   `graph-events.json`, `graph-frame.json`, `graph-timeline.json`,
   `graph-stats.json`, `graph-analysis-nodes.json`,
   `graph-event-history.json`, and `graph-info.json`.
   You can validate the dumped files from this repository with:

   ```sh
   python3 bakkesmod/verify-graph-dump.py <path-to-BakkesMod-data>/subtr-actor \
     --require-event-history \
     --require-graph-events
   ```

   `graph-info.json` should list `analysis_nodes` and `event_history` in
   `graph_output_names`, and `callable_analysis_node_names` should match the names verified by
   `subtr_actor_verify_graph`.
   It should also list `event_history_field_names` and
   `required_event_history_field_names`, plus `graph_event_field_names` and
   `required_graph_event_field_names`; the verifier uses those Rust-declared
   registries when checking `frame_events_state`, cumulative `event_history`,
   and graph-generated `events` fields, and fails if the registries omit a graph
   output or live event field required by the plugin ABI.
   `graph-analysis-nodes.json` should contain keys for every node reported by
   `callable_analysis_node_names`.
   `graph-event-history.json` should contain cumulative raw live event-family
   arrays, so events exercised earlier remain visible after
   `frame_events_state` advances to a later frame.
5. Spot-check individual call paths:

   ```text
   subtr_actor_dump_graph_output analysis_nodes finish
   subtr_actor_dump_graph_output event_history finish
   subtr_actor_dump_analysis_node stats_timeline_events finish
   subtr_actor_dump_analysis_node frame_events_state finish
   ```

   Each command should write a nonempty JSON file in `data\subtr-actor`.
   After the live event exercise above, `graph-node-frame_events_state.json`
   should expose the event-family arrays checked by `subtr_actor_verify_graph`
   (`touch_events`, `dodge_refreshed_events`, `boost_pad_events`,
   `player_stat_events`, `goal_events`, `demo_events`, and `active_demos`), and
   the arrays corresponding to exercised events should contain entries. The
   verifier log should also report nonzero entry counts for the matching
   exercised event-family fields.
   If `frame_events_state` has advanced past the event frame, use
   `graph-output-event_history.json` or `graph-event-history.json` for the
   cumulative raw event-family counts, or rerun `subtr_actor_verify_graph` and
   inspect the cumulative `event_history` counts in the console.

## Linux/Nix support

The optional shell can build the Rust ABI and has an experimental Linux-side
MSVC-ABI plugin build path:

```sh
nix develop .#bakkesmod
bakkesmod/build-linux-msvc.sh
```

This uses `xwin` to download/splat the Microsoft CRT and Windows SDK, then
builds the plugin with `clang-cl` and `lld-link` for the
`x86_64-pc-windows-msvc` ABI. That ABI matters: BakkesMod's `pluginsdk.lib` is
a MSVC C++ library, so a plain MinGW build is useful for header smoke checks but
is not expected to produce a compatible final plugin DLL.

The script produces the C++ plugin DLL and copies the Rust ABI DLL into the same
configuration directory. It also prepares a `bakkesmod-install` directory with
the same install layout as the Windows build: copy its contents into a BakkesMod
root to place `SubtrActorPlugin.dll` under `plugins` and
`subtr_actor_bakkesmod.dll` under `data/subtr-actor`. Runtime validation still
needs Windows, BakkesMod, and Rocket League.
