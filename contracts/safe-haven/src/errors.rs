use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum VaultError {
    InvalidAmount = 1,
    UnlockTimeNotInFuture = 2,
    NoDepositFound = 3,
    FundsStillLocked = 4,
    DepositAlreadyExists = 5,
    LockDurationTooLong = 6,
    Unauthorized = 7,
    AmountTooLarge = 8,
    InvalidPenaltyBps = 9,
    InvalidAdmin = 10,
    LockDurationTooShort = 11,
    ContractPaused = 12,
    VaultAlreadyUnlocked = 13,
    MissingFeeRecipient = 14,
    // ---- Privacy errors (15–18) ----
    /// The provided commitment preimage does not match the stored commitment.
    InvalidCommitment = 15,
    /// This nullifier has already been spent — withdrawal already occurred.
    NullifierAlreadyUsed = 16,
    /// The caller has not opted into privacy mode via `enable_privacy()`.
    PrivacyNotEnabled = 17,
    /// Commitment byte length is not exactly 32 bytes.
    InvalidCommitmentLength = 18,
    /// The caller is not an authorized private-deposit auditor.
    AuditorUnauthorized = 19,
}
