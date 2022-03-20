{
  inputs = {
    dfinity-sdk = {
      url = "github:paulyoung/nixpkgs-dfinity-sdk";
      flake = false;
    };
    flake-utils.url = "github:numtide/flake-utils";
    mozillapkgs = {
      url = "github:mozilla/nixpkgs-mozilla";
      flake = false;
    };
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:nixos/nixpkgs/21.11";
  };

  outputs = {
    self,
    nixpkgs,
    dfinity-sdk,
    flake-utils,
    mozillapkgs,
    naersk,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            (final: prev: (import dfinity-sdk) final prev)
          ];
        };

        # Get a specific rust version
        mozilla = pkgs.callPackage (mozillapkgs + "/package-set.nix") {};
        rust = (mozilla.rustChannelOf {
          channel = "stable";
          version = "1.54.0";
          sha256 = "NL+YHnOj1++1O7CAaQLijwAxKJW9SnHg8qsiOJ1m0Kk=";
          # sha256 = pkgs.lib.fakeSha256;
        }).rust.override {
          extensions = [
            "clippy-preview"
            # "miri-preview"
            # "rls-preview"
            # "rust-analyzer-preview"
            "rustfmt-preview"
            # "llvm-tools-preview"
            # "rust-analysis"
            # "rust-std"
            # "rustc-dev"
            # "rustc-docs"
            "rust-src"
          ];
          targets = [
            "wasm32-unknown-unknown"
          ];
        };

        # Override the version used in naersk
        naersk-lib = naersk.lib."${system}".override {
          cargo = rust;
          rustc = rust;
        };

        dfinitySdk = (pkgs.dfinity-sdk {
          acceptLicenseAgreement = true;
          sdkSystem = system;
        })."0.8.4";
      in
        rec {
          # `nix build`
          defaultPackage = packages.workspace;

          packages.workspace = naersk-lib.buildPackage rec {
            root = ./.;
            cargoBuildOptions = x: x ++ [
              "--target" "wasm32-unknown-unknown"
            ];
            cargoTestOptions = x: x ++ [
              "--target" "wasm32-unknown-unknown"
            ];
            compressTarget = false;
            copyBins = false;
            copyTarget = true;
          };

          packages.icfs = naersk-lib.buildPackage rec {
            pname = "icfs";
            root = ./.;
            cargoBuildOptions = x: x ++ [
              "--package" pname
              "--target" "wasm32-unknown-unknown"
            ];
            cargoTestOptions = x: x ++ [
              "--package" pname
              "--target" "wasm32-unknown-unknown"
            ];
            compressTarget = false;
            copyBins = false;
            copyTarget = true;
          };

          packages.fat = naersk-lib.buildPackage rec {
            pname = "fat";
            root = ./.;
            cargoBuildOptions = x: x ++ [
              "--package" pname
              "--target" "wasm32-unknown-unknown"
            ];
            cargoTestOptions = x: x ++ [
              "--package" pname
              "--target" "wasm32-unknown-unknown"
            ];
            compressTarget = false;
            copyBins = false;
            copyTarget = true;
          };

          # `nix develop`
          devShell = pkgs.mkShell {
            buildInputs = [
              dfinitySdk
            ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.libiconv
            ];

            # supply the specific rust version
            nativeBuildInputs = [ rust ];
          };
        }
    );
}
