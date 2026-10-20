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
│       ├── errors.rs           VaultError enum (12 codes)
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

## MEV Commit Hash

The protocol includes a MEV (Maximal Extractable Value) commit hash mechanism to prevent transaction ordering manipulation. The hash combines deposit parameters (token, amount, price, nonce) using the Keccak256 algorithm with big-endian encoding.

See [MEV_SPEC.md](./MEV_SPEC.md) for:
- Complete hash format specification
- Encoding details and examples
- TypeScript and Rust implementations
- Cross-language test vectors

---

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
6. **Admin recovery** - Admin can emergency-withdraw any deposit (funds always go to depositor, never admin)
7. **Trustless mode** - Admin can be permanently renounced via `renounce_admin()`

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
Locks tokens until a specific ledger sequence number.

#### `withdraw(depositor, deposit_id)`
Withdraws if `now >= unlock_time`.

#### `withdraw_to(depositor, deposit_id, recipient)`
Withdraws to a different address.

#### `cancel_deposit(depositor, deposit_id)`
Early exit with penalty. Penalty goes to fee_recipient; remainder returned to depositor.

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
| 2 | `UnlockTimeNotInFuture` | unlock_time <= now |
| 3 | `NoDepositFound` | No active deposit |
| 4 | `FundsStillLocked` | Lock not yet expired |
| 5 | `DepositAlreadyExists` | (reserved) |
| 6 | `LockDurationTooLong` | Exceeds 5 years |
| 7 | `Unauthorized` | Not the admin |
| 8 | `AmountTooLarge` | Exceeds 10^15 |
| 9 | `InvalidPenaltyBps` | penalty_bps > 10000 |
| 10 | `InvalidAdmin` | Same as current admin |
| 11 | `LockDurationTooShort` | Less than 60 seconds |
| 12 | `ContractPaused` | Deposits paused |

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

## Makefile Commands

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
