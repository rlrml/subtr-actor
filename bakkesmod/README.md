# subtr-actor BakkesMod plugin spike

This is an early BakkesMod integration spike. It is intentionally split into:

- `crates/subtr-actor-bakkesmod`: Rust C ABI that accepts sampled live frames and
  emits mechanics detected by `subtr-actor` calculators.
- `bakkesmod/SubtrActorPlugin.*`: C++ BakkesMod plugin shell that samples the
  local car and ball, calls the Rust ABI, and renders short on-screen labels.

The current spike only feeds the local car. That is enough to test mechanics
whose first pass can work from local kinematics and control state:

- speed flip
- half flip
- wavedash

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
