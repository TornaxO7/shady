{ rustPlatform
, lib
, pkg-config
, alsa-lib
}:
let
  cargoToml = builtins.fromTOML (builtins.readFile ../shady-cli/Cargo.toml);

  dependencies = [
    pkg-config
    alsa-lib
  ];
in
rustPlatform.buildRustPackage rec {
  pname = cargoToml.package.name;
  version = cargoToml.package.version;

  src = builtins.path {
    path = ../.;
  };

  buildAndTestSubdir = "shady-cli";

  buildInputs = dependencies;
  nativeBuildInputs = dependencies;

  cargoLock.lockFile = ../Cargo.lock;

  meta = {
    description = cargoToml.package.description;
    homepage = cargoToml.package.homepage;
    license = lib.licenses.gpl3;
    mainProgram = pname;
  };
}
