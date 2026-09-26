// ============================================================
//  SAFE-HAVEN — Soroban Smart Contract
//  Stellar Blockchain | Soroban SDK v22
// ============================================================

use soroban_sdk::{contract, contractimpl, token, xdr::ToXdr, Address, Bytes, Env, Vec};

use crate::{
    constants::{MAX_BATCH_SIZE, MAX_DEPOSIT_AMOUNT, MAX_LOCK_DURATION_SECS, MIN_LOCK_DURATION_SECS},
    errors::VaultError,
    events, storage,
    types::{LedgerVaultEntry, PrivateBalanceProof, PrivateVaultEntry, VaultEntry},
};

#[contract]
pub struct SafeHaven;

#[contractimpl]
impl SafeHaven {
    // ----------------------------------------------------------------
    //  Initialization
    // ----------------------------------------------------------------

    pub fn initialize(
        env: Env,
        admin: Address,
        fee_recipient: Address,
        max_deposit: Option<i128>,
        max_lock_secs: Option<u64>,
    ) -> Result<(), VaultError> {
        admin.require_auth();

        if storage::is_initialized(&env) {
            return Err(VaultError::Unauthorized);
        }

        storage::set_admin(&env, &admin);
        storage::set_initialized(&env);
        storage::set_fee_recipient(&env, &fee_recipient);

        if let Some(v) = max_deposit {
            if v <= 0 {
                return Err(VaultError::InvalidAmount);
            }
            storage::set_max_deposit(&env, v);
        }

        if let Some(v) = max_lock_secs {
            if v == 0 {
                return Err(VaultError::LockDurationTooLong);
            }
            storage::set_max_lock_secs(&env, v);
        }

        let effective_max_deposit = storage::get_max_deposit(&env).unwrap_or(MAX_DEPOSIT_AMOUNT);
        let effective_max_lock = storage::get_max_lock_secs(&env).unwrap_or(MAX_LOCK_DURATION_SECS);
        events::contract_initialized(&env, &admin, &fee_recipient, effective_max_deposit, effective_max_lock);

        Ok(())
    }

    // ----------------------------------------------------------------
    //  Core: Deposit
    // ----------------------------------------------------------------

    pub fn deposit(
        env: Env,
        depositor: Address,
        token: Address,
        amount: i128,
        unlock_time: u64,
        penalty_bps: u32,
    ) -> Result<u32, VaultError> {
        depositor.require_auth();

        if storage::is_paused(&env) {
            return Err(VaultError::ContractPaused);
        }

        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }

        let max_deposit = storage::get_max_deposit(&env).unwrap_or(MAX_DEPOSIT_AMOUNT);
        if amount > max_deposit {
            return Err(VaultError::AmountTooLarge);
        }

        if penalty_bps > 10_000 {
            return Err(VaultError::InvalidPenaltyBps);
        }

        if penalty_bps > 0 && storage::get_fee_recipient(&env).is_none() {
            return Err(VaultError::MissingFeeRecipient);
        }

        let now = env.ledger().timestamp();
        if unlock_time <= now {
            return Err(VaultError::UnlockTimeNotInFuture);
        }

        let max_lock = storage::get_max_lock_secs(&env).unwrap_or(MAX_LOCK_DURATION_SECS);
        let lock_duration: u64 = unlock_time.saturating_sub(now);
        if lock_duration > max_lock {
            return Err(VaultError::LockDurationTooLong);
        }
        if lock_duration < MIN_LOCK_DURATION_SECS {
            return Err(VaultError::LockDurationTooShort);
        }

