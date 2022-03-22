{
  inputs = {
    dfinity-sdk = {
      url = "github:paulyoung/nixpkgs-dfinity-sdk";
      flake = false;
    };
    flake-utils.url = "github:numtide/flake-utils";

    # https://github.com/nix-community/naersk/pull/211
    naersk.url = "github:mhuesch/naersk?rev=193e049d6e4c841faf800e302551d2e0a48eee88";
    # naersk.url = "github:nix-community/naersk";

    nixpkgs.url = "github:nixos/nixpkgs/21.11";
    nixpkgs-mozilla.url = "github:mozilla/nixpkgs-mozilla";
  };

  outputs = {
    self,
    nixpkgs,
    dfinity-sdk,
    flake-utils,
    naersk,
    nixpkgs-mozilla,
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
        mozilla = pkgs.callPackage (nixpkgs-mozilla + "/package-set.nix") {};
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
              packages.fatfs-example
            ];
          } ''
            touch $out
          '';

          packages.icfs = buildRustPackage "icfs";
          packages.fatfs-example = buildRustPackage "fatfs_example";

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
