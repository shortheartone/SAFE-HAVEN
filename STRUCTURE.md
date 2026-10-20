# Project Structure

This repository is split into two independent layers:

```
SAFE-HAVEN/
├── contracts/                  ← Rust / Soroban smart contract (backend)
│   └── safe-haven/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs          # Crate root
│           ├── contract.rs     # All public entry points
│           ├── types.rs        # VaultKey, VaultEntry, constants
│           ├── errors.rs       # VaultError enum (12 codes)
│           ├── events.rs       # Event emission helpers
│           ├── storage.rs      # Persistent storage + TTL helpers
│           ├── mev_hash.rs     # MEV commit hash (Keccak256)
│           └── test.rs         # 48+ unit tests
│
├── frontend/                   ← React + TypeScript + Vite (UI layer)
│   ├── src/
│   │   ├── App.tsx             # Root component, tab routing
│   │   ├── config.ts           # Contract ID, RPC URLs, constants
│   │   ├── types.ts            # Shared TypeScript types
│   │   ├── context/
│   │   │   └── WalletContext.tsx    # Freighter wallet state + signing
│   │   ├── hooks/
│   │   │   ├── useDeposits.ts       # Load deposits for connected wallet
│   │   │   └── useContractInfo.ts   # Admin/paused/constants
│   │   ├── lib/
│   │   │   ├── stellar.ts           # Contract reads + tx builders
│   │   │   ├── format.ts            # Stroops, dates, countdown, BPS
│   │   │   ├── mev-hash.ts          # MEV commit hash (Keccak256)
│   │   │   ├── mev-hash.test.ts     # Unit tests for MEV hash
│   │   │   └── mev-hash.integration.ts # Cross-language test vectors
│   │   ├── components/
│   │   │   ├── Header.tsx      # Top nav + wallet connect button
│   │   │   ├── TabNav.tsx      # Page tab switcher
│   │   │   ├── DepositCard.tsx # Single deposit UI card
│   │   │   └── TxStatusBadge.tsx # Signing → submitting → confirmed
│   │   └── pages/
│   │       ├── Dashboard.tsx   # My vaults overview
│   │       ├── DepositPage.tsx # New deposit form
│   │       ├── WithdrawPage.tsx # Withdraw / cancel form
│   │       └── AdminPage.tsx   # Admin controls
│   ├── .env.example            # Environment variable template
│   ├── package.json
│   ├── vite.config.ts
│   ├── tailwind.config.js
│   └── README.md               # Frontend-specific docs
│
├── Cargo.toml                  # Rust workspace
├── Cargo.lock
├── Makefile                    # Build / test / lint / deploy helpers
├── MEV_SPEC.md                 # MEV commit hash specification
├── rust-toolchain.toml
├── README.md                   # Contract documentation
└── STRUCTURE.md                # This file
```

## Quick Start

### Smart Contract

```bash
# Build the WASM
make build

# Run all tests
make test

# Deploy to testnet
export SOROBAN_SECRET_KEY=S...
make deploy-testnet
```

### Frontend

```bash
cd frontend
npm install
cp .env.example .env      # paste your contract ID into VITE_CONTRACT_ID
npm run dev               # → http://localhost:5173
```

See [`frontend/README.md`](./frontend/README.md) for the full frontend guide.

## MEV Commit Hash

The protocol includes a MEV (Maximal Extractable Value) commit hash mechanism to prevent transaction ordering manipulation. See [`MEV_SPEC.md`](./MEV_SPEC.md) for complete specification.

### Implementations

- **Rust** (`contracts/safe-haven/src/mev_hash.rs`): `compute_mev_hash()`, `verify_mev_hash()`, `hash_to_hex()`
- **TypeScript** (`frontend/src/lib/mev-hash.ts`): `computeMevHash()`, `verifyMevHash()`, encoding helpers
- **Tests** (`frontend/src/lib/mev-hash.test.ts`, `contracts/safe-haven/src/mev_hash.rs`): Unit tests + integration vectors

### Format

```
commit_hash = Keccak256(token || amount || price || nonce)
```

All fields are encoded in big-endian byte order:
- `token`: 20 bytes (Stellar address)
- `amount`: 16 bytes (i128)
- `price`: 8 bytes (u64)
- `nonce`: 4 bytes (u32)

**Total input: 48 bytes → 32-byte Keccak256 hash → 64-char hex string**

## Architecture Overview

```
Browser (React + Freighter)
        │
        │  signTransaction(xdr)
        ▼
Freighter Wallet Extension
        │
        │  signed XDR
        ▼
 Soroban RPC Server
        │
        │  sendTransaction / simulateTransaction
        ▼
SAFE-HAVEN Contract (Soroban / Stellar)
        │
        ├─ deposit()         → creates VaultEntry in persistent storage
        ├─ withdraw()        → validates timestamp, removes entry, transfers tokens
        ├─ cancel_deposit()  → early exit with configurable penalty
        └─ emergency_withdraw() → admin-only recovery (funds → depositor)
```
