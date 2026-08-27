# SAFE-HAVEN Load Testing and Scaling Report

## Executive Summary

SAFE-HAVEN has two distinct scaling surfaces:

1. **On-chain execution:** Soroban instruction, CPU, storage, and transaction limits for deposits, withdrawals, pagination, and batch calls.
2. **Off-chain access:** RPC request volume, concurrency, latency, retries, and frontend rendering when many deposits are displayed.

This document defines the current capacity assumptions, a repeatable test plan, acceptance thresholds, and the measurements required before production capacity is claimed. No benchmark numbers in this report should be treated as measured until the test runs are recorded in the results table.

## Current Capacity Baseline

| Area | Current behavior or limit | Scaling implication |
|---|---|---|
| Deposit batch queries | `MAX_BATCH_SIZE = 25` | Split larger requests into sequential or rate-limited batches. |
| Batch query instruction cost | Reads up to 25 entries per call | Test `25` as the supported ceiling and measure headroom. |
| Emergency recovery | Per-entry removal, transfer, and event emission | Keep recovery batches bounded; benchmark separately if a batch endpoint is introduced. |
| Depositor enumeration | Append-only depositor index with stale entries filtered at read time | `get_depositors` cost can grow with historical addresses, not only active addresses. |
| Flat deposit pagination | Walks depositor and active-ID lists | Measure instruction cost as both depositor count and deposits-per-depositor grow. |
| Storage TTL | Persistent records are bumped toward `BUMP_TARGET` | Verify long-lock records remain readable through the maximum lock duration. |
| Frontend deposit loading | One ID query, one ledger-time query, then one batch query per 25 IDs | RPC calls scale approximately as `2 + ceil(deposit_count / 25)`. |

### Derived RPC call estimates

| Deposits | Batch calls | Approximate initial RPC calls |
|---:|---:|---:|
| 1-25 | 1 | 3 |
| 26-50 | 2 | 4 |
| 51-75 | 3 | 5 |
| 100 | 4 | 6 |
| 200 | 8 | 10 |

These estimates exclude wallet calls, transaction submission, retries, indexer requests, and other application traffic.

## Workload Matrix

Run each workload on a clean local network and repeat against a representative testnet deployment where possible.

| ID | Workload | Dataset / rate | Primary measurements |
|---|---|---|---|
| W1 | Single-user deposit lifecycle | 1, 25, 50, 100 deposits | Contract latency, instructions, storage growth, RPC calls |
| W2 | Many depositors | 10, 100, 1,000 active depositors | `get_depositors` latency and instruction growth |
| W3 | Concentrated deposits | 1 depositor with 25, 100, 500 active deposits | `get_deposit_ids`, batch fetching, pagination cost |
| W4 | Flat deposit pagination | 100, 1,000, 5,000 total deposits | `get_deposits_page` cost by offset and limit |
| W5 | Mixed deposit types | 50% timestamp, 50% ledger deposits | Lookup, time remaining, withdrawal, and indexing behavior |
| W6 | Concurrent reads | 10, 50, 100 clients; 1-20 req/s/client | RPC p50/p95/p99 latency, errors, throttling |
| W7 | Transaction burst | 1, 5, 10 deposits/sec | submission latency, confirmation latency, failed transactions |
| W8 | Recovery operations | 1, 10, 25 entries | Admin operation cost, event throughput, failure isolation |
| W9 | Long-running retention | Records held through TTL threshold and target | Readability, TTL extension, storage expiration |
| W10 | Failure and retry storm | Inject RPC timeouts, 429s, node restarts | Duplicate safety, retry behavior, user-visible recovery |

## Test Method

### Environment

Record the following for every run:

- Git commit and optimized WASM hash
- Rust, Soroban SDK, and Stellar CLI versions
- Local node or network name and protocol version
- RPC endpoint and configured rate limits
- CPU, memory, and storage allocated to the node/RPC service
- Number of clients, client region, and connection settings
- Token type, token decimals, and account funding strategy

### Instrumentation

Capture metrics from the contract client, RPC layer, and browser or load generator:

- Request count by contract method and HTTP status
- End-to-end latency: p50, p95, p99, and maximum
- Simulation latency versus submission/confirmation latency
- Retry count, timeout count, and 429/5xx rate
- Soroban instruction consumption and budget failures
- Ledger footprint/read-write entries and transaction size
- Node CPU, memory, disk, database, and network utilization
- Frontend time to first deposit and time to complete deposit list
- Browser main-thread time and heap growth for 100, 200, and 500 deposits