        let deposit_id = storage::next_deposit_id(&env, &depositor);

        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&depositor, &env.current_contract_address(), &amount);

        let entry = VaultEntry {
            token: token.clone(),
            amount,
            unlock_time,
            depositor: depositor.clone(),
            penalty_bps,
        };

        storage::set_deposit(&env, &depositor, deposit_id, &entry);
        storage::add_depositor(&env, &depositor);
        events::deposit(&env, &depositor, &token, amount, unlock_time, deposit_id);

        Ok(deposit_id)
    }

    pub fn deposit_for(
        env: Env,
        payer: Address,
        depositor: Address,
        token: Address,
        amount: i128,
        unlock_time: u64,
        penalty_bps: u32,
    ) -> Result<u32, VaultError> {
        payer.require_auth();

        if storage::is_paused(&env) {
            return Err(VaultError::ContractPaused);
        }

        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }

        let max_deposit = storage::get_max_deposit(&env).unwrap_or(MAX_DEPOSIT_AMOUNT);
        if amount > max_deposit {
            return Err(VaultError::AmountTooLarge);
        }

        if penalty_bps > 10_000 {
            return Err(VaultError::InvalidPenaltyBps);
        }

        if penalty_bps > 0 && storage::get_fee_recipient(&env).is_none() {
            return Err(VaultError::MissingFeeRecipient);
        }

        let now = env.ledger().timestamp();
        if unlock_time <= now {
            return Err(VaultError::UnlockTimeNotInFuture);
        }

        let max_lock = storage::get_max_lock_secs(&env).unwrap_or(MAX_LOCK_DURATION_SECS);
        let lock_duration: u64 = unlock_time.saturating_sub(now);
        if lock_duration > max_lock {
            return Err(VaultError::LockDurationTooLong);
        }
        if lock_duration < MIN_LOCK_DURATION_SECS {
            return Err(VaultError::LockDurationTooShort);
        }

        let deposit_id = storage::next_deposit_id(&env, &depositor);

        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&payer, &env.current_contract_address(), &amount);

        let entry = VaultEntry {
            token: token.clone(),
            amount,
            unlock_time,
            depositor: depositor.clone(),
            penalty_bps,
        };

        storage::set_deposit(&env, &depositor, deposit_id, &entry);
        storage::add_depositor(&env, &depositor);
        events::deposit(&env, &depositor, &token, amount, unlock_time, deposit_id);

        Ok(deposit_id)
    }

    // ----------------------------------------------------------------
    //  Core: Deposit by Ledger Sequence (Issue #88)
    // ----------------------------------------------------------------

    pub fn deposit_by_ledger(
        env: Env,
        depositor: Address,
        token: Address,
        amount: i128,
        unlock_ledger: u32,
        penalty_bps: u32,
    ) -> Result<u32, VaultError> {
        depositor.require_auth();

        if storage::is_paused(&env) {
            return Err(VaultError::ContractPaused);
        }

        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }

        let max_deposit = storage::get_max_deposit(&env).unwrap_or(MAX_DEPOSIT_AMOUNT);
        if amount > max_deposit {
            return Err(VaultError::AmountTooLarge);
        }

        if penalty_bps > 10_000 {
            return Err(VaultError::InvalidPenaltyBps);
        }

        if penalty_bps > 0 && storage::get_fee_recipient(&env).is_none() {
            return Err(VaultError::MissingFeeRecipient);
        }

        let current_ledger = env.ledger().sequence();
        if unlock_ledger <= current_ledger {
            return Err(VaultError::UnlockTimeNotInFuture);
        }

        let deposit_id = storage::next_deposit_id(&env, &depositor);

        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&depositor, &env.current_contract_address(), &amount);

        let entry = LedgerVaultEntry {
            token: token.clone(),
            amount,
            unlock_ledger,
            depositor: depositor.clone(),
            penalty_bps,
        };

        storage::set_deposit_by_ledger(&env, &depositor, deposit_id, &entry);
        storage::add_depositor(&env, &depositor);
        events::deposit_by_ledger(&env, &depositor, &token, amount, unlock_ledger, deposit_id);

        Ok(deposit_id)
    }

    // ----------------------------------------------------------------
    //  Core: Cancel Deposit (early exit with penalty)
    // ----------------------------------------------------------------

    pub fn cancel_deposit(env: Env, depositor: Address, deposit_id: u32) -> Result<(), VaultError> {
        depositor.require_auth();

        // Try timestamp-based deposit first
        if let Some(entry) = storage::get_deposit(&env, &depositor, deposit_id) {
            let now = env.ledger().timestamp();
            if now >= entry.unlock_time {
                return Err(VaultError::VaultAlreadyUnlocked);
            }

            storage::remove_deposit(&env, &depositor, deposit_id);
            if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                storage::remove_depositor(&env, &depositor);
            }

            let token_client = token::Client::new(&env, &entry.token);
            let contract = env.current_contract_address();

            let penalty: i128 = (entry.amount * entry.penalty_bps as i128) / 10_000;
            let refund = entry.amount - penalty;

            if penalty > 0 {
                let fee_recipient =
                    storage::get_fee_recipient(&env).ok_or(VaultError::MissingFeeRecipient)?;
                token_client.transfer(&contract, &fee_recipient, &penalty);
            }
            if refund > 0 {
                token_client.transfer(&contract, &depositor, &refund);
            }

            events::deposit_cancelled(&env, &depositor, &entry.token, entry.amount, penalty, deposit_id);
            return Ok(());
        }

        // Try ledger-based deposit
        if let Some(entry) = storage::get_deposit_by_ledger_readonly(&env, &depositor, deposit_id) {
            let current_ledger = env.ledger().sequence();
            if current_ledger >= entry.unlock_ledger {
                return Err(VaultError::VaultAlreadyUnlocked);
            }

            storage::remove_deposit_by_ledger(&env, &depositor, deposit_id);
            if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                storage::remove_depositor(&env, &depositor);
            }

            let token_client = token::Client::new(&env, &entry.token);
            let contract = env.current_contract_address();

            let penalty: i128 = (entry.amount * entry.penalty_bps as i128) / 10_000;
            let refund = entry.amount - penalty;

            if penalty > 0 {
                let fee_recipient =
                    storage::get_fee_recipient(&env).ok_or(VaultError::MissingFeeRecipient)?;
                token_client.transfer(&contract, &fee_recipient, &penalty);
            }
            if refund > 0 {
                token_client.transfer(&contract, &depositor, &refund);
            }

            events::deposit_cancelled(&env, &depositor, &entry.token, entry.amount, penalty, deposit_id);
            return Ok(());
        }

        Err(VaultError::NoDepositFound)
    }

    // ----------------------------------------------------------------
    //  Core: Withdraw
    // ----------------------------------------------------------------

    pub fn withdraw(env: Env, depositor: Address, deposit_id: u32) -> Result<(), VaultError> {
        depositor.require_auth();

        // Try timestamp-based deposit first
        if let Some(entry) = storage::get_deposit_readonly(&env, &depositor, deposit_id) {
            let now = env.ledger().timestamp();
            if now < entry.unlock_time {
                return Err(VaultError::FundsStillLocked);
            }

            storage::remove_deposit(&env, &depositor, deposit_id);
            if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                storage::remove_depositor(&env, &depositor);
            }

            let token_client = token::Client::new(&env, &entry.token);
            token_client.transfer(&env.current_contract_address(), &depositor, &entry.amount);

            events::withdraw(&env, &depositor, &entry.token, entry.amount, deposit_id);
            return Ok(());
        }

        // Try ledger-based deposit
        if let Some(entry) = storage::get_deposit_by_ledger_readonly(&env, &depositor, deposit_id) {
            let current_ledger = env.ledger().sequence();
            if current_ledger < entry.unlock_ledger {
                return Err(VaultError::FundsStillLocked);
            }

            storage::remove_deposit_by_ledger(&env, &depositor, deposit_id);
            if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                storage::remove_depositor(&env, &depositor);
            }

            let token_client = token::Client::new(&env, &entry.token);
            token_client.transfer(&env.current_contract_address(), &depositor, &entry.amount);

            events::withdraw(&env, &depositor, &entry.token, entry.amount, deposit_id);
            return Ok(());
        }

        Err(VaultError::NoDepositFound)
    }

    pub fn withdraw_to(
        env: Env,
        depositor: Address,
        deposit_id: u32,
        recipient: Address,
    ) -> Result<(), VaultError> {
        depositor.require_auth();

        // Try timestamp-based deposit first
        if let Some(entry) = storage::get_deposit_readonly(&env, &depositor, deposit_id) {
            let now = env.ledger().timestamp();
            if now < entry.unlock_time {
                return Err(VaultError::FundsStillLocked);
            }

            storage::remove_deposit(&env, &depositor, deposit_id);
            if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                storage::remove_depositor(&env, &depositor);
            }

            let token_client = token::Client::new(&env, &entry.token);
            token_client.transfer(&env.current_contract_address(), &recipient, &entry.amount);

            events::withdraw_to(&env, &depositor, &recipient, &entry.token, entry.amount);
            return Ok(());
        }

        // Try ledger-based deposit
        if let Some(entry) = storage::get_deposit_by_ledger_readonly(&env, &depositor, deposit_id) {
            let current_ledger = env.ledger().sequence();
            if current_ledger < entry.unlock_ledger {
                return Err(VaultError::FundsStillLocked);
            }

            storage::remove_deposit_by_ledger(&env, &depositor, deposit_id);
            if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                storage::remove_depositor(&env, &depositor);
            }

            let token_client = token::Client::new(&env, &entry.token);
            token_client.transfer(&env.current_contract_address(), &recipient, &entry.amount);

            events::withdraw_to(&env, &depositor, &recipient, &entry.token, entry.amount);
            return Ok(());
        }

        Err(VaultError::NoDepositFound)
    }

    // ----------------------------------------------------------------
    //  Admin: Emergency Withdrawal
    // ----------------------------------------------------------------

    pub fn emergency_withdraw(
        env: Env,
        admin: Address,
        depositor: Address,
        deposit_id: u32,
    ) -> Result<(), VaultError> {
        admin.require_auth();
        storage::require_admin(&env, &admin)?;

        // Try timestamp-based deposit first
        if let Some(entry) = storage::get_deposit_readonly(&env, &depositor, deposit_id) {
            storage::remove_deposit(&env, &depositor, deposit_id);
            if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                storage::remove_depositor(&env, &depositor);
            }

            let token_client = token::Client::new(&env, &entry.token);
            token_client.transfer(&env.current_contract_address(), &depositor, &entry.amount);

            events::emergency_withdraw(&env, &admin, &depositor, &entry.token, entry.amount, deposit_id);
            return Ok(());
        }

        // Try ledger-based deposit
        if let Some(entry) = storage::get_deposit_by_ledger_readonly(&env, &depositor, deposit_id) {
            storage::remove_deposit_by_ledger(&env, &depositor, deposit_id);
            if storage::get_deposit_ids(&env, &depositor).len() == 0 {
                storage::remove_depositor(&env, &depositor);
            }

            let token_client = token::Client::new(&env, &entry.token);
            token_client.transfer(&env.current_contract_address(), &depositor, &entry.amount);

            events::emergency_withdraw(&env, &admin, &depositor, &entry.token, entry.amount, deposit_id);
            return Ok(());
        }

        Err(VaultError::NoDepositFound)
    }

    // ----------------------------------------------------------------
    //  Admin: Pause / Unpause
    // ----------------------------------------------------------------

    pub fn pause(env: Env, admin: Address) -> Result<(), VaultError> {
        admin.require_auth();
        let stored_admin = storage::get_admin(&env).ok_or(VaultError::Unauthorized)?;
        if admin != stored_admin {
            return Err(VaultError::Unauthorized);
        }
        storage::set_paused(&env, true);
        events::paused(&env, &admin);
        Ok(())
    }

    pub fn unpause(env: Env, admin: Address) -> Result<(), VaultError> {
        admin.require_auth();
        let stored_admin = storage::get_admin(&env).ok_or(VaultError::Unauthorized)?;
        if admin != stored_admin {
            return Err(VaultError::Unauthorized);
        }
        storage::set_paused(&env, false);
        events::unpaused(&env, &admin);
        Ok(())
    }

    pub fn is_paused(env: Env) -> bool {
        storage::is_paused(&env)
    }

    // ----------------------------------------------------------------
    //  Admin: Two-Step Admin Transfer
    // ----------------------------------------------------------------

    pub fn transfer_admin(env: Env, admin: Address, new_admin: Address) -> Result<(), VaultError> {
        admin.require_auth();
        let stored_admin = storage::get_admin(&env).ok_or(VaultError::Unauthorized)?;
        if admin != stored_admin {
            return Err(VaultError::Unauthorized);
        }

        if new_admin == stored_admin {
            return Err(VaultError::InvalidAdmin);
        }

        storage::set_pending_admin(&env, &new_admin);
        events::admin_transfer_initiated(&env, &admin, &new_admin);
        Ok(())
    }

    pub fn accept_admin(env: Env, new_admin: Address) -> Result<(), VaultError> {
        new_admin.require_auth();

        let pending_admin = storage::get_pending_admin(&env).ok_or(VaultError::Unauthorized)?;
        if new_admin != pending_admin {
            return Err(VaultError::Unauthorized);
        }

        storage::set_admin(&env, &new_admin);
        storage::remove_pending_admin(&env);
        events::admin_transfer_accepted(&env, &new_admin);
        Ok(())
    }

    pub fn cancel_transfer_admin(env: Env, admin: Address) -> Result<(), VaultError> {
        admin.require_auth();

        let stored_admin = storage::get_admin(&env).ok_or(VaultError::Unauthorized)?;
        if admin != stored_admin {
            return Err(VaultError::Unauthorized);
        }

        // Emit an event when a pending admin is cancelled so off-chain indexers
        // and UIs observing admin state transitions won't show a stale pending admin.
        if let Some(pending) = storage::get_pending_admin(&env) {
            storage::remove_pending_admin(&env);
            events::admin_transfer_cancelled(&env, &admin, &pending);
        }
        Ok(())
    }

    pub fn renounce_admin(env: Env, admin: Address) -> Result<(), VaultError> {
        admin.require_auth();

        let stored_admin = storage::get_admin(&env).ok_or(VaultError::Unauthorized)?;
        if admin != stored_admin {
            return Err(VaultError::Unauthorized);
        }

        storage::remove_admin(&env);
        storage::remove_pending_admin(&env);
        events::admin_renounced(&env, &admin);
        Ok(())
    }

    // ----------------------------------------------------------------
    //  Read-only Queries
    // ----------------------------------------------------------------

    /// No auth required — this is a public read-only query (closes #81)
    pub fn get_vault(env: Env, depositor: Address, deposit_id: u32) -> Option<VaultEntry> {
        storage::get_deposit_readonly(&env, &depositor, deposit_id)
    }

    /// Returns the `LedgerVaultEntry` for a ledger-sequence-based deposit, or `None` if not found.
    /// No auth required — public read-only query (closes #44).
    pub fn get_ledger_vault(env: Env, depositor: Address, deposit_id: u32) -> Option<LedgerVaultEntry> {
        storage::get_deposit_by_ledger_readonly(&env, &depositor, deposit_id)
    }

    pub fn get_vault_batch(env: Env, depositors: Vec<Address>, deposit_id: u32) -> Vec<Option<VaultEntry>> {
        let limit = if depositors.len() > MAX_BATCH_SIZE { MAX_BATCH_SIZE } else { depositors.len() as u32 };
        let mut results = Vec::new(&env);
        for i in 0..limit {
            if let Some(depositor) = depositors.get(i) {
                let entry = storage::get_deposit_readonly(&env, &depositor, deposit_id);
                results.push_back(entry);
            }
        }
        results
    }

    pub fn get_deposit_ids(env: Env, depositor: Address) -> Vec<u32> {
        storage::get_deposit_ids(&env, &depositor)
    }

    /// Returns the current ledger timestamp.
    /// Read-only — does not bump storage TTL.
    pub fn get_time(env: Env) -> u64 {
        env.ledger().timestamp()
    }

    /// No auth required — this is a public read-only query (closes #81)
    ///
    /// For timestamp-based deposits: returns exact seconds remaining.
    /// For ledger-based deposits: returns an estimate in seconds using
    /// `LEDGER_SECONDS` (fixes #21). Returns 0 when unlocked or not found.
    pub fn time_remaining(env: Env, depositor: Address, deposit_id: u32) -> u64 {
        // Timestamp-based path
        if let Some(entry) = storage::get_deposit_readonly(&env, &depositor, deposit_id) {
            let now = env.ledger().timestamp();
            return entry.unlock_time.saturating_sub(now);
        }

        // Ledger-based path: convert remaining ledgers → estimated seconds (fixes #21)
        if let Some(entry) = storage::get_deposit_by_ledger_readonly(&env, &depositor, deposit_id) {
            let current = env.ledger().sequence();
            if current >= entry.unlock_ledger {
                return 0;
            }
            let remaining_ledgers = (entry.unlock_ledger - current) as u64;
            return remaining_ledgers.saturating_mul(storage::LEDGER_SECONDS);
        }

        0
    }

    pub fn get_admin(env: Env) -> Option<Address> {
        storage::get_admin(&env)
    }

    pub fn get_pending_admin(env: Env) -> Option<Address> {
        storage::get_pending_admin(&env)
    }

    pub fn get_constants(env: Env) -> (i128, u64) {
        let max_deposit = storage::get_max_deposit(&env).unwrap_or(MAX_DEPOSIT_AMOUNT);
        let max_lock = storage::get_max_lock_secs(&env).unwrap_or(MAX_LOCK_DURATION_SECS);
        (max_deposit, max_lock)
    }

    pub fn get_fee_recipient(env: Env) -> Option<Address> {
        storage::get_fee_recipient(&env)
    }

    pub fn get_depositor_count(env: Env) -> u32 {
        storage::get_depositor_count(&env)
    }

    pub fn get_depositors(env: Env, offset: u32, limit: u32) -> Vec<Address> {
        storage::get_depositors_page(&env, offset, limit)
    }

    pub fn is_initialized(env: Env) -> bool {
        storage::is_initialized(&env)
    }

    // ----------------------------------------------------------------
    //  Privacy: Opt-in
    // ----------------------------------------------------------------

    /// Permanently enables privacy mode for `depositor`. Once enabled it cannot
    /// be disabled — future deposits made by this address can use
    /// `private_deposit()` instead of (or in addition to) `deposit()`.
    pub fn enable_privacy(env: Env, depositor: Address) -> Result<(), VaultError> {
        depositor.require_auth();
        storage::set_privacy_enabled(&env, &depositor);
        events::privacy_enabled(&env, &depositor);
        Ok(())
    }

    /// Returns `true` if `depositor` has opted into privacy mode.
    pub fn is_privacy_enabled(env: Env, depositor: Address) -> bool {
        storage::is_privacy_enabled(&env, &depositor)
    }

    /// Allow or revoke a compliance auditor. The auditor must still receive
    /// the private preimage from the depositor before an audit can succeed.
    pub fn set_auditor(
        env: Env,
        admin: Address,
        auditor: Address,
        enabled: bool,
    ) -> Result<(), VaultError> {
        admin.require_auth();
        storage::require_admin(&env, &admin)?;
        storage::set_auditor(&env, &auditor, enabled);
        Ok(())
    }

    pub fn is_auditor(env: Env, auditor: Address) -> bool {
        storage::is_auditor(&env, &auditor)
    }

    // ----------------------------------------------------------------
    //  Privacy: Private Deposit
    // ----------------------------------------------------------------

    /// Create a private deposit. The caller must have called `enable_privacy()` first.
    ///
    /// # Arguments
    /// * `depositor`  – The account funding and owning this vault.
    /// * `token`      – SAC token address.
    /// * `amount`     – Token amount (transferred from `depositor`).
    /// * `unlock_time`– Lock expiry (seconds since Unix epoch).
    /// * `penalty_bps`– Early-exit penalty in basis points (0–10 000).
    /// * `commitment` – 32-byte SHA-256 of `secret || token_address || amount || salt`.
    ///                  Computed off-chain by the depositor.
    ///
    /// The `commitment` must be exactly 32 bytes. Neither the token address nor
    /// the amount are stored in the on-chain entry — they are hidden inside the
    /// commitment. The caller is responsible for keeping the preimage secret.
    ///
    /// # Commitment Scheme
    /// ```text
    /// commitment = SHA-256(secret[32] || token_bytes[32] || amount_bytes[16] || salt[32])
    /// nullifier  = SHA-256(secret[32] || deposit_id[4])
    /// ```
    pub fn private_deposit(
        env: Env,
        depositor: Address,
        token: Address,
        amount: i128,
        unlock_time: u64,
        penalty_bps: u32,
        commitment: Bytes,
    ) -> Result<u32, VaultError> {
        depositor.require_auth();

        if storage::is_paused(&env) {
            return Err(VaultError::ContractPaused);
        }

        if !storage::is_privacy_enabled(&env, &depositor) {
            return Err(VaultError::PrivacyNotEnabled);
        }

        // Commitment must be exactly 32 bytes (SHA-256 output).
        if commitment.len() != 32 {
            return Err(VaultError::InvalidCommitmentLength);
        }

        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }

        let max_deposit = storage::get_max_deposit(&env).unwrap_or(MAX_DEPOSIT_AMOUNT);
        if amount > max_deposit {
            return Err(VaultError::AmountTooLarge);
        }

        if penalty_bps > 10_000 {
            return Err(VaultError::InvalidPenaltyBps);
        }

        if penalty_bps > 0 && storage::get_fee_recipient(&env).is_none() {
            return Err(VaultError::MissingFeeRecipient);
        }

        let now = env.ledger().timestamp();
        if unlock_time <= now {
            return Err(VaultError::UnlockTimeNotInFuture);
        }

        let max_lock = storage::get_max_lock_secs(&env).unwrap_or(MAX_LOCK_DURATION_SECS);
        let lock_duration: u64 = unlock_time.saturating_sub(now);
        if lock_duration > max_lock {
            return Err(VaultError::LockDurationTooLong);
        }
        if lock_duration < MIN_LOCK_DURATION_SECS {
            return Err(VaultError::LockDurationTooShort);
        }

        let deposit_id = storage::next_deposit_id(&env, &depositor);

        // Transfer tokens first — checks-effects-interactions:
        // we store state before the transfer below.
        let entry = PrivateVaultEntry {
            commitment: commitment.clone(),
            unlock_time,
            penalty_bps,
        };

        // Effects: store before external call.
        storage::set_private_deposit(&env, &depositor, deposit_id, &entry);
        storage::add_depositor(&env, &depositor);

        // Interaction: token transfer.
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&depositor, &env.current_contract_address(), &amount);

        events::private_deposit(&env, &depositor, &commitment, unlock_time, deposit_id);

        Ok(deposit_id)
    }

    // ----------------------------------------------------------------
    //  Privacy: Private Withdrawal
    // ----------------------------------------------------------------

    /// Withdraw a private deposit by revealing the commitment preimage.
    ///
    /// The caller proves ownership of the locked funds by supplying:
    /// * `secret`  – 32-byte random secret chosen at deposit time.
    /// * `token`   – The token address that was deposited.
    /// * `amount`  – The amount that was deposited.
    /// * `salt`    – 32-byte random salt chosen at deposit time.
    ///
    /// The contract re-derives:
    /// * `commitment = SHA-256(secret || token || amount || salt)` — compared with stored value.
    /// * `nullifier  = SHA-256(secret || deposit_id)`              — checked for double-spend.
    ///
    /// If both checks pass the funds are returned to `depositor`.
    pub fn private_withdraw(
        env: Env,
        depositor: Address,
        deposit_id: u32,
        secret: Bytes,
        token: Address,
        amount: i128,
        salt: Bytes,
    ) -> Result<(), VaultError> {
        depositor.require_auth();

        let entry = storage::get_private_deposit(&env, &depositor, deposit_id)
            .ok_or(VaultError::NoDepositFound)?;

        let now = env.ledger().timestamp();
        if now < entry.unlock_time {
            return Err(VaultError::FundsStillLocked);
        }

        // ---- Verify commitment ----
        let expected_commitment = Self::compute_commitment(&env, &secret, &token, amount, &salt);
        if expected_commitment != entry.commitment {
            return Err(VaultError::InvalidCommitment);
        }

        // ---- Verify & spend nullifier ----
        let nullifier = Self::compute_nullifier(&env, &secret, deposit_id);
        if storage::is_nullifier_used(&env, &nullifier) {
            return Err(VaultError::NullifierAlreadyUsed);
        }

        // Effects: clear state before external call.
        storage::remove_private_deposit(&env, &depositor, deposit_id);
        storage::spend_nullifier(&env, &nullifier);
        if storage::get_deposit_ids(&env, &depositor).len() == 0 {
            storage::remove_depositor(&env, &depositor);
        }

        // Interaction: transfer tokens.
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &depositor, &amount);

        events::private_withdraw(&env, &nullifier, deposit_id);

        Ok(())
    }

    // ----------------------------------------------------------------
    //  Privacy: Audit
    // ----------------------------------------------------------------

    /// Read-only audit query. Returns `true` if the supplied preimage matches
    /// the stored commitment for `(depositor, deposit_id)`.
    ///
    /// Only an auditor explicitly authorized by the admin can call this. The
    /// depositor must separately disclose the preimage to that auditor.
    pub fn audit_private_deposit(
        env: Env,
        auditor: Address,
        depositor: Address,
        deposit_id: u32,
        secret: Bytes,
        token: Address,
        amount: i128,
        salt: Bytes,
    ) -> Result<bool, VaultError> {
        auditor.require_auth();
        if !storage::is_auditor(&env, &auditor) {
            return Err(VaultError::AuditorUnauthorized);
        }
        let entry = match storage::get_private_deposit(&env, &depositor, deposit_id) {
            Some(e) => e,
            None => return Ok(false),
        };
        let expected = Self::compute_commitment(&env, &secret, &token, amount, &salt);
        Ok(expected == entry.commitment)
    }

    /// Return a private deposit amount only after verifying its commitment.
    /// The depositor's authorization prevents an arbitrary public query from
    /// turning a valid preimage into a balance disclosure.
    pub fn private_balance(
        env: Env,
        depositor: Address,
        proof: PrivateBalanceProof,
    ) -> Result<i128, VaultError> {
        depositor.require_auth();
        Self::verify_private_proof(&env, &depositor, &proof)?;
        Ok(proof.amount)
    }

    /// Returns the `PrivateVaultEntry` for a private deposit, or `None` if not found.
    /// The entry only contains the commitment, unlock_time, and penalty_bps — the
    /// amount and token address are NOT exposed.
    pub fn get_private_vault(
        env: Env,
        depositor: Address,
        deposit_id: u32,
    ) -> Option<PrivateVaultEntry> {
        storage::get_private_deposit(&env, &depositor, deposit_id)
    }

    /// Returns `true` if a nullifier has already been spent.
    /// Useful for off-chain indexers to confirm withdrawal finality.
    pub fn is_nullifier_spent(env: Env, nullifier: Bytes) -> bool {
        storage::is_nullifier_used(&env, &nullifier)
    }

    // ----------------------------------------------------------------
    //  Privacy: Commitment helpers (private — used internally)
    // ----------------------------------------------------------------

    /// Compute `SHA-256(secret[32] || token_bytes[32] || amount_bytes[16] || salt[32])`.
    ///
    /// The token Address is serialized via its raw 32-byte Stellar key. The amount
    /// is serialized as a 16-byte big-endian i128.
    fn compute_commitment(env: &Env, secret: &Bytes, token: &Address, amount: i128, salt: &Bytes) -> Bytes {
        let mut preimage = Bytes::new(env);
        preimage.append(secret);
        // Encode token as 32 bytes via Soroban's XDR-compatible bytes representation.
        let token_bytes = token.to_xdr(env);
        preimage.append(&token_bytes);
        // Encode amount as 16-byte big-endian.
        let amount_bytes = Self::i128_to_bytes(env, amount);
        preimage.append(&amount_bytes);
        preimage.append(salt);
        env.crypto().sha256(&preimage).into()
    }

    /// Compute `SHA-256(secret[32] || deposit_id[4])`.
    fn compute_nullifier(env: &Env, secret: &Bytes, deposit_id: u32) -> Bytes {
        let mut preimage = Bytes::new(env);
        preimage.append(secret);
        let id_bytes = Self::u32_to_bytes(env, deposit_id);
        preimage.append(&id_bytes);
        env.crypto().sha256(&preimage).into()
    }

    fn verify_private_proof(
        env: &Env,
        depositor: &Address,
        proof: &PrivateBalanceProof,
    ) -> Result<(), VaultError> {
        let entry = storage::get_private_deposit(env, depositor, proof.deposit_id)
            .ok_or(VaultError::NoDepositFound)?;
        let expected = Self::compute_commitment(
            env,
            &proof.secret,
            &proof.token,
            proof.amount,
            &proof.salt,
        );
        if expected != entry.commitment {
            return Err(VaultError::InvalidCommitment);
        }
        Ok(())
    }

    /// Serialize `i128` as 16 big-endian bytes.
    fn i128_to_bytes(env: &Env, v: i128) -> Bytes {
        let raw = v.to_be_bytes();
        Bytes::from_array(env, &raw)
    }

    /// Serialize `u32` as 4 big-endian bytes.
    fn u32_to_bytes(env: &Env, v: u32) -> Bytes {
        let raw = v.to_be_bytes();
        Bytes::from_array(env, &raw)
    }
}
