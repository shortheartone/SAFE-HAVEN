# SAFE-HAVEN Contract Security Audit Checklist

Use this checklist for each contract release. Record evidence, not only pass/fail results. Any unchecked item is a release blocker unless the risk is explicitly accepted by the project owner.

## 1. Audit Scope and Inventory

- [ ] Record the commit, WASM hash, Soroban SDK version, Rust toolchain, network, and deployment address.
- [ ] Confirm the reviewed source matches the WASM artifact and deployment configuration.
- [ ] Review all public entry points:
  - [ ] `initialize`
  - [ ] `deposit`, `deposit_for`, `deposit_by_ledger`
  - [ ] `withdraw`, `withdraw_to`, `cancel_deposit`, `emergency_withdraw`
  - [ ] `pause`, `unpause`
  - [ ] `transfer_admin`, `accept_admin`, `cancel_transfer_admin`, `renounce_admin`
  - [ ] `migrate`
- [ ] Review all read-only entry points for information leaks, unexpected writes, and bounded resource use.
- [ ] Review each read-only entry point individually: `is_paused`, `get_vault`, `get_ledger_vault`, `get_deposit_type`, `get_vault_batch`, `get_deposit_batch`, `get_deposit_ids`, `get_time`, `time_remaining`, `get_admin`, `get_pending_admin`, `get_constants`, `get_fee_recipient`, `get_depositor_count`, `get_depositors`, `is_initialized`, `version`, `get_deposits_page`, and `get_storage_version`.
- [ ] Confirm the token interface and supported token behavior, including authorization, decimals, clawbacks, pauses, and non-standard implementations.
- [ ] Review `VaultError` mappings and confirm failures do not leave partial state or stranded funds.

## 2. Authentication and Authorization

- [ ] Every user-controlled action requires the correct address authorization.
- [ ] `deposit_for` authorizes the payer, while the beneficiary controls later withdrawal or cancellation as intended.
- [ ] Admin-only actions reject non-admin callers, including after an admin transfer or renunciation.
- [ ] Initialization requires authorization from the proposed admin and cannot be replayed.
- [ ] Two-step admin transfer requires acceptance by the exact pending address.
- [ ] Pending admin cancellation clears only the pending transfer and preserves the current admin.
- [ ] Renouncing admin is deliberate, irreversible, and leaves no privileged path through migration, pause, emergency withdrawal, or configuration.
- [ ] `withdraw_to` cannot be used to redirect another depositor’s funds.
- [ ] No authorization check relies on caller-supplied data that can be substituted or replayed.

## 3. Deposit and Token Movement

- [ ] Reject zero, negative, and over-limit amounts before token transfer.
- [ ] Validate penalty basis points in the inclusive range `0..=10_000`.
- [ ] Reject nonzero penalties when no fee recipient is configured.
- [ ] Validate timestamp unlocks are in the future, meet the minimum duration, and do not exceed the configured maximum.
- [ ] Validate ledger unlocks are in the future and meet `MIN_LOCK_LEDGERS`.
- [ ] Confirm the amount transferred equals the amount recorded, and the recorded token equals the token transferred.
- [ ] Confirm failed token transfers revert the whole invocation without creating a deposit or active-ID entry.
- [ ] Confirm deposits cannot overwrite an existing `(depositor, deposit_id)` entry.
- [ ] Test token contract addresses that are malformed, unauthorized, or otherwise unable to transfer.
- [ ] Verify no callback or token behavior can re-enter a state transition or bypass checks.
- [ ] Confirm all arithmetic is overflow-safe, especially amount limits, ID increments, time differences, penalties, and fee calculations.

## 4. Withdrawal, Cancellation, and Recovery

- [ ] `withdraw` permits release at the exact unlock boundary and rejects one second/ledger before it.
- [ ] `withdraw_to` applies identical lock and ownership rules to `withdraw`.
- [ ] Successful withdrawal removes the entry and active ID before or atomically with releasing funds.
- [ ] Withdrawal transfers exactly the stored principal to the intended recipient.
- [ ] Missing deposits, wrong deposit types, already-removed deposits, and repeated withdrawals fail safely.
- [ ] Cancellation is blocked after unlock and while the deposit is otherwise eligible for normal withdrawal.
- [ ] Cancellation returns the correct amount after the configured penalty and sends the penalty only to the configured recipient.
- [ ] Zero, partial, and full penalty cases preserve accounting invariants.
- [ ] Emergency withdrawal is admin-only, cannot release arbitrary or nonexistent funds, and emits an auditable event.
- [ ] Emergency withdrawal behavior for timestamp and ledger deposits is consistent.
- [ ] Batch emergency withdrawal enforces `MAX_BATCH_SIZE`, handles missing entries safely, and cannot mix beneficiaries or tokens incorrectly.

