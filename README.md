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

## Test

```bash
curl -X POST http://localhost:8080/alerts \
  -H "Content-Type: application/json" \
  -d @test/alert-notification.json
```

---

## Configuration

| Environment variable | Default | Description |
|---|---|---|
| `PORT` | `8080` | TCP port the HTTP server listens on |
| `RUST_LOG` | `info` | Log level filter (`debug`, `info`, `warn`, `error`) |
| `ZULIP_EMAIL` | — | Zulip bot e-mail address (API authentication) |
| `ZULIP_API_KEY` | — | Zulip bot API key |
| `ZULIP_HOST` | — | Zulip server hostname (e.g. `yourorg.zulipchat.com`) |

---

## Development

### Rust

```bash
# Build and run
just run

# Send a test alert to the running service
just test-alert
```

### Docker

```bash
just docker-build            # Build image tagged :latest
just docker-build 1.2.3      # Build image with a specific tag
just docker-push 1.2.3       # Push to the registry
just docker-release 1.2.3    # Build + push in one step
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
