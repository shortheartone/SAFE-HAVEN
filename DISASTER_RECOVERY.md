# SAFE-HAVEN Disaster Recovery Runbook

This runbook covers incidents affecting a deployed SAFE-HAVEN Soroban contract and its operators. It is an operational procedure, not a substitute for an independent smart-contract audit.

Use [MONITORING.md](MONITORING.md) for the recurring health checks and alerts that should detect these incidents early.

## Scope and Assumptions

- The contract is deployed on Stellar/Soroban and identified by a contract ID.
- The deployed contract exposes `pause`, `unpause`, `emergency_withdraw`, and two-step admin transfer.
- The contract has no upgrade entry point. A fixed implementation must be deployed as a new contract and users must move funds through supported contract calls.
- `emergency_withdraw` sends the recorded deposit amount to the depositor. It does not send funds to the admin.
- Stellar ledger history is append-only. “Data corruption” means an application, indexer, RPC, or storage-observation inconsistency unless ledger evidence shows otherwise.
- Never put secret keys, seed phrases, signed XDR, or unredacted user data in tickets, chat, logs, or Git.

## Roles and Responsibilities

| Role | Responsibilities | Decision authority |
|---|---|---|
| Incident commander | Opens the incident, assigns tasks, maintains the timeline, chooses severity, and approves closure. | Coordinates all response decisions. |
| Contract operator | Controls the admin signer, pauses the contract, runs read-only checks, and submits emergency recovery transactions. | Contract state changes, subject to approval policy. |
| Security lead | Validates exploitability, preserves evidence, coordinates disclosure, and approves containment and remediation. | Security severity and disclosure path. |
| Release/deployment engineer | Builds, tests, audits, deploys, and verifies a replacement contract and frontend configuration. | Release readiness and deployment execution. |
| Data/reconciliation owner | Compares ledger events, contract queries, RPC responses, and application indexes; produces the affected-deposit ledger. | Reconciliation sign-off. |
| Communications owner | Sends user, partner, and status updates using approved channels. | External messaging. |
| Depositor support | Handles identity and transaction questions without requesting private keys; tracks claims and returned funds. | Case intake and user follow-up. |

For a small team, one person may hold multiple roles, but the incident commander, security lead, and transaction approver should be separate people for a material incident whenever possible.

## Severity and Escalation

| Severity | Trigger | Initial response | Update cadence |
|---|---|---|---|
| SEV-1 | Active or likely fund loss, private admin key compromise, or contract-wide integrity failure. | Pause immediately; page security lead and incident commander. | Every 30 minutes until contained. |
| SEV-2 | Contract vulnerability with no confirmed loss, repeated failed withdrawals, or material data inconsistency. | Restrict deposits, preserve evidence, and start reconciliation. | Every 2 hours. |
| SEV-3 | Isolated UI, RPC, indexer, or documentation issue with no evidence of incorrect on-chain state. | Route to the owning engineer and monitor. | Daily until resolved. |

Escalate immediately to SEV-1 when any of the following is observed: an unauthorized admin transaction, a token balance decrease not explained by a valid withdrawal, a compromised signer, or evidence that users cannot recover funds through the documented contract paths. Notify the security contact in [SECURITY.md](SECURITY.md) privately; replace its placeholder address before production use.

The incident commander may declare containment only after the security lead confirms that no new unauthorized state changes are occurring and the data owner records the last verified ledger. Public disclosure, user remediation, and unpausing require security and incident-commander approval.

## Common First Response

1. Declare the incident in the private incident channel and record UTC time, reporter, contract ID, network, symptoms, and suspected first affected ledger.
2. Freeze unrelated deployments and frontend configuration changes.
3. Preserve evidence: transaction hashes, operation XDR where safe, RPC responses, contract events, logs, frontend error output, current WASM checksum, and the deployed contract ID. Store artifacts read-only with restricted access.
4. Verify the network and contract ID before signing anything. Use read-only queries first.
5. Confirm the current admin, pending admin, paused state, contract initialization state, and contract/token balances from more than one trusted RPC or explorer when possible.
6. Require a second operator to review every recovery transaction’s destination, amount, contract ID, network passphrase, and simulation result.
7. Keep a transaction ledger containing: UTC time, operator, reviewer, function, depositor, deposit ID, token, amount, transaction hash, result, and evidence link.

## Scenario A: Contract Vulnerability or Active Exploit

### Containment

1. Set severity to SEV-1 unless the security lead confirms there is no plausible fund-loss path.
2. Stop frontend deposits and any automated submissions. If the admin signer is available, call `pause(admin)` and verify `is_paused() == true` from the network.
3. Do not call the suspected vulnerable function again except as part of an approved reproduction in an isolated environment. Do not attempt an unreviewed patch against production state.
4. Identify the vulnerable version, affected entry points, token addresses, affected depositor IDs, first and last suspect ledgers, and the remaining contract balance for each token.
5. Capture all relevant events and transactions before changing state. Keep the original contract ID active for evidence and read-only reconciliation.

### Recovery and remediation

