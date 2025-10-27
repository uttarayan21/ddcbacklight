{
  description = "A simple rust flake using rust-overlay and craneLib";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    nix-github-actions = {
      url = "github:nix-community/nix-github-actions";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = {
    self,
    crane,
    flake-utils,
    nixpkgs,
    rust-overlay,
    advisory-db,
    nix-github-actions,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            rust-overlay.overlays.default
            # (final: prev: {
            #   ddcbacklight = ddcbacklight-overlay.packages.${system}.ddcbacklight.override {   };
            # })
          ];
        };
        inherit (pkgs) lib;

        stableToolchain = pkgs.rust-bin.stable.latest.default;
        stableToolchainWithLLvmTools = pkgs.rust-bin.stable.latest.default.override {
          extensions = ["rust-src" "llvm-tools"];
        };
        stableToolchainWithRustAnalyzer = pkgs.rust-bin.stable.latest.default.override {
          extensions = ["rust-src" "rust-analyzer"];
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain stableToolchain;
        craneLibLLvmTools = (crane.mkLib pkgs).overrideToolchain stableToolchainWithLLvmTools;

        src = let
          filterBySuffix = path: exts: lib.any (ext: lib.hasSuffix ext path) exts;
          sourceFilters = path: type: (craneLib.filterCargoSources path type) || filterBySuffix path [".c" ".h" ".hpp" ".cpp" ".cc"];
        in
          lib.cleanSourceWith {
            filter = sourceFilters;
            src = ./.;
          };
        commonArgs =
          {
            inherit src;
            stdenv = p: p.clangStdenv;
            pname = "ddcbacklight";
            doCheck = false;
            LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
            # nativeBuildInputs = with pkgs; [
            #   cmake
            #   llvmPackages.libclang.lib
            # ];
            buildInputs = with pkgs;
              [ddcutil]
              ++ (lib.optionals pkgs.stdenv.isDarwin [
                libiconv
                # darwin.apple_sdk.frameworks.Metal
              ]);
            nativeBuildInputs = with pkgs; [pkg-config];
          }
          // (lib.optionalAttrs pkgs.stdenv.isLinux {
            # BINDGEN_EXTRA_CLANG_ARGS = "-I${pkgs.llvmPackages.libclang.lib}/lib/clang/18/include";
          });
        cargoArtifacts = craneLib.buildPackage commonArgs;
      in {
        checks =
          {
            ddcbacklight-clippy = craneLib.cargoClippy (commonArgs
              // {
                inherit cargoArtifacts;
                cargoClippyExtraArgs = "--all-targets -- --deny warnings";
              });
            ddcbacklight-docs = craneLib.cargoDoc (commonArgs // {inherit cargoArtifacts;});
            ddcbacklight-fmt = craneLib.cargoFmt {inherit src;};
            ddcbacklight-toml-fmt = craneLib.taploFmt {
              src = pkgs.lib.sources.sourceFilesBySuffices src [".toml"];
            };
            # Audit dependencies
            ddcbacklight-audit = craneLib.cargoAudit {
              inherit src advisory-db;
            };

            # Audit licenses
            ddcbacklight-deny = craneLib.cargoDeny {
              inherit src;
            };
            ddcbacklight-nextest = craneLib.cargoNextest (commonArgs
              // {
                inherit cargoArtifacts;
                partitions = 1;
                partitionType = "count";
              });
          }
          // lib.optionalAttrs (!pkgs.stdenv.isDarwin) {
            ddcbacklight-llvm-cov = craneLibLLvmTools.cargoLlvmCov (commonArgs // {inherit cargoArtifacts;});
          };

        packages = rec {
          ddcbacklight = craneLib.buildPackage (commonArgs
            // {inherit cargoArtifacts;}
            // {
              meta = with lib; {
                homepage = "https://github.com/uttarayan21/ddcbacklight";
                description = "DDC/CI backlight control";
                license = licenses.gpl3;
                maintainers = with maintainers; [uttarayan21];
                platforms = platforms.linux;
                mainProgram = "xbacklight";
              };
              postInstall = ''
                mkdir -p $out/bin
                mkdir -p $out/share/bash-completions
                mkdir -p $out/share/fish/vendor_completions.d
                mkdir -p $out/share/zsh/site-functions
                $out/bin/xbacklight completions bash > $out/share/bash-completions/xbacklight.bash
                $out/bin/xbacklight completions fish > $out/share/fish/vendor_completions.d/xbacklight.fish
                $out/bin/xbacklight completions zsh > $out/share/zsh/site-functions/_xbacklight
              '';
            });
          default = ddcbacklight;
        };

        devShells = {
          default =
            pkgs.mkShell.override {
              stdenv = pkgs.clangStdenv;
            } {
              LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
              nativeBuildInputs = with pkgs; [pkg-config];
              buildInputs = with pkgs; [ddcutil];
              packages = with pkgs; [
                stableToolchainWithRustAnalyzer
                cargo-nextest
                cargo-deny
                ddcutil
                pkg-config
                rust-bindgen
                cargo-outdated
              ];
            };
        };
      }
    )
    // {
      githubActions = nix-github-actions.lib.mkGithubMatrix {
        checks = nixpkgs.lib.getAttrs ["x86_64-linux"] self.checks;
      };
    };
}
