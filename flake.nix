{
  description = "subtr-actor development environment";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    bakkesmod-sdk = {
      url = "github:bakkesmodorg/BakkesModSDK";
      flake = false;
    };
    pyproject-nix = {
      url = "github:pyproject-nix/pyproject.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    uv2nix = {
      url = "github:pyproject-nix/uv2nix";
      inputs.pyproject-nix.follows = "pyproject-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    pyproject-build-systems = {
      url = "github:pyproject-nix/build-system-pkgs";
      inputs.pyproject-nix.follows = "pyproject-nix";
      inputs.uv2nix.follows = "uv2nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      fenix,
      bakkesmod-sdk,
      pyproject-nix,
      uv2nix,
      pyproject-build-systems,
    }:
    let
      workspace = uv2nix.lib.workspace.loadWorkspace { workspaceRoot = ./python; };
    in
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        rustToolchainManifest = {
          channel = "1.97.0";
          sha256 = "sha256-OATSZm98Es5kIFuqaba+UvkQtFsVgJEBMmS+t6od5/U=";
        };
        rustToolchain = fenix.packages.${system}.combine [
          (fenix.packages.${system}.toolchainOf rustToolchainManifest).defaultToolchain
          (fenix.packages.${system}.targets.wasm32-unknown-unknown.toolchainOf rustToolchainManifest).rust-std
          (fenix.packages.${system}.targets.x86_64-pc-windows-msvc.toolchainOf rustToolchainManifest).rust-std
        ];
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };
        # Keep this in sync with .github/scripts/install-cargo-rdme.sh. Version
        # 2.x requires a separate nightly toolchain to resolve intralinks.
        cargoRdme = pkgs.rustPlatform.buildRustPackage rec {
          pname = "cargo-rdme";
          version = "1.5.0";
          src = pkgs.fetchCrate {
            inherit pname version;
            hash = "sha256-TmV6Fc5vlc4fm9w4+iuxmnonwsEbqoJ3jvpIyQOuxjg=";
          };
          cargoHash = "sha256-EjIvKf1XgHubvyWPOAjysNH4nD0xqOWYg1FeiPSYh4c=";
        };
        pythonBase = pkgs.callPackage pyproject-nix.build.packages {
          python = pkgs.python312;
        };
        overlay = workspace.mkPyprojectOverlay {
          sourcePreference = "wheel";
        };
        pythonSet = (
          pythonBase.overrideScope (
            pkgs.lib.composeManyExtensions [
              pyproject-build-systems.overlays.wheel
              overlay
            ]
          )
        );
        pythonEnv = pythonSet.mkVirtualEnv "subtr-actor-python-dev-env" {
          numpy = [ ];
          pytest = [ ];
          wheel = [ ];
        };
        projectVersion = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).workspace.package.version;
        # Build identification for the replay-to-training plugin: the nix
        # sandbox has no .git, so the hash/dirty/date come from the flake's
        # own source metadata and are exported as environment variables the
        # CMake configure and Rust build.rs both honor.
        formatCommitDate =
          value:
          if value == "unknown" then
            value
          else if builtins.stringLength value >= 14 then
            "${builtins.substring 0 4 value}-${builtins.substring 4 2 value}-${builtins.substring 6 2 value}T${builtins.substring 8 2 value}:${builtins.substring 10 2 value}:${builtins.substring 12 2 value}Z"
          else
            value;
        buildGitHash =
          if self ? shortRev then
            self.shortRev
          else if self ? dirtyShortRev then
            self.dirtyShortRev
          else
            "unknown";
        buildGitDirty = if self ? dirtyShortRev then "1" else "0";
        buildCommitDate =
          if self ? lastModifiedDate then formatCommitDate self.lastModifiedDate else "unknown";
        mingw = pkgs.pkgsCross.mingwW64;
        xwinMsvcSysroot = pkgs.stdenvNoCC.mkDerivation {
          pname = "xwin-msvc-sysroot";
          version = "x86_64-desktop";
          dontUnpack = true;
          nativeBuildInputs = [ pkgs.xwin ];
          outputHashAlgo = "sha256";
          outputHashMode = "recursive";
          outputHash = "sha256-PMJqIj13w/ssarSzm0yKzqh4uUfBrOVFe+CmzZPv3xM=";
          installPhase = ''
            runHook preInstall
            export HOME="$TMPDIR"
            xwin \
              --accept-license \
              --cache-dir "$TMPDIR/xwin-cache" \
              --arch x86_64 \
              --variant desktop \
              splat \
              --output "$out" \
              --use-winsysroot-style \
              --preserve-ms-arch-notation \
              --copy
            runHook postInstall
          '';
        };
        shellPackages = [
          pythonEnv
          pkgs.uv
          pkgs.maturin
          pkgs.zlib
          rustToolchain
          cargoRdme
          pkgs.curl
          pkgs.leveldb
          pkgs.python312Packages.twine
        ];
      in
      {
        packages.python-env = pythonEnv;
        packages.js-web-wasm = rustPlatform.buildRustPackage {
          pname = "subtr-actor-js-web-wasm";
          version = projectVersion;
          src = ./.;
          cargoDeps =
            let
              vendorDir = rustPlatform.importCargoLock {
                lockFile = ./Cargo.lock;
                # Fetch git dependencies (e.g. patched boxcars) via
                # builtins.fetchGit so Cargo.lock stays the single source of
                # truth — no outputHashes entry to keep in sync by hand.
                allowBuiltinFetchGit = true;
                extraRegistries = {
                  "https://github.com/rust-lang/crates.io-index" = "https://static.crates.io/crates";
                };
              };
            in
            pkgs.runCommand "cargo-vendor-dir" { } ''
              cp -R ${vendorDir}/. "$out"
              chmod u+w "$out/.cargo" "$out/.cargo/config.toml"
              # Cargo rejects this alias as a duplicate definition of crates-io.
              sed -i '/^\[source\."https:\/\/github.com\/rust-lang\/crates.io-index"\]$/,+2d' \
                "$out/.cargo/config.toml"
            '';
          nativeBuildInputs = [
            pkgs.writableTmpDirAsHomeHook
            pkgs.wasm-bindgen-cli
            pkgs.binaryen
          ];
          buildPhase = ''
            runHook preBuild
            mkdir -p $out
            cd js
            cargo build --release --target wasm32-unknown-unknown
            wasm-bindgen \
              --target web \
              --out-dir "$out" \
              ../target/wasm32-unknown-unknown/release/rl_replay_subtr_actor.wasm
            runHook postBuild
          '';
          installPhase = ''
            runHook preInstall
            runHook postInstall
          '';
          doCheck = false;
          dontCargoInstall = true;
        };
        packages.js-stats-player-pages = pkgs.buildNpmPackage rec {
          pname = "subtr-actor-js-pages";
          version = projectVersion;
          src = pkgs.runCommand "subtr-actor-js-pages-source" { nativeBuildInputs = [ pkgs.nodejs ]; } ''
            mkdir -p "$out"
            cp -R ${./.}/. "$out"/
            chmod -R u+w "$out"
          '';
          npmRoot = "js/stat-evaluation-player";
          npmDeps = pkgs.importNpmLock { npmRoot = "${src}/js/stat-evaluation-player"; };
          npmConfigHook = pkgs.importNpmLock.npmConfigHook;
          # Skip lifecycle install scripts. The only one that matters here is
          # esbuild's postinstall (`node install.js` -> validateBinaryVersion),
          # which crashes in the sandbox when @rlrml/player is pulled in as a
          # nested file: dependency (its esbuild bin lands as the native ELF and
          # gets run via node). esbuild needs no postinstall to work — its
          # platform package is already materialized from the lock — and the
          # site build resolves @rlrml/player via Vite source aliases
          # (../player/src/lib.ts), so the file: dep's prepare/dist is never
          # needed. `npm run build:site`/`build` still run (ignore-scripts only
          # suppresses lifecycle + pre/post hooks, not explicit run targets).
          npmFlags = [ "--ignore-scripts" ];
          preBuild = ''
            rm -rf js/pkg
            mkdir -p js/pkg
            cp -r ${self.packages.${system}.js-web-wasm}/. js/pkg/
            ln -sfn ../stat-evaluation-player/node_modules js/player/node_modules
            ln -sfn ../stat-evaluation-player/node_modules js/pages/node_modules
            mkdir -p js/stat-evaluation-player/node_modules/@rlrml
            rm -rf js/stat-evaluation-player/node_modules/@rlrml/player
            ln -sfn ../../../player js/stat-evaluation-player/node_modules/@rlrml/player
          '';
          buildPhase = ''
            runHook preBuild
            pushd js/stat-evaluation-player
            npm run build:site
            popd
            pushd js/pages
            ../scripts/with-clean-npm-env.sh npm run build
            popd
            runHook postBuild
          '';

          installPhase = ''
            runHook preInstall
            mkdir -p $out
            cp -r js/stat-evaluation-player/dist/. $out/
            cp -r js/player/public/. $out/
            mkdir -p $out/stats
            cp -r js/pages/dist/. $out/stats/
            mkdir -p $out/review
            cp -r js/stat-evaluation-player/dist/. $out/review/
            runHook postInstall
          '';
        };
        # The publishable npm package layout for @rlrml/subtr-actor, byte-for-byte
        # equivalent to what `npm publish` ships (wasm-pack --no-pack output +
        # prepare-wasm-package.mjs manifest). Downstream consumers can vendor this
        # directly instead of the npm registry tarball, guaranteeing the wasm is
        # built from this repo's Cargo.lock (including [patch.crates-io] forks).
        packages.js-wasm-pkg =
          pkgs.runCommand "rlrml-subtr-actor-npm-pkg"
            { nativeBuildInputs = [ pkgs.nodejs ]; }
            ''
              mkdir -p build/js/pkg
              cp ${./js/package.json} build/js/package.json
              cp -r ${./js/scripts} build/js/scripts
              cp -r ${self.packages.${system}.js-web-wasm}/. build/js/pkg/
              chmod -R u+w build
              # Mirror the publish flow (wasm-pack --no-pack): drop wasm-pack's own
              # manifest so prepare-wasm-package.mjs regenerates the publishable one.
              rm -f build/js/pkg/package.json
              node build/js/scripts/prepare-wasm-package.mjs
              mkdir -p $out
              for f in package.json rl_replay_subtr_actor.js rl_replay_subtr_actor.d.ts rl_replay_subtr_actor_bg.wasm; do
                install -m 644 "build/js/pkg/$f" "$out/$f"
              done
            '';
        # The publishable npm package layout for @rlrml/player (dist/ + manifest via
        # prepare-package.mjs), built against this repo's js-wasm-pkg bindings.
        packages.js-player-pkg = pkgs.buildNpmPackage {
          pname = "rlrml-player-npm-pkg";
          version = projectVersion;
          src = ./.;
          npmRoot = "js/player";
          npmDeps = pkgs.importNpmLock { npmRoot = ./js/player; };
          npmConfigHook = pkgs.importNpmLock.npmConfigHook;
          preBuild = ''
            rm -rf js/pkg
            mkdir -p js/pkg
            cp -r ${self.packages.${system}.js-web-wasm}/. js/pkg/
          '';
          buildPhase = ''
            runHook preBuild
            pushd js/player
            npm run build:dist
            popd
            runHook postBuild
          '';
          installPhase = ''
            runHook preInstall
            pushd js/player
            staged="$(node ./scripts/prepare-package.mjs)"
            popd
            mkdir -p $out
            cp -R "$staged"/. $out/
            runHook postInstall
          '';
        };
        # Backward-compatible flake output for callers that have not yet moved
        # from the old viewer package output name to @rlrml/player.
        packages.js-viewer-pkg = self.packages.${system}.js-player-pkg;
        packages.xwin-msvc-sysroot = xwinMsvcSysroot;
        packages.bakkesmod-plugin = rustPlatform.buildRustPackage {
          pname = "subtr-actor-bakkesmod-plugin";
          version = projectVersion;
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            # Same as js-web-wasm: hash-free git-dependency fetching keeps
            # Cargo.lock the single source of truth for crate revisions.
            allowBuiltinFetchGit = true;
          };
          nativeBuildInputs = [
            pkgs.cmake
            pkgs.llvmPackages_21.clang-unwrapped
            pkgs.llvmPackages_21.llvm
            pkgs.lld
            pkgs.ninja
            pkgs.python3
          ];
          buildPhase = ''
            runHook preBuild
            export HOME="$TMPDIR"
            export BUILD_DIR="$TMPDIR/bakkesmod-build"
            export XWIN_SYSROOT="${xwinMsvcSysroot}"
            export BAKKESMODSDK_DIR="${bakkesmod-sdk}"
            export BAKKESMOD_SDK_DIR="$BAKKESMODSDK_DIR"
            bash bakkesmod/subtr-actor/build-linux-msvc.sh
            runHook postBuild
          '';
          installPhase = ''
            runHook preInstall
            mkdir -p "$out"
            cp -r "$BUILD_DIR/Release/." "$out/"
            runHook postInstall
          '';
          doCheck = false;
          dontCargoInstall = true;
        };
        packages.bakkesmod-replay-to-training = rustPlatform.buildRustPackage {
          pname = "subtr-actor-replay-to-training-plugin";
          version = projectVersion;
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };
          nativeBuildInputs = [
            pkgs.cmake
            pkgs.llvmPackages_21.clang-unwrapped
            pkgs.llvmPackages_21.llvm
            pkgs.lld
            pkgs.ninja
            pkgs.python3
          ];
          buildPhase = ''
            runHook preBuild
            export HOME="$TMPDIR"
            export BUILD_DIR="$TMPDIR/bakkesmod-replay-to-training-build"
            export XWIN_SYSROOT="${xwinMsvcSysroot}"
            export BAKKESMODSDK_DIR="${bakkesmod-sdk}"
            export BAKKESMOD_SDK_DIR="$BAKKESMODSDK_DIR"
            export REPLAY_TO_TRAINING_GIT_HASH="${buildGitHash}"
            export REPLAY_TO_TRAINING_GIT_DIRTY="${buildGitDirty}"
            export REPLAY_TO_TRAINING_COMMIT_DATE="${buildCommitDate}"
            bash bakkesmod/replay-to-training/build-linux-msvc.sh
            runHook postBuild
          '';
          installPhase = ''
            runHook preInstall
            mkdir -p "$out"
            cp -r "$BUILD_DIR/Release/." "$out/"
            runHook postInstall
          '';
          doCheck = false;
          dontCargoInstall = true;
        };

        devShells.default = pkgs.mkShell {
          packages = shellPackages;

          env = {
            UV_PYTHON_DOWNLOADS = "never";
          };

          shellHook = ''
            unset PYTHONPATH
            export REPO_ROOT=$(git rev-parse --show-toplevel)
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath shellPackages}:${pkgs.stdenv.cc.cc.lib.outPath}/lib:/run/opengl-driver/lib/:''${LD_LIBRARY_PATH:-}"
          '';
        };
        devShells.bakkesmod = pkgs.mkShell {
          packages = shellPackages ++ [
            mingw.stdenv.cc
            pkgs.cmake
            pkgs.file
            pkgs.llvmPackages_21.clang-unwrapped
            pkgs.llvmPackages_21.llvm
            pkgs.lld
            pkgs.ninja
            pkgs.python3
            pkgs.wine64Packages.unstable
            pkgs.xwin
          ];

          env = {
            UV_PYTHON_DOWNLOADS = "never";
            MCFGTHREAD_INCLUDE = "${mingw.windows.mcfgthreads.dev}/include";
            MCFGTHREAD_LIB = "${mingw.windows.mcfgthreads}/lib";
          };

          shellHook = ''
            unset PYTHONPATH
            export REPO_ROOT=$(git rev-parse --show-toplevel)
            export BAKKESMODSDK_DIR="''${BAKKESMODSDK_DIR:-${bakkesmod-sdk}}"
            export BAKKESMOD_SDK_DIR="''${BAKKESMOD_SDK_DIR:-$BAKKESMODSDK_DIR}"
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath shellPackages}:${pkgs.stdenv.cc.cc.lib.outPath}/lib:/run/opengl-driver/lib/:''${LD_LIBRARY_PATH:-}"
            echo "subtr-actor BakkesMod shell"
            echo "  Rust ABI: cargo build -p subtr-actor-bakkesmod --release"
            echo "  SDK:      $BAKKESMODSDK_DIR"
            echo "  Linux MSVC build: bakkesmod/subtr-actor/build-linux-msvc.sh"
            echo "  Wine:     wine <windows-exe> for local MSVC artifact smoke tests"
            echo "  MinGW note: MinGW can smoke-compile headers, but final plugin linking needs MSVC ABI."
          '';
        };
      }
    );
}
