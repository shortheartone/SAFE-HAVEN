# Requirements Document

## Introduction

The Withdrawal Scheduling feature enables users of the SAFE-HAVEN CosmWasm smart contract to configure automatic withdrawals from their unlocked deposits on specific future timestamps or at recurring intervals. This supports regular income streams from savings vaults without requiring manual withdrawal transactions for each payout.

Schedules are fixed-amount, either one-time or recurring, and are tied to a single deposit. Users may cancel a schedule at any time before its next execution. The contract stores schedule state in `WithdrawalSchedule` entries and emits events at each lifecycle transition.

The feature is implemented in Rust using the Soroban SDK (Stellar blockchain) within the existing `safe-haven` contract, consistent with the project's storage, error, and event patterns.

---

## Glossary

- **Scheduler**: The SAFE-HAVEN smart contract acting as the execution engine for withdrawal schedules.
- **Depositor**: The account that owns a vault deposit and creates or cancels withdrawal schedules against it.
- **WithdrawalSchedule**: A persistent storage entry recording the configuration and state of a single scheduled withdrawal series for one deposit.
- **schedule_id**: A monotonically increasing, per-depositor identifier that uniquely addresses a `WithdrawalSchedule`.
- **deposit_id**: The identifier of the `VaultEntry` or compatible deposit entry from which funds are drawn.
- **execution_time**: The Unix timestamp (seconds) at which the next scheduled withdrawal is due for execution.
- **interval_secs**: The recurring period in seconds between executions. A value of `0` denotes a one-time schedule.
- **ScheduleStatus**: An enum with variants `Active`, `Cancelled`, and `Completed`.
- **WithdrawalScheduled event**: The on-chain event emitted when a new `WithdrawalSchedule` is created.
- **ScheduledWithdrawalExecuted event**: The on-chain event emitted when a scheduled withdrawal is successfully executed.
- **ScheduleCancelled event**: The on-chain event emitted when a `WithdrawalSchedule` is cancelled by the depositor.

---

## Requirements

### Requirement 1: Create a Withdrawal Schedule

**User Story:** As a depositor, I want to schedule a future automatic withdrawal from one of my unlocked deposits, so that I can plan regular income streams without submitting manual transactions.

#### Acceptance Criteria

1. WHEN a depositor calls `schedule_withdrawal` with a valid `deposit_id`, positive `amount`, a future `execution_time`, and a non-negative `interval_secs`, THE Scheduler SHALL create a `WithdrawalSchedule` entry in persistent storage and return the new `schedule_id`.
2. WHEN `schedule_withdrawal` is called, THE Scheduler SHALL assign a monotonically increasing `schedule_id` that is unique per depositor and never reused.
3. WHEN `schedule_withdrawal` is called and the contract is paused, THE Scheduler SHALL return `ContractPaused` and SHALL NOT create a schedule entry.
4. IF the supplied `amount` is less than or equal to zero, THEN THE Scheduler SHALL return `InvalidAmount` and SHALL NOT create a schedule entry.
5. IF the supplied `execution_time` is less than or equal to the current ledger timestamp, THEN THE Scheduler SHALL return `ScheduleTimeNotInFuture` and SHALL NOT create a schedule entry.
6. IF the deposit identified by `deposit_id` does not exist for the calling depositor, THEN THE Scheduler SHALL return `NoDepositFound` and SHALL NOT create a schedule entry.
7. IF the deposit identified by `deposit_id` is still within its lock period at the time of schedule creation, THEN THE Scheduler SHALL return `FundsStillLocked` and SHALL NOT create a schedule entry.
8. IF `interval_secs` is greater than zero and less than 60, THEN THE Scheduler SHALL return `InvalidScheduleInterval` and SHALL NOT create a schedule entry.
9. WHEN a `WithdrawalSchedule` is successfully created, THE Scheduler SHALL emit a `WithdrawalScheduled` event containing the `depositor`, `deposit_id`, `schedule_id`, `amount`, `execution_time`, and `interval_secs`.
10. WHEN a `WithdrawalSchedule` is created, THE Scheduler SHALL set its initial `ScheduleStatus` to `Active`.

