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
| SEV-4 | Cosmetic or informational issue with no user or fund impact (typos, stale docs, non-blocking warnings). | Log and schedule in normal backlog. | Weekly until resolved. |

Escalate immediately to SEV-1 when any of the following is observed: an unauthorized admin transaction, a token balance decrease not explained by a valid withdrawal, a compromised signer, or evidence that users cannot recover funds through the documented contract paths. Notify the security contact in [SECURITY.md](SECURITY.md) privately; replace its placeholder address before production use.

The incident commander may declare containment only after the security lead confirms that no new unauthorized state changes are occurring and the data owner records the last verified ledger. Public disclosure, user remediation, and unpausing require security and incident-commander approval.

### Escalation chain

1. **On-call engineer** — first responder; triages, assigns an initial severity, and opens the incident channel.
2. **Incident commander** — paged for SEV-1/SEV-2; owns the timeline and decisions.
3. **Security lead** — paged for SEV-1 and any suspected exploit; owns containment and disclosure.
4. **Contract operator / transaction approver** — paged for SEV-1; executes reviewed state changes.
5. **Engineering lead / maintainers** — engaged when a replacement deployment or cross-team coordination is required.
6. **Communications owner** — engaged for any user-facing impact.

If a role does not acknowledge within 15 minutes, escalate to the next role in the chain and record the escalation in the incident timeline.

## Incident Response Workflow

Every incident follows the same five phases. Record UTC timestamps for each transition in the incident timeline.

1. **Detect** — an alert, user report, or operator observation opens the incident. Capture symptoms, contract ID, network, and suspected first affected ledger.
2. **Triage** — assign severity using the table above, page the required roles, and open the private incident channel. Target: severity assigned within 15 minutes of detection.
3. **Mitigate** — contain the impact (pause, restrict deposits, freeze deployments) and preserve evidence. Do not attempt unreviewed fixes against production state.
4. **Resolve** — apply the remediation from the matching runbook, verify on-chain state, and confirm the impact is closed.
5. **Follow-up** — record the final timeline, confirm exit criteria, and hand off any open items to their owners. Post-mortems are tracked separately and are out of scope for this runbook.

## Common First Response

1. Declare the incident in the private incident channel and record UTC time, reporter, contract ID, network, symptoms, and suspected first affected ledger.
2. Freeze unrelated deployments and frontend configuration changes.
3. Preserve evidence: transaction hashes, operation XDR where safe, RPC responses, contract events, logs, frontend error output, current WASM checksum, and the deployed contract ID. Store artifacts read-only with restricted access.
4. Verify the network and contract ID before signing anything. Use read-only queries first.
5. Confirm the current admin, pending admin, paused state, contract initialization state, and contract/token balances from more than one trusted RPC or explorer when possible.
6. Require a second operator to review every recovery transaction’s destination, amount, contract ID, network passphrase, and simulation result.
7. Keep a transaction ledger containing: UTC time, operator, reviewer, function, depositor, deposit ID, token, amount, transaction hash, result, and evidence link.

## Runbooks for Top 5 Incidents

Each runbook below is a condensed procedure. Follow the Common First Response first, then the matching runbook.

### Runbook 1: Active exploit or contract vulnerability

See [Scenario A: Contract Vulnerability or Active Exploit](#scenario-a-contract-vulnerability-or-active-exploit). Pause the contract, preserve evidence, build and deploy a reviewed replacement, and migrate users through supported contract calls.

### Runbook 2: Compromised admin signer

1. Set SEV-1 and page the security lead and incident commander.
2. If a safe signer remains, call `pause(admin)` and verify `is_paused() == true` from the network.
3. Begin a two-step admin transfer to a freshly generated, hardware-backed signer; require a second operator to review the pending-admin transaction.
4. Do not sign recovery transactions from the compromised key. Reconcile balances and follow Runbook 1 for remediation.

### Runbook 3: Data corruption or state mismatch

See [Scenario B: Data Corruption or State Mismatch](#scenario-b-data-corruption-or-state-mismatch). Treat ledger history as authoritative; reconcile indexer, RPC, and application views against on-chain events before changing state.

### Runbook 4: Failed or stuck withdrawals

1. Set SEV-2 unless fund loss is confirmed.
2. Reproduce the failure with a read-only simulation; capture the error, deposit ID, token, and amount.
3. Verify the deposit exists, the contract is not paused, and the token balance covers the withdrawal.
4. If the contract is paused, follow the unpause approval path; otherwise escalate to the security lead for a contract-level defect.

### Runbook 5: RPC, indexer, or frontend outage

1. Set SEV-3 unless on-chain state is affected.
2. Confirm the contract is reachable through an alternate trusted RPC or explorer.
3. Fail over the frontend/RPC configuration and verify read-only queries succeed.
4. If on-chain state appears inconsistent, escalate to Runbook 3.

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

1. Treat an indexer or fronte

/* … truncated 6667 chars — edit only what you need near the top … */
