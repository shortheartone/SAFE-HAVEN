# SAFE-HAVEN Documentation

Welcome to the SAFE-HAVEN documentation. SAFE-HAVEN is a production-ready
decentralized vault on the Stellar blockchain (Soroban) that lets you lock
tokens until a future timestamp with configurable early-exit penalties.

> **Quick links:**  
> [Get Started](#getting-started) · [API Reference](./API.md) · [Contributing](../CONTRIBUTING.md) · [Security](../SECURITY.md) · [Changelog](../CHANGELOG.md)

---

## Getting Started

New to SAFE-HAVEN? Start here.

| Guide | Description |
|---|---|
| [Getting Started](../GETTING_STARTED.md) | Full setup guide: install Rust, build the contract, deploy locally, run the frontend |
| [Quick Reference](../QUICK_REFERENCE.md) | Cheat-sheet of the most common commands |
| [Frontend README](../frontend/README.md) | How to run and configure the React frontend |
| [Faucet Guide](../FAUCET.md) | Fund testnet accounts for development and testing |

---

## User Guide

For users interacting with SAFE-HAVEN through the frontend or Stellar CLI.

| Guide | Description |
|---|---|
| [User Onboarding](./USER_ONBOARDING.md) | Step-by-step guide for first-time vault users |
| [Knowledge Base & FAQ](./KNOWLEDGE_BASE_AND_FAQ.md) | Common questions about deposits, withdrawals, penalties, and staking |
| [Support & Lifecycle](../SUPPORT.md) | Version lifecycle, maintenance matrix, bug-fix SLAs |
| [Legal](../LEGAL.md) | Terms of Service, Privacy Policy, GDPR compliance |

---

## Developer Guide

For contributors building on or extending SAFE-HAVEN.

| Guide | Description |
|---|---|
| [Contributing](../CONTRIBUTING.md) | Development environment, git workflow, PR process |
| [Coding Standards](../CODING_STANDARDS.md) | Rust and TypeScript style guide, enforcement tools |
| [Branching Strategy](../BRANCHING.md) | Branch model, naming conventions, release checklist |
| [Testing Strategy](./TESTING_STRATEGY.md) | Testing pyramid, coverage requirements, test data management |
| [Code Review Checklist](./CODE_REVIEW_CHECKLIST.md) | Review checklist, severity levels, approval process, reviewer assignment |
| [Testing Guide (RPC Batching)](../TESTING_GUIDE.md) | How to verify RPC batch optimization works correctly |
| [CI Testing Guide](../CI_TESTING_GUIDE.md) | How to run and interpret CI checks locally |
| [Makefile Reference](#makefile-reference) | All `make` targets with descriptions |

---

## Architecture & Design

Technical decisions and architectural context.

| Document | Description |
|---|---|
| [Architecture Decision Records](./adr/README.md) | Index of all ADRs — why key decisions were made |
| [ADR-001: Dual Deposit Types](./adr/ADR-001-dual-deposit-types.md) | Timestamp vs ledger-sequence deposit model |
| [ADR-002: Storage Layout](./adr/ADR-002-storage-layout.md) | Persistent storage schema and cancel semantics |
| [ADR-003: Admin Transfer](./adr/ADR-003-admin-transfer-and-renunciation.md) | Two-step admin transfer and renunciation |
| [ADR-004: WASM Size Budget](./adr/ADR-004-wasm-size-budget.md) | 64 KB WASM size limit and optimization pipeline |
| [ADR-005: Penalty Model](./adr/ADR-005-early-exit-penalty-model.md) | Early-exit penalty split (30% fee / 70% stakers) |
| [Version Compatibility](./VERSION_COMPATIBILITY.md) | SDK, toolchain, and dependency compatibility matrix |
| [Proposals](./proposals/TEMPLATE.md) | Template for proposing architectural changes |

---

## API Reference

| Document | Description |
|---|---|
| [Contract API](./API.md) | All public contract functions, parameters, return values, error codes |
| [Rust API Docs](https://kenedybok3.github.io/SAFE-HAVEN/safe_haven/) | Auto-generated `cargo doc` output (deployed on push to `main`) |

---

## Operations

For operators, deployers, and on-call engineers.

| Guide | Description |
|---|---|
| [Monitoring](../MONITORING.md) | Contract health checks, alert thresholds, TTL and storage monitoring |
| [Disaster Recovery](../DISASTER_RECOVERY.md) | Disaster scenarios, recovery procedures, escalation rules |
| [Incident Response](../INCIDENT_RESPONSE.md) | Incident classification, response steps, communication templates |
| [Postmortem Process](../POSTMORTEM.md) | Blameless post-mortem guide and template |
| [Backup](../BACKUP.md) | Contract state export and backup procedures |
| [Deployment Scripts](../scripts/) | `deploy.sh`, `smoke_test_local.sh`, `pre_deploy_check.sh` |
| [Contract Upgrade Playbook](./CONTRACT_UPGRADE_AND_MIGRATION_PLAYBOOK.md) | How to deploy a new version and migrate users |
| [Legacy Contract Migration](./LEGACY_CONTRACT_MIGRATION.md) | Migrating from older contract deployments |
| [Governance](../GOVERNANCE.md) | On-chain upgrade governance: propose, review, vote, timelock, execute |

---

## Security

| Document | Description |
|---|---|
| [Security Policy](../SECURITY.md) | Vulnerability reporting, response timeline, scope |
| [Security Education](../frontend/SECURITY_EDUCATION.md) | Frontend security guide for contributors |
| [GAS Profiling](../GAS_PROFILING.md) | Instruction budget analysis and optimization notes |

---

## Process & Roadmap

| Document | Description |
|---|---|
| [Roadmap](./ROADMAP.md) | Planned features and milestones |
| [Changelog](../CHANGELOG.md) | Release history and breaking changes |
| [Debt Register](../DEBT.md) | Tracked technical debt items |
| [Communication Process](./COMMUNICATION_PROCESS.md) | Team communication channels, decision-making, meeting cadence |
| [Annual Review & Planning](./ANNUAL_REVIEW_AND_PLANNING.md) | Yearly retrospective and planning framework |
| [Community Strategy](./COMMUNITY_STRATEGY.md) | Community engagement and ecosystem participation |

---

## Documentation Structure

```
SAFE-HAVEN/
├── README.md                           Project overview and quick start
├── CONTRIBUTING.md                     How to contribute
├── CODING_STANDARDS.md                 Code style guide
├── BRANCHING.md                        Git branching and release strategy
├── CHANGELOG.md                        Release history
├── SECURITY.md                         Vulnerability reporting
├── SUPPORT.md                          Version lifecycle and SLAs
├── MONITORING.md                       Operational monitoring
├── DISASTER_RECOVERY.md                DR runbook
├── POSTMORTEM.md                       Incident post-mortem guide
├── GOVERNANCE.md                       On-chain upgrade governance
├── LEGAL.md                            Terms of Service and privacy
│
├── docs/                               Extended documentation
│   ├── index.md                        ← You are here
│   ├── API.md                          Contract API reference
│   ├── TESTING_STRATEGY.md             Testing strategy and guidelines
│   ├── COMMUNICATION_PROCESS.md        Team communication process
│   ├── USER_ONBOARDING.md              User onboarding guide
│   ├── KNOWLEDGE_BASE_AND_FAQ.md       FAQ
│   ├── ROADMAP.md                      Feature roadmap
│   ├── VERSION_COMPATIBILITY.md        Compatibility matrix
│   ├── CONTRACT_UPGRADE_AND_MIGRATION_PLAYBOOK.md
│   ├── LEGACY_CONTRACT_MIGRATION.md
│   ├── adr/                            Architecture Decision Records
│   │   ├── README.md                   ADR index and process
│   │   ├── ADR-001-*.md
│   │   └── ...
│   └── proposals/                      Change proposals
│       └── TEMPLATE.md
│
├── contracts/safe-haven/               Rust smart contract
│   └── src/
│       └── ...
│
└── frontend/                           React TypeScript frontend
    └── ...
```

---

## Makefile Reference

Run `make help` to see all available targets, or refer to this table:

| Target | Description |
|---|---|
| `make build` | Compile the contract to WASM |
| `make test` | Run all unit tests (`cargo test --features testutils`) |
| `make watch` | Auto-run tests on file change |
| `make lint` | Run Clippy (fail on warnings) |
| `make fmt` | Format all Rust source files |
| `make check` | fmt + lint + test + audit + deny (mirrors CI) |
| `make audit` | `cargo audit` — check for known CVEs |
| `make deny` | `cargo deny check` — license and ban policy |
| `make doc` | Build and open Rust API docs |
| `make optimize` | Optimize WASM with stellar CLI |
| `make check-wasm-size` | Fail if WASM exceeds 64 KB |
| `make dev` | Full local dev: build + deploy + frontend |
| `make dev-stop` | Stop the local Stellar node |
| `make deploy-testnet` | Deploy to Stellar testnet |
| `make deploy-mainnet` | Deploy to Stellar mainnet |
| `make smoke-test-local` | E2E test against a local node |
| `make install-tools` | Install all recommended dev tools |
| `make backup` | Export and back up contract state |

---

## Was This Page Helpful?

If something is unclear, missing, or wrong in this documentation:

1. **Small fixes** (typos, broken links, outdated commands) — open a PR directly.
   No issue required.
2. **Missing topics** — open a GitHub issue with the label `documentation`.
3. **Incorrect technical content** — open an issue and tag a maintainer.

For questions about using SAFE-HAVEN, check the
[Knowledge Base & FAQ](./KNOWLEDGE_BASE_AND_FAQ.md) first.

---

*This index is maintained by the SAFE-HAVEN maintainers.  
Auto-generated Rust API docs are deployed to GitHub Pages on every push to `main`.*