1. The security lead writes a short impact assessment and an exploit/reproduction test. The release engineer adds a regression test and runs `make fmt-check`, `make lint`, and `make test`.
2. If the admin is trusted and the contract’s recovery path is safe, create an affected-deposit list from on-chain state. For each confirmed deposit, independently verify depositor, ID, token, and amount before calling `emergency_withdraw(admin, depositor, deposit_id)`.
3. Submit recovery calls in small reviewed groups. After each transaction, verify the event, recipient balance, removed deposit state, and remaining contract token balance. Never substitute an admin-controlled destination: the contract is designed to return funds to the depositor.
4. If emergency withdrawal is unsafe or the admin is compromised, do not sign recovery transactions. Escalate to the security lead and incident commander for a coordinated disclosure and a chain-specific legal/technical response; the contract cannot be upgraded in place.
5. Build and deploy a reviewed replacement contract with a new contract ID. Initialize it with the approved admin and fee recipient, then verify `get_admin()`, `get_constants()`, `is_initialized()`, and `is_paused()`.
6. Update the frontend contract ID only after deployment verification. Keep the replacement paused until smoke tests, reconciliation, and communications are complete.
7. Publish migration instructions that require users to sign their own deposits or withdrawals. Do not ask users for secret keys. Record old and new transaction hashes.
8. Leave the vulnerable contract paused when possible. Unpause it only if the security lead documents why it is safe and the incident commander approves it.

### Exit criteria

- No unauthorized transactions after the containment ledger.
- Impacted deposits have a disposition: returned, migrated, or explicitly unresolved with an owner.
- Regression tests and deployment verification pass.
- Users have received an accurate status and recovery path.

## Scenario B: Data Corruption or State Mismatch

### Triage

1. Treat an indexer or frontend mismatch as a data incident, not a contract incident, until ledger evidence proves otherwise. Do not pause the contract solely because a dashboard disagrees with the chain.
2. Compare the same deposit and token across at least two independent RPC providers, a block explorer, and the application index. Record the ledger sequence used for each observation.
3. Identify the earliest ledger where the observations diverge and the last ledger where they agree.
4. Preserve raw RPC responses, indexer query output, application logs, and the deployed frontend build hash.

### Recovery

1. If the contract state is correct and only the index is wrong, rebuild the index from the last agreed ledger. Do not submit on-chain transactions to fix an off-chain view.
2. If the contract state is wrong, escalate to Scenario A and follow its containment and recovery steps.
3. Reconcile every affected deposit against on-chain state before restoring user-facing balances or statuses.
4. Verify the rebuilt index against a second RPC provider before re-enabling deposits or withdrawals in the frontend.

### Exit criteria

- The index and contract state agree at a recorded ledger.
- The root cause is documented with an owner and a follow-up fix.
- Users see accurate balances and statuses.

## Scenario C: Admin Key Compromise

1. Set severity to SEV-1 and pause the contract if the compromised signer still has authority.
2. If the admin is not yet compromised but a pending admin is suspect, cancel or complete the two-step transfer only with security-lead approval and a reviewed transaction.
3. Rotate the admin signer to a new, independently held key using the two-step transfer. Verify `get_admin()` and `get_pending_admin()` after each step.
4. Treat all transactions signed by the compromised key after the suspected compromise ledger as unauthorized until proven otherwise.
5. Follow Scenario A for any resulting fund movement and user remediation.

## Scenario D: RPC or Infrastructure Outage

1. Confirm the outage is external by checking multiple RPC providers and the network status page.
2. Keep the contract state unchanged. Do not pause for a read-only outage unless withdrawals are failing in a way that risks funds.
3. Switch the frontend and indexer to a healthy RPC provider and record the change.
4. Communicate degraded service to users and restore normal operation once providers recover.

## Backup and Recovery of Operational Data

On-chain state is authoritative and does not require backups. Operational data that must be backed up includes the application index/database, deployment configuration, and incident evidence.

### Automated daily backups

- A scheduled job runs daily and produces an encrypted database dump plus the current deployment configuration.
- Backups are written to durable, access-controlled storage separate from the primary database host.
- Each backup records a UTC timestamp, checksum, size, and the source database version.
- Backups are encrypted at rest and in transit; encryption keys are stored separately from the backup data.

### Retention policy

| Tier | Retention | Purpose |
|---|---|---|
| Daily | 30 days | Routine recovery and short-term rollback. |
| Weekly | 12 weeks | Medium-term recovery and trend analysis. |
| Monthly | 12 months | Compliance and audit evidence. |

Retention meets the compliance requirement of at least 12 months of monthly backups. Expired backups are deleted automatically by the retention job; deletion is logged.

### Backup monitoring and alerts

- The backup job emits a success or failure signal on completion.
- An alert fires if a daily backup does not complete successfully within its scheduled window, if the backup checksum fails verification, or if no successful backup exists in the last 26 hours.
- Alerts route to the on-call engineer and the data/reconciliation owner.
- A failed backup is treated as a SEV-2 data incident until a verified backup exists.

### Restoration procedure

1. Select the most recent backup that passes checksum verification.
2. Restore into an isolated environment, never directly over production.
3. Verify row counts, key tables, and a sample of deposits against on-chain state at the backup’s recorded ledger.
4. Record the restore start and end time; restoration must complete in under 1 hour.
5. Promote the restored database only after reconciliation sign-off.

### Monthly restoration test

- On the first business day of each month, restore the latest monthly backup into an isolated environment and run the verification steps above.
- Record the test date, operator, backup used, restore duration, and verification result in the incident log.
- A failed or overdue restoration test is a SEV-2 data incident and must be resolved before the next monthly cycle.

## Communications

- Use approved channels only. Do not speculate about cause, impact, or timelines before the security lead confirms them.
- Provide users with the contract ID, affected functions, and the exact recovery steps they must take themselves.
- Never request secret keys, seed phrases, or signed transactions from users.

## Post-Incident Review

1. Hold a blameless review within five business days of closure.
2. Document the timeline, root cause, impact, detection gaps, and follow-up actions with owners and due dates.
3. Update this runbook, [MONITORING.md](MONITORING.md), and tests based on lessons learned.
