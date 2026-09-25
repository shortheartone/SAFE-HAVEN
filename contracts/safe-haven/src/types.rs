use soroban_sdk::{contracttype, Address, Vec};

pub const MAX_DEPOSIT_AMOUNT: i128 = 1_000_000_000_000_000;
pub const MAX_LOCK_DURATION_SECS: u64 = 157_788_000;
pub const MIN_LOCK_DURATION_SECS: u64 = 60;

/// Maximum number of tokens allowed in a single multi-token deposit (issue #330).
pub const MAX_TOKENS_PER_DEPOSIT: u32 = 5;

/// Current storage schema version. Bump this constant when the on-chain
/// layout of a `contracttype` struct changes so `migrate()` can detect
/// and upgrade stale entries.
pub const STORAGE_VERSION: u32 = 1;

/// Fraction of the penalty fee reserved for the insurance pool (5 = 5%).
pub const INSURANCE_POOL_BPS: u32 = 500; // 5% in basis points

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DepositType {
    TimeBased,
    LedgerBased,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DepositRequest {
    pub token: Address,
    pub amount: i128,
    pub unlock_time: u64,
    pub penalty_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DepositType {
    TimeBased,
    LedgerBased,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VaultKey {
    Deposit(Address, u32),
    DepositByLedger(Address, u32),
    /// Multi-token deposit entry (issue #330).
    MultiDeposit(Address, u32),
    /// Withdrawal whitelist for a deposit (issue #331).
    WithdrawalWhitelist(Address, u32),
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
    /// Set-once flag recording that an address was appended to `DepositorList`.
    /// Never deleted, so re-deposits don't create duplicate list entries even
    /// after the corresponding `DepositorFlag` has been cleared by `remove_depositor`.
    DepositorInList(Address),
    FeeRecipient,
    MaxDeposit,
    MaxLockSecs,
    Paused,
    /// Boolean membership flag for the token allowlist.
    AllowedToken(Address),
    /// When true, deposits may use only tokens in the allowlist.
    StrictTokenAllowlist,
    /// Stores the token vetting workflow state.
    TokenVetting(Address),
    ProposalCounter,
    GovernanceProposal(u32),
    GovernanceVote(u32, Address),
    /// Persists the schema version written by the last `migrate()` call (or 1
    /// for contracts that were initialized before versioning was introduced).
    StorageVersion,
    /// Staker entry: maps staker address to their stake amount
    Staker(Address),
    /// List of all registered stakers
    StakerList,
    /// Flag to track if a staker is in the StakerList (prevents duplicates)
    StakerInList(Address),
    /// Total amount staked by all stakers
    TotalStaked,
    /// Rewards pool for stakers (accumulated from penalties)
    RewardsPool,
    /// Rewards claimed by a staker (track cumulative for auditing)
    StakerRewardsClaimed(Address),
    /// Encrypted metadata for a specific deposit (depositor, deposit_id).
    /// Key material is never stored; only the ciphertext + nonce + auth_tag are persisted.
    EncryptedMetadata(Address, u32),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaultEntry {
    pub token: Address,
    pub amount: i128,
    pub unlock_time: u64,
    pub depositor: Address,
    pub penalty_bps: u32,
    /// Compound interest accrual frequency in seconds (0 = no compounding). (issue #332)
    pub compound_frequency_secs: u64,
    /// Timestamp of last compound accrual (issue #332).
    pub last_accrual_timestamp: u64,
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

/// A single token+amount pair used in multi-token deposits (issue #330).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenDeposit {
    pub token: Address,
    pub amount: i128,
}

/// Vault entry that holds multiple token deposits (issue #330).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiTokenVaultEntry {
    /// Each element is a (token, amount) pair. Length ≤ MAX_TOKENS_PER_DEPOSIT.
    pub tokens: Vec<TokenDeposit>,
    pub unlock_time: u64,
    pub depositor: Address,
    pub penalty_bps: u32,
    /// Compound interest accrual frequency in seconds (0 = no compounding). (issue #332)
    pub compound_frequency_secs: u64,
    /// Timestamp of last compound accrual (issue #332).
    pub last_accrual_timestamp: u64,
}

/// The deposit type discriminant returned by `get_deposit_type`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DepositType {
    /// Timestamp-based (`VaultEntry`) single-token deposit.
    TimeBased,
    /// Ledger-sequence-based (`LedgerVaultEntry`) deposit.
    LedgerBased,
    /// Multi-token timestamp-based deposit (`MultiTokenVaultEntry`). (issue #330)
    MultiToken,
}

/// Paginated query result for depositor addresses.
/// (Soroban `#[contracttype]` does not support generics.)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Page {
    /// The items in this page
    pub items: soroban_sdk::Vec<Address>,
    /// Total number of active items across all pages
    pub total_count: u32,
}

/// Staker entry: tracks stake amount and optionally last claim timestamp
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StakerEntry {
    pub staker: Address,
    pub stake_amount: i128,
}

// ----------------------------------------------------------------
//  Encrypted Metadata
// ----------------------------------------------------------------

/// Encrypted metadata attached to a vault deposit.
///
/// ## Encryption scheme: HMAC-CTR (XOR-stream cipher)
///
/// Because the Soroban WASM sandbox is `no_std` and provides only
/// `env.crypto().sha256()`, we implement a lightweight authenticated
/// XOR-stream cipher:
///
/// 1. **Key derivation** (prevents direct SHA256 length-extension attacks):
///    ```
///    round_key(i) = SHA256(key_bytes || nonce_bytes || i_as_4_le_bytes)
///    ```
///    Each 32-byte block of keystream is produced by hashing the caller's
///    32-byte key, the 8-byte nonce, and the 4-byte block counter.
///
/// 2. **Encryption**:
///    ```
///    ciphertext[i] = plaintext[i] XOR keystream_byte(i)
///    ```
///
/// 3. **Authentication tag** — a final SHA256 over `(key || nonce || ciphertext)`:
///    ```
///    tag = SHA256(key || nonce || ciphertext)
///    ```
///    The tag is verified before decryption; if it does not match, the
///    function returns `DecryptionFailed` without exposing partial plaintext.
///
/// 4. **Nonce** — derived deterministically from `(ledger_sequence, deposit_id)`:
///    ```
///    nonce = SHA256(ledger_sequence_as_4_le_bytes || deposit_id_as_4_le_bytes)[0..8]
///    ```
///    The nonce is stored alongside the ciphertext so decryption does not
///    require the original ledger sequence.
///
/// ## Security properties
///
/// | Property | Guarantee |
/// |---|---|
/// | Confidentiality | Ciphertext is XOR-masked; key required to recover plaintext |
/// | Integrity | SHA256-based MAC tag; any bit-flip in ciphertext is detected |
/// | Auth enforcement | `require_auth()` checked before every encrypt/decrypt operation |
/// | Nonce uniqueness | Nonce derived from (ledger_sequence, deposit_id); changes on each new deposit |
/// | Key rotation | `rotate_encryption_key` decrypts with old key then re-encrypts with new key atomically |
///
/// ## Limitations
///
/// * The scheme is NOT IND-CCA2 secure; it provides authenticated encryption
///   equivalent to HMAC-CTR with a SHA256 PRF. Sufficient for metadata
///   confidentiality on-chain but not a substitute for AES-GCM in general use.
/// * Maximum plaintext size is `MAX_METADATA_BYTES` (512 bytes) to stay within
///   the Soroban instruction budget.
/// * Key material is supplied by the caller at encrypt/decrypt time and is
///   NEVER stored on-chain.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncryptedMetadata {
    /// XOR-stream ciphertext of the original metadata bytes.
    pub ciphertext: soroban_sdk::Bytes,
    /// 8-byte nonce derived from `(ledger_sequence, deposit_id)` at encryption time.
    pub nonce: soroban_sdk::BytesN<8>,
    /// SHA256 authentication tag over `(key || nonce || ciphertext)`.
    pub auth_tag: soroban_sdk::BytesN<32>,
    /// Ledger sequence at which this metadata was last encrypted or rotated.
    pub encrypted_at_ledger: u32,
}

/// Maximum plaintext metadata size in bytes.
/// Chosen to stay comfortably within the Soroban instruction budget.
pub const MAX_METADATA_BYTES: u32 = 512;

/// Deposit type indicator — distinguishes between timestamp-based and ledger-based deposits
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DepositType {
    TimeBased,
    LedgerBased,
}
