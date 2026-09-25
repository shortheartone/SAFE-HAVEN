# Requirements Document

## Introduction

The risk scoring system adds a transparent, on-chain risk assessment layer to the SAFE-HAVEN vault contract. Each deposit receives a numeric score (0–100) and a categorical label derived from measurable deposit parameters: lock duration, early-withdrawal penalty, deposited amount, and token identity. Scores are computed deterministically at deposit time and recomputed whenever relevant parameters change. A new query function lets users filter deposits by risk category, enabling informed comparison of risk levels across the vault.

The system operates entirely within the Soroban execution environment; it introduces no off-chain oracle dependencies and does not perform real-time market data lookups.

---

## Glossary

- **RiskScore**: A struct containing a numeric score (`u32`, range 0–100) and a `RiskCategory` variant.
- **RiskCategory**: An enum with four variants — `Low`, `Medium`, `High`, `Critical` — derived from the numeric score.
- **RiskScorer**: The pure scoring subsystem responsible for computing `RiskScore` from deposit parameters.
- **VaultEntry**: The existing struct that stores a time-based deposit; extended with a `risk_score` field of type `RiskScore`.
- **LedgerVaultEntry**: The existing struct that stores a ledger-sequence-based deposit; extended with a `risk_score` field of type `RiskScore`.
- **SafeHaven**: The top-level Soroban smart contract.
- **Depositor**: An account that has created one or more deposits in the vault.
- **penalty_bps**: Early-withdrawal penalty expressed in basis points (0–10 000).
- **lock_duration_secs**: The number of seconds between deposit creation and `unlock_time`.
- **Score Factor**: An individual contribution to the total numeric score, derived from a single deposit parameter.

---

## Requirements

### Requirement 1: RiskScore Data Type

**User Story:** As a developer integrating with SAFE-HAVEN, I want a well-defined `RiskScore` type returned by queries, so that I can display and compare risk levels without additional parsing logic.

#### Acceptance Criteria

1. THE `RiskScore` struct SHALL contain exactly two fields: `score` of type `u32` and `category` of type `RiskCategory`.
2. THE `RiskCategory` enum SHALL define exactly four variants: `Low`, `Medium`, `High`, and `Critical`.
3. THE `RiskScore` struct and `RiskCategory` enum SHALL be annotated with `#[contracttype]` so that Soroban serialises them without custom code.
4. WHEN `score` is in the range 0–30 (inclusive), THE `RiskScore` SHALL carry a `category` of `Low`.
5. WHEN `score` is in the range 31–60 (inclusive), THE `RiskScore` SHALL carry a `category` of `Medium`.
6. WHEN `score` is in the range 61–80 (inclusive), THE `RiskScore` SHALL carry a `category` of `High`.
7. WHEN `score` is in the range 81–100 (inclusive), THE `RiskScore` SHALL carry a `category` of `Critical`.
8. THE `RiskScore` struct SHALL implement `Clone`, `Debug`, `Eq`, and `PartialEq` so that tests and the contract can compare values.

---

### Requirement 2: Risk Score Calculation

**User Story:** As a depositor, I want my deposit's risk score to reflect its actual risk characteristics, so that I can understand the exposure I am accepting before committing funds.

#### Acceptance Criteria

1. THE `RiskScorer` SHALL compute `score` as the sum of individual Score Factors, clamped to a maximum of 100.
2. THE `RiskScorer` SHALL derive a **Duration Factor** from `lock_duration_secs` using the following scale:
   - 0–2 592 000 seconds (≤ 30 days): 0 points
   - 2 592 001–7 776 000 seconds (31–90 days): 10 points
   - 7 776 001–15 552 000 seconds (91–180 days): 20 points
   - 15 552 001–31 536 000 seconds (181–365 days): 30 points
   - > 31 536 000 seconds (> 365 days): 40 points
