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
  `analysis_nodes`, or `graph_info`) using the output names reported in
  `graph-info.json`.
  The `subtr_actor_dump_analysis_node <node_name> [finish]` console command
  writes one graph-backed builtin analysis node by name, using the node names
  reported in `graph-info.json`.
  The `subtr_actor_verify_graph [finish]` console command calls the fixed graph
  outputs plus every builtin analysis node by name from the loaded plugin
  runtime and logs byte sizes, giving a quick in-game check that the graph
  surface is callable after at least one live frame has been processed.

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
- the full builtin analysis-node output map through the `analysis_nodes` named
  graph output, using `subtr_actor_bakkesmod_graph_output_json_len` and
  `subtr_actor_bakkesmod_write_graph_output_json`
- the resolved graph DAG, builtin node registry, and stats module registry through
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
   `analysis_nodes`, `graph_info`, and every name reported by
   `builtin_analysis_node_names`.
3. Exercise live events that should be visible to the graph: touch the ball,
   pick up a boost pad, score a goal, and trigger a demolition when possible.
   Overlay labels should appear for drainable graph events.
4. Dump the complete live graph surface:

   ```text
   subtr_actor_dump_graph finish
   ```

   BakkesMod should write these files under `data\subtr-actor`:
   `graph-events.json`, `graph-frame.json`, `graph-timeline.json`,
   `graph-stats.json`, `graph-analysis-nodes.json`, and `graph-info.json`.
   `graph-info.json` should list `analysis_nodes` in `graph_output_names`, and
   `graph-analysis-nodes.json` should contain keys for the builtin analysis
   nodes reported by `builtin_analysis_node_names`.
5. Spot-check individual call paths:

   ```text
   subtr_actor_dump_graph_output analysis_nodes finish
   subtr_actor_dump_analysis_node stats_timeline_events finish
   subtr_actor_dump_analysis_node frame_events_state finish
   ```

   Each command should write a nonempty JSON file in `data\subtr-actor`.

## Linux/Nix support

The optional shell is for Linux-side Rust development and MinGW smoke checks:

```sh
nix develop .#bakkesmod
```

This Linux workspace can build and check the Rust ABI, but cannot fully compile
or load the C++ BakkesMod plugin without the Windows SDK and Rocket League
runtime. The `.#bakkesmod` shell includes MinGW for header/smoke builds, but
the official `pluginsdk.lib` is MSVC ABI, so SDK-linking plugin builds still
need MSVC or a compatible Windows build environment.
