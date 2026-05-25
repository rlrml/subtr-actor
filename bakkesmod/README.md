# subtr-actor BakkesMod plugin spike

This is an early BakkesMod integration spike. It is intentionally split into:

- `crates/subtr-actor-bakkesmod`: Rust C ABI that accepts sampled live frames,
  evaluates the shared `subtr-actor` analysis graph, drains normalized mechanic
  events for overlay use, and exposes the current graph event bundle plus frame
  stats snapshot as JSON.
- `bakkesmod/SubtrActorPlugin.*`: C++ BakkesMod plugin shell that samples active
  cars and the ball, calls the Rust ABI, and renders short on-screen labels.
  The `subtr_actor_dump_graph` console command writes the current full timeline
  event bundle and current frame stats snapshot to BakkesMod's `data/subtr-actor`
  directory as JSON.

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
