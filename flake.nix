{
  description = "Pico 2 W rust code for fan control in my server closet";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    fenix.url = "github:nix-community/fenix";
    git-hooks.url = "github:cachix/git-hooks.nix";
  };
  outputs =
    {
      self,
      nixpkgs,
      fenix,
      git-hooks,
    }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = fn: nixpkgs.lib.genAttrs supportedSystems (system: fn system);
      pkgs = system: nixpkgs.legacyPackages.${system};
      fenixpkgs = system: fenix.packages.${system};
      toolchainFor =
        system:
        (fenixpkgs system).combine [
          (fenixpkgs system).stable.cargo
          (fenixpkgs system).stable.rustc
          (fenixpkgs system).latest.rustfmt
          (fenixpkgs system).stable.clippy
          (fenixpkgs system).stable.rust-src
          (fenixpkgs system).targets."thumbv8m.main-none-eabihf".stable.rust-std
          (fenixpkgs system).targets."riscv32imac-unknown-none-elf".stable.rust-std
          (fenixpkgs system).stable.rust-analyzer
        ];
    in
    {
      devShells = forAllSystems (system: {
        default = (pkgs system).mkShell {
          packages = [
            (toolchainFor system)
            (pkgs system).picotool
            (pkgs system).probe-rs-tools
            (pkgs system).cargo-generate
          ];
          RUST_SRC_PATH = "${(fenixpkgs system).stable.rust-src}/lib/rustlib/src/rust/library";
          RUST_BACKTRACE = "1";
        };
      });
    };
}
