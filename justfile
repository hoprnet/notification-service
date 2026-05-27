# notification-service — task runner
# https://github.com/casey/just
#
# Install: cargo install just  or  brew install just
# Usage:   just <recipe>

# Default: list all available recipes
default:
    @just --list

# ── Development ───────────────────────────────────────────────────────────────

# Build (debug)
build:
    cargo build

# Build (release / optimised)
build-release:
    cargo build --release

# Run the service locally (reads PORT / RUST_LOG env vars if set)
run:
    cargo run

# Run the full test suite
test:
    cargo test

# Format source code with rustfmt
fmt:
    cargo fmt

# Lint with clippy (treat warnings as errors)
lint:
    cargo clippy -- -D warnings

# Type-check without producing binaries
check:
    cargo check

# Send a test alert to a locally running instance (pass file= to override)
test-alert file="test/KubePodCrashLooping-01.json":
    curl -s -X POST http://localhost:8080/alerts \
      -H "Content-Type: application/json" \
      -d @{{file}} | jq .

# ── Docker ────────────────────────────────────────────────────────────────────

registry := "europe-west3-docker.pkg.dev/hoprassociation/docker-images/notification-service"

# Build Docker image (version defaults to "latest")
docker-build version="latest":
    docker build -t {{registry}}:{{version}} .

# Push Docker image to the registry
docker-push version="latest":
    docker push {{registry}}:{{version}}

# Build and push in one step
docker-release version: (docker-build version) (docker-push version)

# ── Helm ──────────────────────────────────────────────────────────────────────

chart_dir := "charts/notification-service"

# Lint the Helm chart
helm-lint:
    helm lint {{chart_dir}}

# Render the Helm templates (dry-run, no cluster needed)
helm-template:
    helm template notification-service {{chart_dir}}

# Install the chart into a namespace (default: "default")
helm-install namespace="default":
    helm install notification-service {{chart_dir}} \
      --namespace {{namespace}} \
      --create-namespace

# Upgrade an existing Helm release
helm-upgrade namespace="default":
    helm upgrade notification-service {{chart_dir}} \
      --namespace {{namespace}}

# Uninstall the Helm release
helm-uninstall namespace="default":
    helm uninstall notification-service --namespace {{namespace}}

# Package the chart into a .tgz archive
helm-package:
    helm package {{chart_dir}}

# Package and push to an OCI registry
helm-publish registry version: helm-package
    helm push notification-service-*.tgz oci://{{registry}}
