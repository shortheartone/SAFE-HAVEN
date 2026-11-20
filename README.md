# Project

## Microservices Architecture (Issue #963)

This document defines the service boundaries, communication protocol, shared library,
service discovery, and migration path for extracting services from the monolith.

### Service Boundaries

Services are split along domain boundaries so each can scale and deploy independently:

| Service | Domain | Responsibility |
|---------|--------|----------------|
| `api-gateway` | Edge | Request routing, auth, rate limiting |
| `metrics-service` | Observability | Metrics collection and aggregation |
| `config-service` | Configuration | Centralized config distribution |
| `backup-service` | Data lifecycle | Backups and disaster recovery |

Each service owns its data and exposes a versioned API. No service reaches into
another service's storage directly.

### Inter-Service Communication Protocol

- **Synchronous:** HTTP/JSON over internal network for request/response calls.
  - Versioned paths: `/v1/<resource>`
  - Standard error envelope: `{ "error": { "code": string, "message": string } }`
  - Health endpoint on every service: `GET /healthz`
- **Asynchronous:** out of scope for this issue (message queues deferred).

### Shared Library

Common functionality (config loading, error types, metrics helpers, HTTP client)
lives in a shared crate consumed by all services to avoid duplication.

### Service Discovery

- Services register themselves on startup and expose `GET /healthz`.
- The gateway resolves upstream instances via the discovery registry and performs
  health-checked load balancing.
- Discovery is environment-configurable so local and production setups differ only
  by configuration.

### Migration Path

1. Extract the first service (`metrics-service`) behind the gateway.
2. Route a percentage of traffic to the extracted service; keep the monolith as fallback.
3. Extract remaining services one at a time following the same pattern.
4. Decommission monolith routes once all traffic is served by extracted services.

### Acceptance Criteria Mapping

- Service boundaries: defined above.
- Communication protocol: documented above.
- Independent deployment: each service has its own build/deploy target.
- Service discovery: registry + health checks described above.
- First service extracted: `metrics-service`.
