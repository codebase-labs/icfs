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
          channel = "1.55.0";
          sha256 = "HNIlEerJvk6sBfd8zugzwSgSiHcQH8ZbqWQn9BGfmpo=";
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

        buildRustPackage = name:
          let
            defaultArgs = [
              "--target" "wasm32-unknown-unknown"
            ];
            packageArgs = [
              "--package" name
            ];
          in
            naersk-lib.buildPackage {
              root = ./.;
              cargoBuildOptions = x: x ++ defaultArgs ++ packageArgs;
              cargoTestOptions = x: x ++ defaultArgs ++ packageArgs;
              compressTarget = true;
              copyBins = false;
              copyTarget = true;
            };
      in
        rec {
          # `nix build`
          defaultPackage = packages.all;

          packages.all =  pkgs.runCommand "all" {
            buildInputs = [
              packages.icfs
              packages.fat
            ];
          } ''
            touch $out
          '';

          packages.icfs = buildRustPackage "icfs";
          packages.fat = buildRustPackage "fat";

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
