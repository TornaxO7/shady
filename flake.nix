{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-parts.url = "github:hercules-ci/flake-parts";
    cargo-watchdoc.url = "github:modprog/cargo-watchdoc";
  };

  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; }
      {
        systems = [
          "x86_64-linux"
          "aarch64-linux"
        ];

        perSystem = { self', lib, system, pkgs, config, ... }:
          let
            dependencies = with pkgs; [
              xorg.libX11
              xorg.libxcb
              xorg.libXi
              xorg.libXrandr
              xorg.libXcursor

              wayland

              alsa-lib

              libGL
              libxkbcommon

              vulkan-loader
              vulkan-validation-layers
              vulkan-tools

              wgpu-utils
            ];
          in
          {
            _module.args.pkgs = import inputs.nixpkgs {
              inherit system;

              overlays = with inputs; [
                rust-overlay.overlays.default
              ];
            };

            packages = {
              shady-cli = pkgs.callPackage (import ./nix/shady-cli-package.nix) { };
              shady-toy = pkgs.callPackage (import ./nix/shady-toy-package.nix) { };
            };

            devShells.default =
              let
                rust-toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
              in
              pkgs.mkShell rec {
                packages = [ rust-toolchain inputs.cargo-watchdoc.packages.${system}.default ]
                  ++ (with pkgs; [ cargo-flamegraph cargo-release cargo-dist ]);


                buildInputs = dependencies;
                nativeBuildInputs = with pkgs; [ pkg-config ];

                shellHook = ''
                  export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:${lib.makeLibraryPath buildInputs}
                '';
              };
          };
      };
}
