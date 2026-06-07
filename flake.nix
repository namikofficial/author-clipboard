{
  description = "Native COSMIC clipboard manager with wlroots compositor support";

  # Nix flake for author-clipboard.
  #
  # Usage:
  #   nix run github:namikofficial/author-clipboard
  #   nix profile install github:namikofficial/author-clipboard
  #   nix build github:namikofficial/author-clipboard
  #
  # To enter a development shell with all build deps available:
  #   nix develop github:namikofficial/author-clipboard
  #
  # The flake pins the upstream tag to the workspace version (currently 0.5.0).
  # When cutting a release, update the `src.tag` below and the `version` arg
  # in lockstep with Cargo.toml.

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Pin the Rust toolchain. 1.79 is the minimum that compiles the
        # workspace cleanly with the current libcosmic version.
        rustToolchain = pkgs.rust-bin.stable."1.79.0".default;

        # Version pinned in lockstep with workspace.version. CI verifies
        # that this matches the tag used in `src`.
        version = "0.5.0";
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "author-clipboard";
          inherit version;
          src = pkgs.fetchFromGitHub {
            owner = "namikofficial";
            repo = "author-clipboard";
            rev = "v${version}";
            # SHA256 placeholder. Update with `nix-prefetch-url` on first build.
            # `nix flake update --override-input author-clipboard ...` is a
            # cleaner path; until first build is verified, leave at "".
            sha256 = "0000000000000000000000000000000000000000000000000000";
          };

          inherit rustToolchain;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          # Match the workspace lints; suppress upstream libcosmic warnings.
          RUSTFLAGS = "-D warnings";

          # Build all binaries in the workspace.
          cargoBuildFlags = [ "--workspace" ];

          # We don't run the test suite from the flake; CI does that.
          doCheck = false;

          # Runtime deps. COSMIC_DATA_CONTROL_ENABLED is set at runtime by
          # the user; we don't bake it into the wrapper.
          buildInputs = with pkgs; [
            wayland
            libxkbcommon
            sqlite
            pkg-config
            wayland-protocols
          ];

          # No postInstall patching needed: the binaries are static enough
          # to run from a Nix store path. The user is expected to provide
          # WAYLAND_DISPLAY and XDG_RUNTIME_DIR.
        };

        # `nix run` launches the applet.
        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/author-clipboard";
        };

        # A dev shell with all build-time deps. Use `nix develop` to enter.
        devShells.default = pkgs.mkShell {
          inputsFrom = [ self.packages.${system}.default ];
          packages = with pkgs; [
            pkg-config
            wayland
            libxkbcommon
            sqlite
            wayland-protocols
            gtk4
            # gtk4-layer-shell is in nixpkgs as `gtk4-layer-shell`.
            gtk4-layer-shell
            cargo
            rustc
            rustfmt
            clippy
            rust-analyzer
          ];

          RUSTFLAGS = "-D warnings";
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (
            (with pkgs; [ wayland libxkbcommon sqlite gtk4 gtk4-layer-shell ])
          );
        };

        # Expose the applet / daemon / ctl / hypr-picker as separate outputs
        # for users who only want a single binary.
        packages.applet = self.packages.${system}.default;
        packages.daemon = pkgs.runCommand "author-clipboard-daemon-${version}" { } ''
          mkdir -p $out/bin
          ln -s ${self.packages.${system}.default}/bin/author-clipboard-daemon $out/bin/
        '';
        packages.ctl = pkgs.runCommand "author-clipboard-ctl-${version}" { } ''
          mkdir -p $out/bin
          ln -s ${self.packages.${system}.default}/bin/author-clipboard-ctl $out/bin/
        '';
        packages.hypr-picker = pkgs.runCommand "author-clipboard-hypr-picker-${version}" { } ''
          mkdir -p $out/bin
          ln -s ${self.packages.${system}.default}/bin/author-clipboard-hypr-picker $out/bin/
        '';
      });
}
