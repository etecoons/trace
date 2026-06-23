{
  description = "Minimalist Nix-built container for RustWho";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustVersion = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "wasm32-unknown-unknown" ];
        };
        rustPlatform = pkgs.makeRustPlatform {
          rustc = rustVersion;
          cargo = rustVersion;
        };

        # 1. Build the WASM frontend
        frontend = pkgs.stdenv.mkDerivation {
          pname = "rustwho-frontend";
          version = "1.0.0";
          src = ./.;

          nativeBuildInputs = [
            rustVersion
            pkgs.wasm-bindgen-cli
            pkgs.trunk
          ];

          buildPhase = ''
            export HOME=$TMPDIR
            cd frontend
            trunk build --release
          '';

          installPhase = ''
            mkdir -p $out/dist
            cp -r dist/* $out/dist/
          '';
        };

        # 2. Build the Axum backend
        backend = rustPlatform.buildRustPackage {
          pname = "rustwho-backend";
          version = "1.0.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl ];

          doCheck = false;

          buildPhase = ''
            cargo build --release --bin backend
          '';

          installPhase = ''
            mkdir -p $out/bin
            cp target/release/backend $out/bin/backend
          '';
        };

        # 3. Create the layered Docker container image
        dockerImage = pkgs.dockerTools.buildLayeredImage {
          name = "rustwho-nix";
          tag = "latest";
          
          # Run under the nobody user (UID 65534)
          config = {
            Cmd = [ "${backend}/bin/backend" ];
            WorkingDir = "/app";
            Env = [
              "PORT=4404"
            ];
            ExposedPorts = {
              "4404/tcp" = {};
            };
            User = "nobody:nobody";
          };

          # Create /app directory structure inside the container
          extraCommands = ''
            mkdir -p app/frontend
            cp -r ${frontend}/dist app/frontend/dist
          '';
        };

      in {
        packages = {
          inherit frontend backend dockerImage;
          default = dockerImage;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustVersion
            pkgs.trunk
            pkgs.wasm-bindgen-cli
          ];
        };
      }
    );
}
