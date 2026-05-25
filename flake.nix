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
      url = "github:bakkesmodorg/BakkesModSDK/479e8f571cf554b25f4eeb64d611dec4133edcaf";
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
        rustToolchain = fenix.packages.${system}.combine [
          fenix.packages.${system}.stable.toolchain
          fenix.packages.${system}.targets.wasm32-unknown-unknown.stable.rust-std
        ];
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };
        pythonBase = pkgs.callPackage pyproject-nix.build.packages {
          python = pkgs.python311;
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
        mingw = pkgs.pkgsCross.mingwW64;
        shellPackages = [
          pythonEnv
          pkgs.uv
          pkgs.maturin
          pkgs.zlib
          rustToolchain
          pkgs.curl
          pkgs.leveldb
          pkgs.python311Packages.twine
        ];
      in
      {
        packages.python-env = pythonEnv;
        packages.js-web-wasm = rustPlatform.buildRustPackage {
          pname = "subtr-actor-js-web-wasm";
          version = "0.8.6";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = [
            pkgs.writableTmpDirAsHomeHook
            pkgs.wasm-pack
            pkgs.wasm-bindgen-cli
            pkgs.binaryen
          ];
          buildPhase = ''
            runHook preBuild
            mkdir -p $out
            cd js
            wasm-pack build --target web --out-dir "$out"
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
          version = "0.8.6";
          src = ./.;
          npmRoot = "js/stat-evaluation-player";
          npmDeps = pkgs.fetchNpmDeps {
            inherit pname version;
            src = ./js/stat-evaluation-player;
            hash = "sha256-T3Dt7nkyEVgg0gApGocV8FNBD+gmZDapGTVI+q2/2PY=";
          };
          preBuild = ''
            rm -rf js/pkg
            mkdir -p js/pkg
            cp -r ${self.packages.${system}.js-web-wasm}/. js/pkg/
            ln -sfn ../stat-evaluation-player/node_modules js/player/node_modules
            ln -sfn ../stat-evaluation-player/node_modules js/pages/node_modules
          '';
          buildPhase = ''
            runHook preBuild
            pushd js/stat-evaluation-player
            npm run build:site
            popd
            pushd js/pages
            ../scripts/with-clean-npm-env.sh npm run build
            popd
            pushd js/mechanic-review-player
            ../scripts/with-clean-npm-env.sh npm run build
            popd
            runHook postBuild
          '';

          installPhase = ''
            runHook preInstall
            mkdir -p $out
            cp -r js/stat-evaluation-player/dist/. $out/
            mkdir -p $out/stats
            cp -r js/pages/dist/. $out/stats/
            mkdir -p $out/review
            cp -r js/mechanic-review-player/dist/. $out/review/
            runHook postInstall
          '';
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
            pkgs.ninja
            pkgs.python3
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
            echo "  C++ note: MinGW can smoke-compile headers, but SDK linking generally needs MSVC."
          '';
        };
      }
    );
}