---

### Requirement 2: Execute a Scheduled Withdrawal

**User Story:** As a depositor, I want my scheduled withdrawal to execute automatically when the execution time is reached, so that I receive funds at the configured time without manual intervention.

#### Acceptance Criteria

1. WHEN `execute_scheduled_withdrawal` is called for a schedule whose `execution_time` is less than or equal to the current ledger timestamp and whose `ScheduleStatus` is `Active`, THE Scheduler SHALL transfer `amount` tokens from the contract to the depositor's address.
2. WHEN `execute_scheduled_withdrawal` is called and the deposit balance is less than `amount`, THE Scheduler SHALL return `InsufficientDepositBalance` and SHALL NOT execute the transfer.
3. WHEN a one-time schedule (where `interval_secs` equals zero) is successfully executed, THE Scheduler SHALL set the `ScheduleStatus` to `Completed` and SHALL NOT reschedule.
4. WHEN a recurring schedule (where `interval_secs` is greater than zero) is successfully executed, THE Scheduler SHALL update `execution_time` by adding `interval_secs` to the previous `execution_time` and SHALL retain `ScheduleStatus` as `Active`.
5. IF `execute_scheduled_withdrawal` is called for a schedule whose `execution_time` is greater than the current ledger timestamp, THEN THE Scheduler SHALL return `ScheduleNotYetDue` and SHALL NOT execute the transfer.
6. IF `execute_scheduled_withdrawal` is called for a schedule whose `ScheduleStatus` is not `Active`, THEN THE Scheduler SHALL return `ScheduleNotActive` and SHALL NOT execute the transfer.
7. IF the deposit identified by `deposit_id` no longer exists at execution time, THEN THE Scheduler SHALL set the schedule `ScheduleStatus` to `Completed` and SHALL return `NoDepositFound`.
8. WHEN a scheduled withdrawal is successfully executed, THE Scheduler SHALL deduct `amount` from the deposit's stored balance.
9. WHEN a scheduled withdrawal is successfully executed, THE Scheduler SHALL emit a `ScheduledWithdrawalExecuted` event containing the `depositor`, `deposit_id`, `schedule_id`, `amount`, and the timestamp of execution.

---

### Requirement 3: Cancel a Withdrawal Schedule

**User Story:** As a depositor, I want to cancel a scheduled withdrawal before it executes, so that I can change my withdrawal plans without losing control of my funds.

#### Acceptance Criteria

1. WHEN a depositor calls `cancel_withdrawal_schedule` with a valid `schedule_id` and the corresponding `ScheduleStatus` is `Active`, THE Scheduler SHALL set the `ScheduleStatus` to `Cancelled` and SHALL NOT execute any further transfers for that schedule.
2. IF `cancel_withdrawal_schedule` is called by an address that is not the depositor who created the schedule, THEN THE Scheduler SHALL return `Unauthorized` and SHALL NOT modify the schedule.
3. IF `cancel_withdrawal_schedule` is called for a schedule whose `ScheduleStatus` is not `Active`, THEN THE Scheduler SHALL return `ScheduleNotActive` and SHALL NOT modify the schedule.
4. IF `cancel_withdrawal_schedule` is called with a `schedule_id` that does not exist for the calling depositor, THEN THE Scheduler SHALL return `NoScheduleFound` and SHALL NOT modify any state.
5. WHEN a `WithdrawalSchedule` is successfully cancelled, THE Scheduler SHALL emit a `ScheduleCancelled` event containing the `depositor`, `deposit_id`, `schedule_id`, and the cancellation timestamp.

---

### Requirement 4: Query Withdrawal Schedules

**User Story:** As a depositor, I want to query my withdrawal schedules by ID or by deposit, so that I can audit and manage my scheduled cash flows.

#### Acceptance Criteria

