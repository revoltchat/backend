{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell rec {
  buildInputs = [
    # Tools
    pkgs.git
    pkgs.just

    # Cargo
    pkgs.cargo
    pkgs.cargo-nextest
    pkgs.cargo-release

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
