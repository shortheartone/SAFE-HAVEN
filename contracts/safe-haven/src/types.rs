use soroban_sdk::{contracttype, Address};

pub const MAX_DEPOSIT_AMOUNT: i128 = 1_000_000_000_000_000;
pub const MAX_LOCK_DURATION_SECS: u64 = 157_788_000;
pub const MIN_LOCK_DURATION_SECS: u64 = 60;

/// Current storage schema version. Bump this constant when the on-chain
/// layout of a `contracttype` struct changes so `migrate()` can detect
/// and upgrade stale entries.
pub const STORAGE_VERSION: u32 = 1;

// ----------------------------------------------------------------
//  Notification Preferences Types
// ----------------------------------------------------------------

/// Priority level assigned to each event type.
/// Higher priority events are more likely to be surfaced by off-chain listeners.
/// - `Critical`: Must-see events (e.g. emergency withdrawals, admin changes).
/// - `High`:     Important user-facing events (deposits, withdrawals, cancellations).
/// - `Medium`:   Informational events (lock extensions, pause/unpause).
/// - `Low`:      Verbose / administrative events (initialized, storage migration).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NotificationPriority {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

/// Every distinct event type the contract can emit.
/// Users toggle individual event types on or off in their `NotificationPreferences`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventType {
    Deposit,
    DepositByLedger,
    Withdraw,
    WithdrawTo,
    DepositCancelled,
    EmergencyWithdraw,
    LockExtended,
    Paused,
    Unpaused,
    AdminTransferInitiated,
    AdminTransferAccepted,
    AdminTransferCancelled,
    AdminRenounced,
    ContractInitialized,
}

/// Per-user notification preferences stored on-chain.
///
/// Each boolean field controls whether the corresponding event type should
/// trigger a notification for this user.  Off-chain indexers read these
/// preferences to decide whether to forward an event to the user.
///
/// The `min_priority` field lets users suppress all events below a given
/// importance level regardless of individual toggles.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotificationPreferences {
    /// Suppress any event whose priority is below this level.
    pub min_priority: NotificationPriority,
    /// Receive deposit notifications.
    pub deposit: bool,
    /// Receive deposit-by-ledger notifications.
    pub deposit_by_ledger: bool,
    /// Receive withdraw notifications.
    pub withdraw: bool,
    /// Receive withdraw-to notifications.
    pub withdraw_to: bool,
    /// Receive deposit-cancelled notifications.
    pub deposit_cancelled: bool,
    /// Receive emergency-withdraw notifications.
    pub emergency_withdraw: bool,
    /// Receive lock-extended notifications.
    pub lock_extended: bool,
    /// Receive paused/unpaused notifications.
    pub paused: bool,
    /// Receive admin-transfer notifications (initiated / accepted / cancelled).
    pub admin_transfer: bool,
    /// Receive admin-renounced notifications.
    pub admin_renounced: bool,
    /// Receive contract-initialized notifications.
    pub contract_initialized: bool,
}

impl NotificationPreferences {
    /// Default preferences: everything enabled at High priority and above.
    pub fn default(env: &soroban_sdk::Env) -> Self {
        let _ = env; // env not needed currently but keeps signature consistent
        NotificationPreferences {
            min_priority: NotificationPriority::Low,
            deposit: true,
            deposit_by_ledger: true,
            withdraw: true,
            withdraw_to: true,
            deposit_cancelled: true,
            emergency_withdraw: true,
            lock_extended: true,
            paused: true,
            admin_transfer: true,
            admin_renounced: true,
            contract_initialized: true,
        }
    }

    /// Returns `true` when the given event type and its associated priority
    /// pass this user's preference filters.
    pub fn allows(&self, event_type: &EventType) -> bool {
        // Check the per-event-type toggle.
        let enabled = match event_type {
            EventType::Deposit => self.deposit,
            EventType::DepositByLedger => self.deposit_by_ledger,
            EventType::Withdraw => self.withdraw,
            EventType::WithdrawTo => self.withdraw_to,
            EventType::DepositCancelled => self.deposit_cancelled,
            EventType::EmergencyWithdraw => self.emergency_withdraw,
            EventType::LockExtended => self.lock_extended,
            EventType::Paused | EventType::Unpaused => self.paused,
            EventType::AdminTransferInitiated
            | EventType::AdminTransferAccepted
            | EventType::AdminTransferCancelled => self.admin_transfer,
            EventType::AdminRenounced => self.admin_renounced,
            EventType::ContractInitialized => self.contract_initialized,
        };
        if !enabled {
            return false;
        }

        // Check the priority threshold.
        let priority = event_priority(event_type);
        priority_value(&priority) >= priority_value(&self.min_priority)
    }
}

/// Returns the canonical priority level for each event type.
pub fn event_priority(event_type: &EventType) -> NotificationPriority {
    match event_type {
        // Critical — always important regardless of who is watching
        EventType::EmergencyWithdraw => NotificationPriority::Critical,
        EventType::AdminRenounced => NotificationPriority::Critical,
        // High — primary user-facing actions
        EventType::Deposit => NotificationPriority::High,
        EventType::DepositByLedger => NotificationPriority::High,
        EventType::Withdraw => NotificationPriority::High,
        EventType::WithdrawTo => NotificationPriority::High,
        EventType::DepositCancelled => NotificationPriority::High,
        EventType::AdminTransferInitiated => NotificationPriority::High,
        EventType::AdminTransferAccepted => NotificationPriority::High,
        EventType::AdminTransferCancelled => NotificationPriority::High,
        // Medium — operational / state changes
        EventType::LockExtended => NotificationPriority::Medium,
        EventType::Paused => NotificationPriority::Medium,
        EventType::Unpaused => NotificationPriority::Medium,
        // Low — informational / one-time setup
        EventType::ContractInitialized => NotificationPriority::Low,
    }
}

/// Numeric value of a `NotificationPriority` for comparison.
pub fn priority_value(priority: &NotificationPriority) -> u32 {
    match priority {
        NotificationPriority::Low => 0,
        NotificationPriority::Medium => 1,
        NotificationPriority::High => 2,
        NotificationPriority::Critical => 3,
    }
}

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
    /// Set-once flag recording that an address was appended to `DepositorList`.
    /// Never deleted, so re-deposits don't create duplicate list entries even
    /// after the corresponding `DepositorFlag` has been cleared by `remove_depositor`.
    DepositorInList(Address),
    FeeRecipient,
    MaxDeposit,
    MaxLockSecs,
    Paused,
    /// Persists the schema version written by the last `migrate()` call (or 1
    /// for contracts that were initialized before versioning was introduced).
    StorageVersion,
    /// Per-user notification preferences — keyed by the user's `Address`.
    NotificationPreferences(Address),
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
