# Test payloads

Send any of these against a locally running service (`just run`).

## Alerts

### KubePodCrashLooping — firing

```bash
curl -X POST http://localhost:8080/alerts \
  -H "Content-Type: application/json" \
  -d @test/KubePodCrashLooping-01.json
```

### KubePodCrashLooping — resolved

```bash
curl -X POST http://localhost:8080/alerts \
  -H "Content-Type: application/json" \
  -d @test/KubePodCrashLooping-02.json
```

### KubeContainerWaiting

```bash
curl -X POST http://localhost:8080/alerts \
  -H "Content-Type: application/json" \
  -d @test/KubeContainerWaiting-01.json
```

## Incidents

### incident-example

```bash
curl -X POST http://localhost:8080/incidents \
  -H "Content-Type: application/json" \
  -d @test/incident-example.json
```

## Reminders

`POST /reminders` accepts a pre-built digest of currently open alerts and
incidents (e.g. produced by a K8s CronJob that queries the Keep database) and
posts it to Zulip as a single Markdown-table message. The destination stream,
topic, and environment label are fixed per-deployment via
`ZULIP_REMINDER_STREAM` (default `Town Square`), `ZULIP_REMINDER_TOPIC`
(default `daily updates`), and `ENVIRONMENT_NAME` — set in the Helm chart,
not part of the request.

### reminder-example — open alerts and an open incident

```bash
curl -X POST http://localhost:8080/reminders \
  -H "Content-Type: application/json" \
  -d @test/reminder-example.json
```
