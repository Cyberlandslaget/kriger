{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, rust-overlay }@inputs:
  let
    inherit (nixpkgs) lib;

    systems = [
      "x86_64-linux"
      "aarch64-linux"
      "x86_64-darwin"
      "aarch64-darwin"
      "armv7l-linux"
    ];

    forAllSystems = f: nixpkgs.lib.genAttrs systems (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          (import rust-overlay)
        ];
      };

      rust-bin = rust-overlay.lib.mkRustBin { } pkgs.buildPackages;
      toolchain = rust-bin.stable.latest.default;
    in f system pkgs toolchain);
  in {

    devShell = forAllSystems (system: pkgs: toolchain: pkgs.mkShell {
      nativeBuildInputs = with pkgs; [
        toolchain
        mysql-client
        cargo-nextest
      ] ++ lib.optionals stdenv.isDarwin [
        darwin.apple_sdk.frameworks.Security
      ];

      RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
    });
  };
}
