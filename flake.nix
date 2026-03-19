{
  description = "subtr-actor development environment";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
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
          version = "0.2.2";
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
          pname = "subtr-actor-js-stats-player-pages";
          version = "0.2.2";
          src = ./.;
          npmRoot = "js/stat-evaluation-player";
          npmDeps = pkgs.fetchNpmDeps {
            inherit pname version;
            src = ./js/stat-evaluation-player;
            hash = "sha256-6SViGHrI+sBh2kFA5KiWTG1L+f602xMeXDVSLIfDXkM=";
          };
          preBuild = ''
            rm -rf js/pkg
            mkdir -p js/pkg
            cp -r ${self.packages.${system}.js-web-wasm}/. js/pkg/
          '';
          buildPhase = ''
            runHook preBuild
            pushd js/stat-evaluation-player
            npm run build:site
            popd
            runHook postBuild
          '';

          installPhase = ''
            runHook preInstall
            mkdir -p $out
            cp -r js/stat-evaluation-player/dist/. $out/
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
      }
    );
}
