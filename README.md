# SAFE-HAVEN

[![Rust](https://img.shields.io/badge/Rust-1.81%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![Soroban SDK](https://img.shields.io/badge/Soroban-SDK%20v22-blue?logo=stellar)](https://github.com/stellar/rs-soroban-sdk)
[![License](https://img.shields.io/badge/License-MIT-green)](./LICENSE)
[![Tests](https://github.com/kenedybok3/SAFE-HAVEN/actions/workflows/ci.yml/badge.svg)](https://github.com/kenedybok3/SAFE-HAVEN/actions)

A production-ready decentralized vault on the Stellar blockchain (Soroban) — with a full React/TypeScript frontend.

Tokens are locked in the smart contract until a future timestamp. Early exits are possible with a configurable penalty. Admin rights can be permanently renounced for fully trustless operation.

---

## Project Structure

```
SAFE-HAVEN/
├── contracts/safe-haven/       Smart contract (Rust / Soroban)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs              Crate root
│       ├── contract.rs         All public entry points
│       ├── types.rs            VaultKey, VaultEntry, constants
│       ├── errors.rs           VaultError enum (14 codes)
│       ├── events.rs           Event emission helpers
│       ├── storage.rs          Persistent storage + TTL helpers
│       └── test.rs             48+ unit tests
│
├── frontend/                   React + TypeScript + Vite (UI)
│   ├── src/
│   │   ├── App.tsx
│   │   ├── config.ts           Contract ID, RPC URLs
│   │   ├── context/            Freighter wallet
│   │   ├── hooks/              useDeposits, useContractInfo
│   │   ├── lib/                Stellar SDK helpers, formatting
│   │   ├── components/         Header, TabNav, DepositCard, etc.
│   │   └── pages/              Dashboard, Deposit, Withdraw, Admin
│   ├── .env.example
│   └── README.md
│
├── Cargo.toml                  Rust workspace
├── Makefile                    Build / test / lint / deploy
└── STRUCTURE.md                Detailed project layout
```

---

## Quick Start

### Smart Contract

```bash
# Prerequisites
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown
cargo install --locked soroban-cli

# Build
make build

# Test
make test

# Deploy to testnet
export SOROBAN_SECRET_KEY=S...
make deploy-testnet
```

### Frontend

```bash
cd frontend
npm install
cp .env.example .env   # set VITE_CONTRACT_ID to your deployed contract
npm run dev            # -> http://localhost:5173
```

See [`frontend/README.md`](./frontend/README.md) for the full frontend guide.

---

## Overview

| Property | Value |
|---|---|
| Network | Stellar (Soroban) |
| Language | Rust |
| SDK | soroban-sdk v22 |
| Storage | Persistent (per-depositor) |
| Max deposit | 10^15 units |
| Max lock duration | 5 years |
| Min lock duration | 60 seconds |
| Early-exit penalty | 0-100% (basis points, set at deposit time) |

---

## How It Works

1. **Deposit** - User calls `deposit(token, amount, unlock_time, penalty_bps)` — tokens transfer into the contract
2. **Storage** - Contract stores a `VaultEntry` in persistent storage keyed by `(depositor, deposit_id)`
3. **Verification** - On `withdraw()`, contract checks `ledger.timestamp() >= unlock_time`
4. **Unlock** - Tokens returned to depositor. Otherwise call fails with `FundsStillLocked`
5. **Early exit** - `cancel_deposit()` returns funds minus penalty; penalty goes to `fee_recipient`
6. **Renewal** - The depositor can call `renew_deposit()` after unlock to set a new unlock time and penalty without moving or changing the amount
7. **Admin recovery** - Admin can emergency-withdraw any deposit (funds always go to depositor, never admin)
8. **Trustless mode** - Admin can be permanently renounced via `renounce_admin()`

---

## Contract API

### Initialization

#### `initialize(admin, fee_recipient, max_deposit?, max_lock_secs?)`
One-time setup. Sets admin and fee recipient. Optionally overrides compile-time limits.

---

### Core Functions

#### `deposit(depositor, token, amount, unlock_time, penalty_bps) -> u32`
Locks tokens. Returns the deposit ID.

#### `deposit_for(payer, depositor, token, amount, unlock_time, penalty_bps) -> u32`
Payer funds a vault for a different beneficiary.

#### `deposit_by_ledger(depositor, token, amount, unlock_ledger, penalty_bps) -> u32`
Locks tokens until a specific Stellar ledger sequence number is reached, instead of a wall-clock timestamp.

Use this when you need to express a lock period in terms of on-chain ledger progression — for example, "release after the network has produced exactly N more ledgers" — rather than relying on the ledger's timestamp field.

**Parameters**

| Parameter | Type | Description |
|---|---|---|
| `depositor` | `Address` | Account locking the tokens. Must sign the transaction. |
| `token` | `Address` | SAC-compatible token contract address. |
| `amount` | `i128` | Amount to lock (> 0, ≤ `max_deposit`). |
| `unlock_ledger` | `u32` | Ledger sequence number at or after which withdrawal is permitted. Must be > `current_ledger + 12` (minimum gap of 12 ledgers ≈ 60 seconds at 5 s/ledger). |
| `penalty_bps` | `u32` | Early-exit penalty in basis points (0–10000). Requires a `fee_recipient` to be configured if > 0. |

**Returns** the deposit ID (`u32`), shared with the same per-depositor counter as timestamp-based deposits.

**Withdrawal** — `withdraw()` and `withdraw_to()` both accept ledger-based deposits. On withdrawal the contract checks `env.ledger().sequence() >= unlock_ledger`; if the ledger sequence has not yet reached the target the call fails with `FundsStillLocked`.

**Estimating wall-clock time from a ledger number** — Stellar produces a new ledger roughly every 5 seconds. To approximate the unlock time in seconds from the current ledger:

```
estimated_seconds = (unlock_ledger - current_ledger) × 5
```

This is an estimate only. Actual ledger close times vary and cannot be predicted exactly. `time_remaining(depositor, id)` performs this same calculation internally.

> **Known Limitations**
>
> - **No frontend support.** The React UI exposes `deposit` and `deposit_for` only. `deposit_by_ledger` must be called directly via the Stellar CLI or an SDK.
> - **Minimum lock is 12 ledgers (≈ 60 s), not the full timestamp minimum validation path.** The check exists (`MIN_LOCK_LEDGERS = 12`) but uses ledger count rather than seconds, so the two minimums are equivalent in practice but not enforced by the same code.
> - **`time_remaining` returns an estimate.** For ledger-based deposits the return value is `remaining_ledgers × 5` seconds — an approximation, not a guaranteed timestamp.
> - **`get_vault` returns `None` for ledger-based deposits.** Use `get_ledger_vault(depositor, id)` instead to retrieve a `LedgerVaultEntry`.

#### `withdraw(depositor, deposit_id)`
Withdraws if `now >= unlock_time`.

#### `withdraw_to(depositor, deposit_id, recipient)`
Withdraws to a different address.

#### `cancel_deposit(depositor, deposit_id)`
Early exit with penalty. Penalty goes to fee_recipient; remainder returned to depositor.

#### `renew_deposit(depositor, deposit_id, new_unlock_time, penalty_bps)`
Renews an existing timestamp-based deposit after its current unlock time has been reached. The depositor must authorize the call. The new unlock time must be in the future and satisfy the configured minimum and maximum lock duration; the penalty must be 0–10000 basis points and requires a configured fee recipient when non-zero. The amount and deposit ID remain unchanged. A `deposit_renewed` event includes the depositor, token, deposit ID, unchanged amount, old and new unlock times, and updated penalty. The deposit becomes withdrawable again when the new unlock time is reached. Ledger-based deposits are not renewable.

---

### Admin Functions

#### `emergency_withdraw(admin, depositor, deposit_id)`
Admin-only. Returns funds to depositor regardless of lock time.

#### `pause(admin)` / `unpause(admin)`
Halts / restores deposits.

#### `transfer_admin(admin, new_admin)` / `accept_admin(new_admin)`
Two-step admin transfer.

#### `cancel_transfer_admin(admin)`
Cancels a pending transfer.

#### `renounce_admin(admin)`
Permanently removes admin. Contract becomes fully trustless.

---

### Read-only Queries

| Function | Returns |
|---|---|
| `get_vault(depositor, id)` | `Option<VaultEntry>` |
| `get_deposit_ids(depositor)` | `Vec<u32>` |
| `time_remaining(depositor, id)` | `u64` seconds |
| `get_time()` | Current ledger timestamp |
| `get_admin()` | `Option<Address>` |
| `get_pending_admin()` | `Option<Address>` |
| `get_fee_recipient()` | `Option<Address>` |
| `get_constants()` | `(max_deposit, max_lock_secs)` |
| `get_depositor_count()` | `u32` |
| `get_depositors(offset, limit)` | `Vec<Address>` |
| `is_paused()` | `bool` |
| `is_initialized()` | `bool` |

---

## Error Codes

| Code | Name | Meaning |
|---|---|---|
| 1 | `InvalidAmount` | Amount <= 0 |
| 2 | `UnlockTimeNotInFuture` | unlock_time (or unlock_ledger) <= current value |
| 3 | `NoDepositFound` | No active deposit at the given (depositor, deposit_id) |
| 4 | `FundsStillLocked` | Lock not yet expired |
| 5 | `DepositAlreadyExists` | Reserved — defined in the enum to hold the slot and prevent future code-number collisions, but never emitted by any current code path |
| 6 | `LockDurationTooLong` | Exceeds 5 years |
| 7 | `Unauthorized` | Caller is not the admin (or contract is not initialized) |
| 8 | `AmountTooLarge` | Exceeds 10^15 |
| 9 | `InvalidPenaltyBps` | penalty_bps > 10000 |
| 10 | `InvalidAdmin` | Proposed new admin is same as current admin |
| 11 | `LockDurationTooShort` | Lock duration < 60 seconds (or < 12 ledgers for `deposit_by_ledger`) |
| 12 | `ContractPaused` | Deposits are paused |
| 13 | `VaultAlreadyUnlocked` | `cancel_deposit` called after the lock has already expired |
| 14 | `MissingFeeRecipient` | penalty_bps > 0 but no fee_recipient is configured |

---

## Security Properties

| Property | Implementation |
|---|---|
| Checks-Effects-Interactions | Storage cleared before token transfer on every withdrawal |
| Auth-first | `require_auth()` is the first call in every mutating function |
| No re-entrancy | State removed before any external token call |
| Bounded inputs | Amount capped at 10^15; lock 60s-5yr |
| No admin theft | Emergency withdraw always sends to depositor |
| Trustless mode | `renounce_admin()` permanently removes admin |
| Safe transfer | Two-step admin transfer prevents key loss |

---

## Known Limitations

The following gaps apply specifically to `deposit_by_ledger` deposits. All other deposit types (`deposit`, `deposit_for`) are unaffected.

| Limitation | Detail |
|---|---|
| **No frontend support** | The React UI only exposes `deposit` and `deposit_for`. Ledger-based deposits must be made via the Stellar CLI or a custom SDK integration. |
| **No maximum lock duration** | `deposit` and `deposit_for` reject lock durations longer than `max_lock_secs` (default 5 years). `deposit_by_ledger` only enforces a *minimum* gap of 12 ledgers (`MIN_LOCK_LEDGERS`). There is no equivalent upper-bound check on `unlock_ledger`, so arbitrarily far-future ledger numbers are accepted. |
| **`get_vault` returns `None`** | The `get_vault(depositor, id)` query only searches timestamp-based entries. To retrieve a ledger-based deposit, use `get_ledger_vault(depositor, id)` which returns `Option<LedgerVaultEntry>`. |
| **`time_remaining` is an estimate** | For ledger-based deposits, `time_remaining` returns `remaining_ledgers × 5` seconds. This is an approximation because actual ledger close times are not exactly 5 seconds. Do not rely on this value for precise scheduling. |
| **`get_deposits_page` excludes ledger-based deposits** | The paginated flat deposits view only iterates over timestamp-based `VaultEntry` records. To enumerate ledger-based deposits, use `get_depositors` + `get_deposit_ids` + `get_ledger_vault`. |

These limitations are tracked as open issues and will be addressed in future releases.

---

```bash
make build            # Compile to WASM
make test             # Run all tests
make lint             # Clippy
make fmt              # Format
make check            # fmt + lint + test + audit + deny
make optimize         # Optimize WASM with soroban CLI
make check-wasm-size  # Fail if WASM > 64 KB
make deploy-testnet   # Deploy to Stellar testnet
make smoke-test-local # End-to-end test against local node
make audit            # cargo audit (security)
make deny             # cargo deny (licenses)
```

---

## Use Cases

- **Savings** - Lock funds for a fixed period to enforce discipline
- **Token vesting** - Team/investor tokens released on a schedule
- **HODL commitments** - Commit to not selling until a future date
- **Escrow** - Time-gated release of payment

---

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) and [CHANGELOG.md](./CHANGELOG.md).

## License

MIT