1. WHEN `get_withdrawal_schedule` is called with a valid `depositor` and `schedule_id`, THE Scheduler SHALL return the `WithdrawalSchedule` entry including `deposit_id`, `amount`, `execution_time`, `interval_secs`, and `ScheduleStatus`.
2. IF `get_withdrawal_schedule` is called for a `schedule_id` that does not exist for the given `depositor`, THEN THE Scheduler SHALL return `None` (no error).
3. WHEN `get_schedules_for_deposit` is called with a valid `depositor` and `deposit_id`, THE Scheduler SHALL return all `WithdrawalSchedule` entries associated with that deposit, regardless of `ScheduleStatus`.

---

### Requirement 5: Schedule Storage Lifecycle

**User Story:** As a contract operator, I want schedule storage to follow the same persistent TTL bump patterns as other vault storage, so that schedules are not silently evicted by storage expiry.

#### Acceptance Criteria

1. WHEN a `WithdrawalSchedule` is created or updated, THE Scheduler SHALL extend the storage TTL of the schedule entry using `BUMP_THRESHOLD` and `BUMP_TARGET` consistent with the existing vault storage patterns.
2. WHEN a `WithdrawalSchedule` is read during `execute_scheduled_withdrawal` or `cancel_withdrawal_schedule`, THE Scheduler SHALL extend the storage TTL of the schedule entry.
3. THE Scheduler SHALL assign unique `VaultKey` variants for `WithdrawalSchedule` storage that do not collide with any existing `VaultKey` variants.

---

### Requirement 6: One-Time and Recurring Schedule Support

**User Story:** As a depositor, I want to choose between a single scheduled withdrawal or a repeating one, so that I can cover both one-off future payouts and ongoing income needs.

#### Acceptance Criteria

1. WHEN `interval_secs` is zero, THE Scheduler SHALL treat the schedule as one-time and SHALL set `ScheduleStatus` to `Completed` after a single successful execution.
2. WHEN `interval_secs` is greater than or equal to 60, THE Scheduler SHALL treat the schedule as recurring and SHALL advance `execution_time` by `interval_secs` after each successful execution without changing `ScheduleStatus`.
3. THE Scheduler SHALL support multiple concurrent `Active` schedules against the same `deposit_id`, each with its own independent `execution_time`, `interval_secs`, and `amount`.
4. WHEN a deposit is fully withdrawn through the normal `withdraw` path, any remaining `Active` schedules against that deposit SHALL return `NoDepositFound` upon the next execution attempt, which SHALL transition them to `Completed`.

---

### Requirement 7: Event Emission for Schedule Lifecycle

**User Story:** As an off-chain indexer or client, I want events emitted at each schedule lifecycle transition, so that I can track scheduled withdrawal activity without polling contract state.

#### Acceptance Criteria

1. THE Scheduler SHALL emit a `WithdrawalScheduled` event with `depositor`, `deposit_id`, `schedule_id`, `amount`, `execution_time`, and `interval_secs` when a schedule is created.
2. THE Scheduler SHALL emit a `ScheduledWithdrawalExecuted` event with `depositor`, `deposit_id`, `schedule_id`, `amount`, and `executed_at` when a scheduled withdrawal is executed.
3. THE Scheduler SHALL emit a `ScheduleCancelled` event with `depositor`, `deposit_id`, `schedule_id`, and `cancelled_at` when a schedule is cancelled.
4. THE Scheduler SHALL NOT emit duplicate events for the same lifecycle transition within a single transaction.

---

### Requirement 8: Access Control for Schedule Operations

**User Story:** As a contract operator, I want schedule creation and cancellation to respect the existing ACL and pause mechanisms, so that withdrawal scheduling does not bypass vault security controls.

#### Acceptance Criteria

1. WHEN `schedule_withdrawal` is called, THE Scheduler SHALL verify the depositor has the `Withdraw` permission using the existing ACL check and SHALL return `Unauthorized` if the permission is absent.
2. WHEN the contract is in emergency lockdown, THE Scheduler SHALL return `EmergencyLockdown` from `schedule_withdrawal` and SHALL NOT create a schedule entry.
3. WHEN the contract is paused, THE Scheduler SHALL return `ContractPaused` from `execute_scheduled_withdrawal` and SHALL NOT transfer funds.