Do not count a failed request as successful capacity. Report both offered load and completed load.

## Acceptance Criteria

These are proposed production gates and must be confirmed with product and infrastructure owners before release.

### Contract execution

- [ ] 25-item batch queries complete with at least 30% instruction-budget headroom.
- [ ] Batch queries at 26 items are rejected or split before contract invocation; no oversized call reaches the node.
- [ ] `get_depositors` and `get_deposits_page` remain below the agreed instruction budget at the largest supported dataset.
- [ ] No load case produces arithmetic overflow, storage-key collision, or inconsistent active-deposit state.
- [ ] Failed token transfers and failed simulations leave no partial lifecycle or analytics state.
- [ ] Long-lived deposits remain queryable through the configured maximum lock duration.

### RPC and frontend

- [ ] Typical 20-deposit dashboard load uses no more than 3-4 initial contract RPC calls.
- [ ] At the planned steady-state rate, RPC error rate is below 1%, with 429 responses below 0.1%.
- [ ] p95 read latency is below 1 second and p99 is below 3 seconds under normal offered load.
- [ ] A 100-deposit dashboard completes within 3 seconds on the supported client and network profile.
- [ ] Retry logic uses bounded exponential backoff and does not create duplicate submissions.
- [ ] The UI remains responsive while loading the largest supported deposit set.

### Operational resilience

- [ ] A node restart does not corrupt client retry behavior or duplicate user actions.
- [ ] Load shedding is visible and recoverable when RPC capacity is exceeded.
- [ ] Alerts exist for RPC error rate, p95 latency, instruction-budget failures, and storage/CPU saturation.
- [ ] Capacity limits and supported maximums are documented for operators and users.

## Execution Checklist

1. Build the exact release artifact with `make build` and record its hash.
2. Start a local Soroban network and deploy a fresh contract.
3. Seed accounts and token balances using deterministic test data.
4. Generate W1-W5 datasets with known counts and mixed unlock types.
5. Run read workloads at increasing concurrency: 1, 10, 50, and 100 clients.
6. Run write workloads at increasing rates until an acceptance threshold fails.
7. Collect RPC, node, transaction, and browser measurements for each run.
8. Repeat boundary cases at 24, 25, 26, 50, 100, 200, and 500 deposits.
9. Repeat the highest supported workload after a node restart and injected RPC failures.
10. Attach raw logs and dashboards to the result record before approving scale.

## Tooling

The repository provides the following starting points:

```bash
make build
make test
make smoke-test-local
make check-wasm-size
```

For frontend measurements:

```bash
cd frontend
npm install
npm run test
npm run build
```

Use browser DevTools or an HTTP-capable load generator to count RPC requests and capture latency distributions. For contract-level measurements, use Soroban simulation output or node diagnostics that expose instruction and ledger-entry usage. If those values are unavailable, mark the run as incomplete rather than estimating them from wall-clock time.

## Scaling Recommendations

### Near term

- Keep the 25-entry batch ceiling and enforce it in every client.
- Add a shared RPC queue with bounded concurrency and exponential backoff.
- Cache immutable deposit fields and ledger time briefly in the frontend.
- Prefer paginated reads over loading all historical depositors or deposits.
- Monitor active versus stale depositor-index entries because append-only history affects read cost.

### Before larger datasets

- Benchmark `get_depositors` and `get_deposits_page` against historical-index growth, not just active count.
- Define a supported maximum for active deposits per account and total active depositors.
- Consider a server-side/indexer read model for analytics and historical reporting instead of scanning contract storage for every dashboard.
- Add a dedicated, bounded batch path for ledger-based deposits if they become common in the frontend.
- Add load-test automation to CI for batch boundaries and WASM size/instruction regressions.

### Horizontal scaling boundary

Contract state is serialized by the Stellar/Soroban network; adding frontend or RPC workers increases read capacity but does not remove per-transaction contract limits. Scale RPC clients horizontally only after applying shared rate limits, request deduplication, and backpressure. Scale analytics/indexing separately from the contract execution path.

## Results Log

| Run | Commit/WASM | Workload | Offered load | Completed load | p95 | p99 | Error/429 rate | Instructions | Result |
|---|---|---|---:|---:|---:|---:|---:|---:|---|
| | | | | | | | | | |

## Known Measurement Gaps

- No benchmark results are committed by this report; the listed thresholds are release targets.
- The current environment must provide Rust/Cargo and a Soroban node before contract instruction and transaction-capacity measurements can be collected.
- Testnet results vary with RPC provider throttling and network conditions; record provider-specific limits with every run.
