let
  # Pinned nixpkgs, deterministic. Last updated: 28-07-2024.
  pkgs = import (fetchTarball("https://github.com/NixOS/nixpkgs/archive/9b34ca580417e1ebc56c4df57d8b387dad686665.tar.gz")) {};

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

    # Cargo
    pkgs.cargo
    pkgs.cargo-nextest

    # Rust
    pkgs.rustc
    pkgs.clippy
    pkgs.rustfmt
    pkgs.pkg-config
    pkgs.openssl.dev

    # mdbook
    pkgs.mdbook
  ];

  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
