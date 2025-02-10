{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell rec {
  buildInputs = [
    # Tools
    pkgs.git

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
