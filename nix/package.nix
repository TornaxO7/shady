{ rustPlatform
, lib
, pkg-config

, libX11
, libxcb
, libXi
, libXrandr
, libXcursor

, alsa-lib

, libGL
, libxkbcommon

, vulkan-loader
, vulkan-validation-layers
, vulkan-tools
}:
let
  cargoToml = builtins.fromTOML (builtins.readFile ../shady-app/Cargo.toml);

  dependencies = [
    pkg-config
    libX11
    libxcb
    libXi
    libXrandr
    libXcursor
    alsa-lib
    libGL
    libxkbcommon
    vulkan-loader
    vulkan-validation-layers
    vulkan-tools
  ];
in
rustPlatform.buildRustPackage rec {
  pname = cargoToml.package.name;
  version = cargoToml.package.version;

  src = builtins.path {
    path = ../.;
  };

  buildInputs = dependencies;
  nativeBuildInputs = dependencies;

  cargoLock.lockFile = ../Cargo.lock;

  meta = {
    description = cargoToml.package.description;
    homepage = cargoToml.package.homepage;
    license = lib.licenses.gpl2;
    mainProgram = pname;
  };
}