## 5. State, Storage, and Lifecycle Invariants

- [ ] For every active deposit, exactly one matching timestamp or ledger entry exists.
- [ ] Every stored entry appears exactly once in `ActiveDepositIds`; removed entries do not remain active.
- [ ] Deposit IDs are monotonic per depositor and cannot collide after withdrawal, cancellation, or migration.
- [ ] Timestamp and ledger deposit types cannot be confused by lookup, withdrawal, cancellation, or pagination.
- [ ] Depositor count/list flags remain correct through first deposit, multiple deposits, full withdrawal, and re-deposit.
- [ ] Persistent storage TTL is extended for every long-lived record and covers the maximum permitted lock period.
- [ ] Reads that are documented as read-only do not mutate state or unexpectedly extend TTL.
- [ ] Pagination offsets and limits are bounded and cannot exhaust transaction resources.
- [ ] Batch query inputs and outputs are length-bounded and preserve input order.
- [ ] Storage keys remain backward-compatible with the documented layout and do not collide across variants.
- [ ] Migration is idempotent, admin-authorized, version-gated, and cannot delete or reinterpret user funds.
- [ ] Storage version is updated only after all migration steps succeed.

## 6. Configuration, Pause, and Governance

- [ ] Initialization rejects invalid custom maximum deposit and lock values.
- [ ] Default limits apply when optional configuration is absent.
- [ ] Configuration cannot be changed after initialization unless an explicitly reviewed governance path exists.
- [ ] Pause blocks every fund-creating or otherwise risky user action intended by the specification.
- [ ] Pause does not unexpectedly block withdrawals or emergency recovery unless documented and reviewed.
- [ ] Only the current admin can pause/unpause, and the paused state is observable.
- [ ] Admin, fee recipient, limits, pending admin, initialized state, and pause state have expected TTL and absence behavior.

## 7. Events and Observability

- [ ] Every deposit, withdrawal, cancellation, penalty, emergency action, pause change, admin change, initialization, and migration emits the expected event.
- [ ] Events identify the depositor, payer/recipient where relevant, token, amount, deposit ID, and unlock value without ambiguity.
- [ ] Event values match committed state and token transfers.
- [ ] Failed transactions do not produce misleading success events.
- [ ] Event topics and payloads are documented and stable for indexers.

## 8. Adversarial and Property Tests

- [ ] Run the complete Rust test suite and add regression tests for every finding.
- [ ] Fuzz amounts, penalties, timestamps, ledger sequences, IDs, pagination, batch sizes, and addresses.
- [ ] Test boundary values: `0`, `1`, maximums, maximum plus one, exact unlock, unlock minus one, and integer limits.
- [ ] Test repeated calls and reordered calls: initialize, withdraw, cancel, pause, admin accept/cancel, renounce, and migrate.
- [ ] Test multiple users, multiple tokens, multiple deposits, mixed timestamp/ledger deposits, and shared payers.
- [ ] Assert conservation: contract token balance equals the sum of active principals plus any explicitly retained fee balance.
- [ ] Assert no unauthorized actor can alter another user’s deposits, admin state, or recovery destination.
- [ ] Run static analysis, formatting, dependency/vulnerability checks, and WASM size checks.
- [ ] Run a local-node smoke test against the exact deployment artifact.

## 9. Deployment and Operational Sign-Off

- [ ] Verify deployment and initialization transactions on the intended network.
- [ ] Confirm the admin and fee-recipient addresses through an independent channel.
- [ ] Confirm contract ID, WASM hash, limits, pause state, and storage version after deployment.
- [ ] Test a small deposit, read, withdrawal, cancellation, and event indexing flow on the target network.
- [ ] Document emergency recovery authority, key custody, and admin-transfer procedures.
- [ ] Document supported tokens and known token-specific assumptions.
- [ ] Publish the audit scope, findings, accepted risks, remediation status, and residual risk.
- [ ] Obtain approvals from the contract owner, reviewer/auditor, and release operator.

## Findings Log

| ID | Severity | Location | Finding | Recommendation | Owner | Status |
|---|---|---|---|---|---|---|
| | | | | | | |

## Release Sign-Off

- Commit/WASM hash: ______________________________
- Network and contract ID: _______________________
- Auditor: ______________________________________
- Contract owner: _______________________________
- Release operator: _____________________________
- Open findings accepted by: _____________________
- Date: _________________________________________
- Final decision: [ ] Approved  [ ] Blocked
