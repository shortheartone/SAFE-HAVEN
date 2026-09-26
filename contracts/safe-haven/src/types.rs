use soroban_sdk::{contracttype, Address, Bytes};

pub const MAX_DEPOSIT_AMOUNT: i128 = 1_000_000_000_000_000;
pub const MAX_LOCK_DURATION_SECS: u64 = 157_788_000;
pub const MIN_LOCK_DURATION_SECS: u64 = 60;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VaultKey {
    Deposit(Address, u32),
    DepositByLedger(Address, u32),
    DepositCounter(Address),
    /// Stores a `Vec<u32>` of active deposit IDs for a depositor (both timestamp- and
    /// ledger-based). Maintained alongside the counter so `get_deposit_ids` is O(1).
    ActiveDepositIds(Address),
    Admin,
    PendingAdmin,
    Initialized,
    DepositorList,
    /// Boolean existence flag per depositor — O(1) duplicate check in `add_depositor`.
    DepositorFlag(Address),
    FeeRecipient,
    MaxDeposit,
    MaxLockSecs,
    Paused,
    // ---- Privacy keys ----
    /// Marks that `Address` has opted into privacy mode.
    PrivacyEnabled(Address),
    /// Stores the `PrivateVaultEntry` for a private deposit.
    /// Keyed by (depositor, deposit_id) — same namespace as `Deposit` for counter sharing.
    PrivateDeposit(Address, u32),
    /// Nullifier spent flag: `true` once a nullifier has been used for withdrawal.
    /// Stored under the 32-byte nullifier value so no address linkage is possible.
    Nullifier(Bytes),
    /// Marks an address as an authorized private-deposit auditor.
    Auditor(Address),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaultEntry {
    pub token: Address,
    pub amount: i128,
    pub unlock_time: u64,
    pub depositor: Address,
    pub penalty_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LedgerVaultEntry {
    pub token: Address,
    pub amount: i128,
    pub unlock_ledger: u32,
    pub depositor: Address,
    pub penalty_bps: u32,
}

// ----------------------------------------------------------------
//  Privacy types
// ----------------------------------------------------------------

/// A private vault entry. The `amount` is **not** stored here — it is hidden
/// inside the commitment. The owner proves knowledge of the amount by
/// supplying the preimage (`secret`, `token`, `amount`, `salt`) during
/// withdrawal, which the contract re-hashes and compares against the
/// stored commitment.
///
/// Layout of the commitment preimage (all big-endian):
///   commitment = SHA-256(secret[32] || token_address[32] || amount[16] || salt[32])
///
/// Layout of the nullifier preimage:
///   nullifier  = SHA-256(secret[32] || deposit_id[4])
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivateVaultEntry {
    /// SHA-256 commitment that hides the depositor's token address and amount.
    pub commitment: Bytes,
    /// Lock expiry timestamp (seconds). The unlock time is public so the
    /// contract can enforce it without knowing the amount.
    pub unlock_time: u64,
    /// Early-exit penalty in basis points (0–10 000). Public so penalty math
    /// can be done on-chain without revealing the amount.
    pub penalty_bps: u32,
}

/// Preimage supplied to a private balance query or audit request.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivateBalanceProof {
    pub deposit_id: u32,
    pub secret: Bytes,
    pub token: Address,
    pub amount: i128,
    pub salt: Bytes,
}
