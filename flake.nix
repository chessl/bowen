{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1.*.tar.gz";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      overlays = [
        rust-overlay.overlays.default
        (final: prev: {
          rustToolchain =
            let
              rust = prev.rust-bin;
            in
            if builtins.pathExists ./rust-toolchain.toml then
              rust.fromRustupToolchainFile ./rust-toolchain.toml
            else if builtins.pathExists ./rust-toolchain then
              rust.fromRustupToolchainFile ./rust-toolchain
            else
              rust.stable.latest.minimal.override {
                extensions = [
                  "rust-src"
                  "rustfmt"
                  "clippy"
                ];
              };
        })
      ];
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forEachSupportedSystem =
        f:
        nixpkgs.lib.genAttrs supportedSystems (
          system: f { pkgs = import nixpkgs { inherit overlays system; }; }
        );
      cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      mkBowenPackage =
        pkgs:
        let
          rustPlatform = pkgs.makeRustPlatform {
            cargo = pkgs.rustToolchain;
            rustc = pkgs.rustToolchain;
          };
        in
        rustPlatform.buildRustPackage {
          pname = "bowen";
          version = cargoToml.package.version;
          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;
          cargoBuildFlags = [
            "--bin"
            "bowen"
          ];

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs =
            with pkgs;
            [
              openssl
            ]
            ++ lib.optionals stdenv.isDarwin [
              libiconv
              apple-sdk
            ];

          doCheck = false;

          meta = {
            description = cargoToml.package.description;
            mainProgram = "bowen";
          };
        };
    in
    {
      packages = forEachSupportedSystem (
        { pkgs }:
        let
          bowen = mkBowenPackage pkgs;
        in
        {
          default = bowen;
          bowen = bowen;
        }
      );

      apps = forEachSupportedSystem (
        { pkgs }:
        let
          system = pkgs.stdenv.hostPlatform.system;
        in
        {
          default = {
            type = "app";
            program = "${self.packages.${system}.default}/bin/bowen";
            meta.description = cargoToml.package.description;
          };
        }
      );

      devShells = forEachSupportedSystem (
        { pkgs }: {
          default = pkgs.mkShell {
            packages =
              with pkgs;
              [
                rustToolchain
                openssl
                pkg-config
                cargo-info
                cargo-deny
                cargo-edit
                cargo-watch
                cargo-udeps
                cargo-wizard
                rust-analyzer
                bacon

                # unittest
                cargo-nextest

              ]
              ++ lib.optionals pkgs.stdenv.isDarwin [
                # Additional darwin specific inputs can be set here
                libiconv
                apple-sdk
              ];
          };
        }
      );
    };
}
