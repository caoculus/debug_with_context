{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs = {
    nixpkgs,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };
      rust-bin = pkgs.rust-bin.selectLatestNightlyWith (toolchain:
        toolchain.default.override {
          extensions = ["rust-src" "rust-analyzer"];
        });
    in {
      devShells.default = pkgs.mkShell {
        buildInputs = [
          rust-bin
        ];
      };
    });
}
