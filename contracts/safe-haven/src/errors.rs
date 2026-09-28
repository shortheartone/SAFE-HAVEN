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
    /// `initialize` was called on an already-initialized contract.
    /// The `is_initialized` flag is the sole re-initialization guard (closes #46).
    AlreadyInitialized = 15,
    /// Too many tokens in a multi-token deposit — exceeds MAX_TOKENS_PER_DEPOSIT (issue #330).
    TooManyTokens = 16,
    /// A multi-token deposit must contain at least one token (issue #330).
    EmptyTokenList = 17,
    /// Recipient is not on the withdrawal whitelist (issue #331).
    RecipientNotWhitelisted = 18,
    /// Compound frequency must be >= 60 seconds if non-zero (issue #332).
    InvalidCompoundFrequency = 19,
    /// Sponsorship fund not initialized
    SponsorshipNotInitialized = 20,
    /// User not eligible for sponsorship
    NotEligibleForSponsorship = 21,
    /// Sponsorship fund insufficient
    InsufficientSponsorshipFund = 22,
    /// User exceeded daily sponsorship limit
    SponsorshipDailyLimitExceeded = 23,
    /// User transaction cooldown still active
    SponsorshipCooldownActive = 24,
    /// Potential sybil attack detected
    PotentialSybilAttack = 25,
    /// Sponsorship configuration error
    InvalidSponsorshipConfig = 26,
    /// Depositor exceeded the maximum number of deposits per rate-limiting window (issue #492).
    DepositRateLimitExceeded = 27,
    /// Scheduled deposit not found (issue #494).
    ScheduledDepositNotFound = 30,
    /// Scheduled deposit execution time has not yet arrived (issue #494).
    ScheduledDepositNotReady = 31,
    /// Scheduled deposit has already been executed (issue #494).
    ScheduledDepositAlreadyExecuted = 32,
    /// Scheduled deposit has been cancelled (issue #494).
    ScheduledDepositCancelled = 33,
    /// No insurance claim found for the given claim_id (issue #493).
    NoClaimFound = 34,
    /// Insurance claim has already been resolved (approved or denied) (issue #493).
    ClaimAlreadyResolved = 35,
    /// Insurance pool balance is insufficient for the requested claim amount (issue #493).
    InsufficientInsurancePool = 36,
}
