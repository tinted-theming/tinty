{
  lib,
  rustPlatform,
}:
rustPlatform.buildRustPackage {
  pname = "tinty";
  version = (builtins.fromTOML (builtins.readFile ../Cargo.toml)).package.version;
  src = lib.cleanSource ./..;
  cargoLock.lockFile = ../Cargo.lock;
  doCheck = false;
  meta = {
    description = "Change the theme of your terminal, text editor and anything else with one command!";
    homepage = "https://github.com/tinted-theming/tinty";
    license = lib.licenses.gpl3Only;
    mainProgram = "tinty";
  };
}
