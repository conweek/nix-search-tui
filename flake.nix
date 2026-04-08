{
  description = "Nix Search TUI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, rust-overlay, crane }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };
          rustToolchain = pkgs.rust-bin.stable.latest.default;
          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

          # Filter source to only include Rust-relevant files
          src = craneLib.cleanCargoSource ./.;

          # Common arguments shared between build steps
          commonArgs = {
            inherit src;
            buildInputs = [
              pkgs.pkg-config
              pkgs.openssl
            ];
            nativeBuildInputs = [
              pkgs.pkg-config
            ];
          };

          # Build only the cargo dependencies for caching
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          # Build the actual binary
          nix-search-tui-unwrapped = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
          });

          # Wrap with runtime dependencies (curl, man)
          nix-search-tui = pkgs.runCommand "nix-search-tui" {
            nativeBuildInputs = [ pkgs.makeWrapper ];
          } ''
            mkdir -p $out/bin
            makeWrapper ${nix-search-tui-unwrapped}/bin/nix-search-tui $out/bin/nix-search-tui \
              --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.curl pkgs.man ]}
          '';
        in
        {
          default = nix-search-tui;
          inherit nix-search-tui;
        }
      );

      checks = forAllSystems (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };
          rustToolchain = pkgs.rust-bin.stable.latest.default;
          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
          src = craneLib.cleanCargoSource ./.;
          commonArgs = {
            inherit src;
            buildInputs = [ pkgs.pkg-config pkgs.openssl ];
            nativeBuildInputs = [ pkgs.pkg-config ];
          };
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        in
        {
          clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          fmt = craneLib.cargoFmt {
            inherit src;
          };
        }
      );

      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };
          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
          };
        in
        {
          default = pkgs.mkShell {
            buildInputs = [
              rustToolchain
              pkgs.pkg-config
              pkgs.openssl
            ];
          };
        }
      );
    };
}
