{
  lib,
  pkg-config,
  rustPlatform,
  sqlite,
}:

let
  rustSrc = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../Cargo.lock
      ../Cargo.toml
      ../crates
    ];
  };
in
rec {
  default = pushcompat-bridge;

  pushcompat-bridge = rustPlatform.buildRustPackage {
    pname = "pushcompat-bridge";
    version = "0.1.0";
    src = rustSrc;

    cargoLock.lockFile = ../Cargo.lock;
    buildAndTestSubdir = "crates/bridge";

    nativeBuildInputs = [ pkg-config ];
    buildInputs = [ sqlite ];

    meta = {
      description = "FCM to UnifiedPush relay server";
      mainProgram = "pushcompat-bridge";
    };
  };
}
