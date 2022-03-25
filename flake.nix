{
  inputs = {
    dfinity-sdk = {
      url = "github:paulyoung/nixpkgs-dfinity-sdk";
      flake = false;
    };
    ic-repl-src = {
      url = "github:chenyan2002/ic-repl";
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
    ic-repl-src,
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
          channel = "1.56.0";
          sha256 = "L1e0o7azRjOHd0zBa+xkFnxdFulPofTedSTEYZSjj2s=";
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

        buildRustPackage = name: root: attrs:
          let
            packageArgs = [
              "--package" name
            ];
          in
            naersk-lib.buildPackage ({
              inherit root;
              compressTarget = true;
              copyBins = true;
              copyLibs = true;
              copyTarget = true;
            } // attrs);

        buildLocalRustPackage = name:
          let
            options = [
              "--package" name
              "--target" "wasm32-unknown-unknown"
            ];
          in
            buildRustPackage name ./. {
              cargoBuildOptions = x: x ++ options;
              cargoTestOptions = x: x ++ options;
              copyBins = false;
            }
        ;

        ic-repl =
          buildRustPackage "ic-repl" ic-repl-src {
            buildInputs = [
              pkgs.libiconv

              # https://nixos.wiki/wiki/Rust#Building_the_openssl-sys_crate
              pkgs.openssl_1_1
              pkgs.pkgconfig
            ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.darwin.apple_sdk.frameworks.Security
            ];
          };

        buildExampleTest = name: package: pkgs.runCommand "${name}-example-test" {
          buildInputs = [
            dfinitySdk
            ic-repl
            pkgs.jq
          ];
        } ''
          HOME=$TMP
          cp -R ${package}/. result
          mkdir -p examples/${name}
          cp -R ${./examples}/${name}/. examples/${name}

          cp ${./dfx.json} dfx.json
          jq '.canisters = (.canisters | map_values(.build = "echo"))' dfx.json > new.dfx.json
          mv new.dfx.json dfx.json

          dfx start --background
          dfx deploy
          ic-repl --replica local examples/${name}/test.ic-repl
          dfx stop

          touch $out
        '';
      in
        rec {
          # `nix build`
          defaultPackage = pkgs.runCommand "all" {
            buildInputs = pkgs.lib.attrValues packages;
          } ''
            touch $out
          '';

          packages.icfs = buildLocalRustPackage "icfs";
          packages.icfs-fatfs = buildLocalRustPackage "icfs-fatfs";
          packages.fatfs-example = buildLocalRustPackage "fatfs-example";

          packages.fatfs-example-test = buildExampleTest "fatfs" packages.fatfs-example;

          # `nix develop`
          devShell = pkgs.mkShell {
            buildInputs = [
              dfinitySdk
              ic-repl
            ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.libiconv
            ];

            # supply the specific rust version
            nativeBuildInputs = [ rust ];
          };
        }
    );
}
