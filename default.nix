# Non-flake fallback for users without the `flakes` experimental feature.
# Prefer using the flake (`flake.nix`) when possible.
#
# Usage:
#   nix-build -E '(import <nixpkgs> {}).callPackage ./default.nix {}'
#   nix-build ./default.nix
#
# This is intentionally simple: a single `rustPlatform.buildRustPackage`
# derivation, mirroring what `flake.nix` does.

{ rustPlatform
, fetchFromGitHub
, lib
, wayland
, libxkbcommon
, sqlite
, pkg-config
, wayland-protocols
, gtk4
, gtk4-layer-shell
}:

rustPlatform.buildRustPackage rec {
  pname = "author-clipboard";
  # Mirror workspace.version + flake.nix. CI verifies they stay in sync.
  version = "0.5.0";

  src = fetchFromGitHub {
    owner = "namikofficial";
    repo = "author-clipboard";
    rev = "v${version}";
    sha256 = "0000000000000000000000000000000000000000000000000000";
  };

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  RUSTFLAGS = "-D warnings";
  cargoBuildFlags = [ "--workspace" ];

  buildInputs = [
    wayland
    libxkbcommon
    sqlite
    pkg-config
    wayland-protocols
    gtk4
    gtk4-layer-shell
  ];

  doCheck = false;
  meta = with lib; {
    description = "Native COSMIC clipboard manager with wlroots compositor support";
    homepage = "https://github.com/namikofficial/author-clipboard";
    license = licenses.gpl3;
    platforms = [ "x86_64-linux" "aarch64-linux" ];
    mainProgram = "author-clipboard";
  };
}
