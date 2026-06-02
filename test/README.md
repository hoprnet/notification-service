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