3. THE `RiskScorer` SHALL derive a **Penalty Factor** from `penalty_bps` using the following scale:
   - 0 bps: 0 points
   - 1–500 bps: 10 points
   - 501–1 000 bps: 20 points
   - 1 001–3 000 bps: 30 points
   - > 3 000 bps: 40 points
4. THE `RiskScorer` SHALL derive an **Amount Factor** from the deposited `amount` relative to `MAX_DEPOSIT_AMOUNT` (1 000 000 000 000 000 base units) using the following scale:
   - ≤ 1% of MAX: 0 points
   - > 1% and ≤ 10% of MAX: 5 points
   - > 10% and ≤ 50% of MAX: 10 points
   - > 50% of MAX: 20 points
5. WHEN `score` would exceed 100 after summing all factors, THE `RiskScorer` SHALL clamp `score` to 100.
6. THE `RiskScorer` SHALL be a pure function: given identical inputs it SHALL produce an identical `RiskScore`.
7. THE `RiskScorer` SHALL accept parameters `(lock_duration_secs: u64, penalty_bps: u32, amount: i128)` and return `RiskScore`.

---

### Requirement 3: VaultEntry and LedgerVaultEntry Integration

**User Story:** As a depositor, I want every deposit I create to automatically carry a risk score, so that I never have to call a separate function to get risk information.

#### Acceptance Criteria

1. THE `VaultEntry` struct SHALL include a `risk_score` field of type `RiskScore`.
2. THE `LedgerVaultEntry` struct SHALL include a `risk_score` field of type `RiskScore`.
3. WHEN the `SafeHaven` contract processes a `deposit` call, THE `SafeHaven` SHALL compute `risk_score` via `RiskScorer` before writing `VaultEntry` to storage.
4. WHEN the `SafeHaven` contract processes a `deposit_for` call, THE `SafeHaven` SHALL compute `risk_score` via `RiskScorer` before writing `VaultEntry` to storage.
5. WHEN the `SafeHaven` contract processes a `deposit_by_ledger` call, THE `SafeHaven` SHALL compute `risk_score` via `RiskScorer` and use `lock_duration_secs` estimated as `(unlock_ledger - current_ledger) * 5` seconds before writing `LedgerVaultEntry` to storage.
6. WHEN a `cancel_deposit` call removes a deposit, THE `SafeHaven` SHALL remove the associated `risk_score` as part of removing the full entry (no separate cleanup required).

---

### Requirement 4: Risk Score Visibility in Queries

**User Story:** As a frontend developer, I want deposit queries to return the risk score alongside the deposit data, so that I can render risk information without a second contract call.

#### Acceptance Criteria

1. WHEN `get_vault` is called with a valid `depositor` and `deposit_id`, THE `SafeHaven` SHALL return a `VaultEntry` that includes the `risk_score` field.
2. WHEN `get_vault_batch` is called, THE `SafeHaven` SHALL return `VaultEntry` values each including their `risk_score` field.
3. THE `SafeHaven` SHALL expose a `get_risk_score` read-only function that accepts `(depositor: Address, deposit_id: u32)` and returns `Option<RiskScore>`.
4. WHEN `get_risk_score` is called for a deposit that does not exist, THE `SafeHaven` SHALL return `None`.
5. WHEN `get_risk_score` is called for a valid deposit, THE `SafeHaven` SHALL return `Some(risk_score)` without bumping the storage TTL.

---

### Requirement 5: Filtering Deposits by Risk Category

**User Story:** As a depositor, I want to query all my deposits filtered by risk category, so that I can quickly identify and act on deposits in a given risk tier.

#### Acceptance Criteria

