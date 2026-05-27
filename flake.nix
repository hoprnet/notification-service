{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

        # Shell application that builds the Docker image for the notification-service.
        # The image is tagged as `notification-service:latest` so the hopr-workflows
        # build-docker reusable workflow can locate, re-tag, and push it.
        dockerBuild = pkgs.writeShellApplication {
          name = "dockerBuild";
          runtimeInputs = [
            pkgs.docker
            pkgs.coreutils
          ];
          text = ''
            set -euo pipefail

            echo "[+] Building: notification-service:latest"
            docker build --platform linux/amd64 -t notification-service:latest .
            echo "[✓] Done: notification-service:latest"
          '';
        };

      in
      {
        # devShells.ci is loaded by the build-docker reusable workflow for any
        # pre/post-build tooling. Keep it lean — add tools here as needed.
        devShells.ci = pkgs.mkShell {
          packages = [ ];
        };

        # Default development shell — provides all tools required to build and
        # work on notification-service locally.
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            clippy
            rustfmt
            just
            helm
            docker
          ];

          shellHook = ''
            echo "Development environment loaded:"
            echo "  Rust:  $(rustc --version)"
            echo "  Cargo: $(cargo --version)"
            echo "  Just:  $(just --version)"
            echo "  Helm:  $(helm version --short)"
          '';
        };

        # Nix apps exposed to the build-docker reusable workflow.
        # `nix run .#docker-x86_64-linux` is invoked as the build_command.
        apps = rec {
          docker-x86_64-linux = {
            type = "app";
            program = "${dockerBuild}/bin/dockerBuild";
          };
          default = docker-x86_64-linux;
        };
      }
    );
}
