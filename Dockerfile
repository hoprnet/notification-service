# ── Build stage ───────────────────────────────────────────────────────────────
# Compiles the Rust binary targeting musl for a fully static executable.
FROM rust:alpine AS builder

WORKDIR /app

# musl-dev is required to link against the musl C library on Alpine.
RUN apk add --no-cache musl-dev

# 1. Copy only the manifest files first so Docker can cache the dependency
#    compilation layer independently of source changes.
COPY Cargo.toml ./
# Cargo.lock is optional locally but should be present in CI for reproducible builds.
COPY Cargo.lock* ./

# 2. Build a dummy binary to compile and cache all dependencies.
RUN mkdir src && echo 'fn main() {}' > src/main.rs \
    && cargo build --release \
    && rm -rf src

# 3. Copy the real source and rebuild only our crate (deps are already cached).
COPY src ./src
RUN touch src/main.rs && cargo build --release

# ── Runtime stage ─────────────────────────────────────────────────────────────
# Minimal Alpine image — no Rust toolchain, no build artifacts.
FROM alpine:3.21

LABEL description="Hopr Notification Service"

WORKDIR /app

# tini: proper PID-1 / signal forwarding.
# ca-certificates: needed for outbound TLS connections (e.g. future Zulip integration).
RUN apk add --no-cache tini ca-certificates

# Run as a non-root user.
RUN addgroup -S app && adduser -S app -G app

COPY --from=builder /app/target/release/notification-service /app/notification-service

ENV PORT=8080
ENV ZULIP_EMAIL=""
ENV ZULIP_API_KEY=""
ENV ZULIP_HOST=""
EXPOSE ${PORT}

USER app

ENTRYPOINT ["/sbin/tini", "--"]
CMD ["/app/notification-service"]