1. THE `SafeHaven` SHALL expose a `get_deposits_by_risk` read-only function that accepts `(depositor: Address, category: RiskCategory)` and returns `Vec<u32>` (a list of matching deposit IDs).
2. WHEN `get_deposits_by_risk` is called, THE `SafeHaven` SHALL iterate over all deposit IDs for the given `depositor` and include only those whose `risk_score.category` matches the requested `category`.
3. WHEN no deposits match the requested `category`, THE `SafeHaven` SHALL return an empty `Vec`.
4. WHEN a `depositor` has no deposits at all, THE `SafeHaven` SHALL return an empty `Vec`.
5. THE `get_deposits_by_risk` function SHALL check both timestamp-based (`VaultEntry`) and ledger-based (`LedgerVaultEntry`) deposits, and SHALL include matching IDs from both types in the result.

---

### Requirement 6: Score Recalculation on Parameter Change

**User Story:** As a depositor, I want the risk score to reflect the current deposit parameters at all times, so that any future admin-level parameter update does not leave stale scores displayed to users.

#### Acceptance Criteria

1. THE `SafeHaven` SHALL expose an `update_risk_score` function that accepts `(depositor: Address, deposit_id: u32)` and recalculates the `risk_score` for that deposit.
2. WHEN `update_risk_score` is called for a timestamp-based deposit, THE `SafeHaven` SHALL recompute `lock_duration_secs` as `entry.unlock_time.saturating_sub(current_timestamp)` and pass it to `RiskScorer`.
3. WHEN `update_risk_score` is called for a ledger-based deposit, THE `SafeHaven` SHALL recompute `lock_duration_secs` as `(entry.unlock_ledger.saturating_sub(current_ledger)) * 5` and pass it to `RiskScorer`.
4. WHEN `update_risk_score` is called for a deposit that does not exist, THE `SafeHaven` SHALL return `Err(VaultError::NoDepositFound)`.
5. WHEN `update_risk_score` successfully recalculates, THE `SafeHaven` SHALL persist the updated `VaultEntry` (or `LedgerVaultEntry`) with the new `risk_score` and return `Ok(RiskScore)` to the caller.
6. THE `update_risk_score` function SHALL NOT require authentication — any caller MAY trigger a score refresh for any deposit.

---

### Requirement 7: Risk Methodology Documentation

**User Story:** As an auditor or integrating developer, I want the scoring methodology to be documented in the source code and spec, so that I can verify that scores are calculated correctly and consistently.

#### Acceptance Criteria

1. THE scoring module SHALL contain a doc-comment block that describes each Score Factor, its input parameter, and its point scale.
2. THE doc-comment SHALL specify the category boundary thresholds (0–30 Low, 31–60 Medium, 61–80 High, 81–100 Critical).
3. THE doc-comment SHALL state that the scorer is a pure function with no external dependencies or randomness.
4. THE score range SHALL be documented as 0 (lowest risk) to 100 (highest risk) inclusive.

---

### Requirement 8: Test Coverage for Score Calculations

**User Story:** As a contract maintainer, I want automated tests to verify score calculations, so that changes to scoring logic are caught before deployment.

#### Acceptance Criteria

1. THE test suite SHALL include a unit test that verifies each Duration Factor boundary produces the correct point contribution.
2. THE test suite SHALL include a unit test that verifies each Penalty Factor boundary produces the correct point contribution.
3. THE test suite SHALL include a unit test that verifies each Amount Factor boundary produces the correct point contribution.
4. THE test suite SHALL include a unit test that verifies clamping: a deposit whose factor sum exceeds 100 SHALL receive a `score` of exactly 100.
5. THE test suite SHALL include a unit test that verifies `get_deposits_by_risk` returns only deposit IDs whose `category` matches the requested value.
6. THE test suite SHALL include a unit test that verifies `update_risk_score` persists a new score when called on an existing deposit.
7. FOR ALL valid combinations of `(lock_duration_secs, penalty_bps, amount)`, THE `RiskScorer` SHALL produce a `score` in the range 0–100 inclusive (property-based test).
8. FOR ALL valid `RiskScore` values, THE `category` SHALL be consistent with the `score` range thresholds (property-based test — round-trip invariant between score and category).
