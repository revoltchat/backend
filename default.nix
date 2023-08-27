let
  # Pinned nixpkgs, deterministic. Last updated: 11-08-2023.
  pkgs = import (fetchTarball("https://github.com/NixOS/nixpkgs/archive/bb9707ef2ea4a5b749b362d5cf81ada3ded2c53f.tar.gz")) {};

  # Rolling updates, not deterministic.
  # pkgs = import (fetchTarball("channel:nixpkgs-unstable")) {};
in pkgs.mkShell {
  name = "revoltEnv";

  # LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
  #   pkgs.gcc-unwrapped
  #   pkgs.zlib
  #   pkgs.glib
  #   pkgs.libGL
  # ];

  buildInputs = [
    # Tools
    pkgs.git

    # Database
    # pkgs.mongodb

    # Rust
    pkgs.cargo
    pkgs.rustc
    pkgs.clippy
    pkgs.pkgconfig
    pkgs.openssl.dev
  ];

  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
