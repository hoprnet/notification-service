# notification-service

A lightweight, high-performance alert notification relay service written in **Rust**. It exposes a REST API that receives alert payloads, enriches them with processing metadata, and dispatches them to a target notification system.

## Architecture

```
                ┌────────────────────────────────────────────────┐
                │              notification-service              │
                │                                                │
Alert sources   │   POST /alerts                                 │
(Keep, etc.) ──►│   ──────────────► Processor ──► Output adapter │──► Zulip
                │                                                │
                └────────────────────────────────────────────────┘
```

## Development

### Rust

```bash
# Set environment variables
cp .env.example .env

# Build and run
just run

# Send a test alert to the running service
just test-alert
```

### Docker

```bash
just docker-build            # Build image tagged :latest
just docker-build 1.2.3      # Build image with a specific tag
just docker-push             # Push to the registry :latest
just docker-push 1.2.3       # Push to the registry
```


### Helm

```bash
# Lint / dry-run (no cluster needed)
just helm-lint
just helm-template

# Install into the default namespace
just helm-install

# Upgrade an existing release
just helm-upgrade

# Uninstall
just helm-uninstall
```

---

## License

GPL-3.0
