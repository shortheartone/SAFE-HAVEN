use soroban_sdk::{Address, Env, Vec};

use crate::types::{VaultEntry, VaultKey, LedgerVaultEntry, MAX_LOCK_DURATION_SECS};

// Number of seconds per ledger — Soroban ledgers are ~5 seconds apart.
pub const LEDGER_SECONDS: u64 = 5;

// How many ledgers to extend TTL to cover the maximum allowed lock duration.
pub const BUMP_THRESHOLD: u32 = 518_400;
pub const BUMP_TARGET: u32 = ((MAX_LOCK_DURATION_SECS + LEDGER_SECONDS - 1) / LEDGER_SECONDS) as u32;

// ----------------------------------------------------------------
//  Deposit counter helpers
// ----------------------------------------------------------------

pub fn next_deposit_id(env: &Env, depositor: &Address) -> u32 {
    let key = VaultKey::DepositCounter(depositor.clone());
    let id: u32 = env.storage().persistent().get(&key).unwrap_or(0);
    env.storage().persistent().set(&key, &(id + 1));
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
    id
}

// ----------------------------------------------------------------
//  Active deposit ID list helpers (fixes #18 and #20)
//
//  A `Vec<u32>` stored under `ActiveDepositIds(depositor)` is the
//  authoritative list of IDs that currently have either a timestamp-
//  based or a ledger-based deposit entry. Maintained in O(1) on push
//  and O(n-active) on removal (n-active is bounded by actual open
//  deposits, not historical counter value).
// ----------------------------------------------------------------

fn get_active_ids(env: &Env, depositor: &Address) -> Vec<u32> {
    let key = VaultKey::ActiveDepositIds(depositor.clone());
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env))
}

fn save_active_ids(env: &Env, depositor: &Address, ids: &Vec<u32>) {
    let key = VaultKey::ActiveDepositIds(depositor.clone());
    env.storage().persistent().set(&key, ids);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

/// Append `deposit_id` to the active ID list for `depositor`. Called
/// immediately after a new deposit entry is written to storage.
pub fn add_active_deposit_id(env: &Env, depositor: &Address, deposit_id: u32) {
    let mut ids = get_active_ids(env, depositor);
    ids.push_back(deposit_id);
    save_active_ids(env, depositor, &ids);
}

/// Remove `deposit_id` from the active ID list for `depositor`. Called
/// immediately after a deposit entry is removed from storage.
pub fn remove_active_deposit_id(env: &Env, depositor: &Address, deposit_id: u32) {
    let ids = get_active_ids(env, depositor);
    let mut new_ids: Vec<u32> = Vec::new(env);
    for id in ids.iter() {
        if id != deposit_id {
            new_ids.push_back(id);
        }
    }
    save_active_ids(env, depositor, &new_ids);
}

/// O(1) single storage read — returns all active deposit IDs for
/// `depositor`, regardless of whether they are timestamp- or
/// ledger-based (fixes #18 and #20).
pub fn get_deposit_ids(env: &Env, depositor: &Address) -> Vec<u32> {
    get_active_ids(env, depositor)
}

// ----------------------------------------------------------------
//  Deposit helpers
// ----------------------------------------------------------------

pub fn set_deposit(env: &Env, depositor: &Address, deposit_id: u32, entry: &VaultEntry) {
    let key = VaultKey::Deposit(depositor.clone(), deposit_id);
    env.storage().persistent().set(&key, entry);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
    add_active_deposit_id(env, depositor, deposit_id);
}

pub fn get_deposit(env: &Env, depositor: &Address, deposit_id: u32) -> Option<VaultEntry> {
    let key = VaultKey::Deposit(depositor.clone(), deposit_id);
    let entry: Option<VaultEntry> = env.storage().persistent().get(&key);
    if entry.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
    }
    entry
}

pub fn get_deposit_readonly(env: &Env, depositor: &Address, deposit_id: u32) -> Option<VaultEntry> {
    let key = VaultKey::Deposit(depositor.clone(), deposit_id);
    env.storage().persistent().get(&key)
}

pub fn remove_deposit(env: &Env, depositor: &Address, deposit_id: u32) {
    let key = VaultKey::Deposit(depositor.clone(), deposit_id);
    env.storage().persistent().remove(&key);
    remove_active_deposit_id(env, depositor, deposit_id);
}

// ----------------------------------------------------------------
//  Ledger-based deposit helpers
// ----------------------------------------------------------------

pub fn set_deposit_by_ledger(env: &Env, depositor: &Address, deposit_id: u32, entry: &LedgerVaultEntry) {
    let key = VaultKey::DepositByLedger(depositor.clone(), deposit_id);
    env.storage().persistent().set(&key, entry);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
    add_active_deposit_id(env, depositor, deposit_id);
}

pub fn get_deposit_by_ledger_readonly(env: &Env, depositor: &Address, deposit_id: u32) -> Option<LedgerVaultEntry> {
    let key = VaultKey::DepositByLedger(depositor.clone(), deposit_id);
    env.storage().persistent().get(&key)
}

pub fn remove_deposit_by_ledger(env: &Env, depositor: &Address, deposit_id: u32) {
    let key = VaultKey::DepositByLedger(depositor.clone(), deposit_id);
    env.storage().persistent().remove(&key);
    remove_active_deposit_id(env, depositor, deposit_id);
}

// ----------------------------------------------------------------
//  Admin helpers
// ----------------------------------------------------------------

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().persistent().set(&VaultKey::Admin, admin);
    env.storage()
        .persistent()
        .extend_ttl(&VaultKey::Admin, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&VaultKey::Admin)
}

pub fn remove_admin(env: &Env) {
    env.storage().persistent().remove(&VaultKey::Admin);
}

pub fn set_pending_admin(env: &Env, pending: &Address) {
    env.storage()
        .persistent()
        .set(&VaultKey::PendingAdmin, pending);
    env.storage()
        .persistent()
        .extend_ttl(&VaultKey::PendingAdmin, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn get_pending_admin(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&VaultKey::PendingAdmin)
}

pub fn remove_pending_admin(env: &Env) {
    env.storage().persistent().remove(&VaultKey::PendingAdmin);
}

// ----------------------------------------------------------------
//  Initialized flag
// ----------------------------------------------------------------

pub fn set_initialized(env: &Env) {
    env.storage()
        .persistent()
        .set(&VaultKey::Initialized, &true);
    env.storage()
        .persistent()
        .extend_ttl(&VaultKey::Initialized, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn is_initialized(env: &Env) -> bool {
    env.storage()
        .persistent()
        .get::<VaultKey, bool>(&VaultKey::Initialized)
        .unwrap_or(false)
}

// ----------------------------------------------------------------
//  Runtime limits helpers
// ----------------------------------------------------------------

pub fn set_max_deposit(env: &Env, v: i128) {
    env.storage().persistent().set(&VaultKey::MaxDeposit, &v);
    env.storage()
        .persistent()
        .extend_ttl(&VaultKey::MaxDeposit, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn get_max_deposit(env: &Env) -> Option<i128> {
    env.storage().persistent().get(&VaultKey::MaxDeposit)
}

pub fn set_max_lock_secs(env: &Env, v: u64) {
    env.storage().persistent().set(&VaultKey::MaxLockSecs, &v);
    env.storage()
        .persistent()
        .extend_ttl(&VaultKey::MaxLockSecs, BUMP_THRESHOLD, BUMP_TARGET);
}

/// Returns the runtime-configured max lock duration, or `None` to use the compile-time default.
pub fn get_max_lock_secs(env: &Env) -> Option<u64> {
    env.storage().persistent().get(&VaultKey::MaxLockSecs)
}

// ----------------------------------------------------------------
//  Fee recipient helpers
// ----------------------------------------------------------------

/// Persists the `fee_recipient` address and bumps TTL. Called once during `initialize`.
pub fn set_fee_recipient(env: &Env, recipient: &Address) {
    env.storage()
        .persistent()
        .set(&VaultKey::FeeRecipient, recipient);
    env.storage()
        .persistent()
        .extend_ttl(&VaultKey::FeeRecipient, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn get_fee_recipient(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&VaultKey::FeeRecipient)
}

// ----------------------------------------------------------------
//  Depositor list helpers
// ----------------------------------------------------------------

fn get_depositor_list(env: &Env) -> Vec<Address> {
    env.storage()
        .persistent()
        .get(&VaultKey::DepositorList)
        .unwrap_or_else(|| Vec::new(env))
}

fn save_depositor_list(env: &Env, list: &Vec<Address>) {
    env.storage()
        .persistent()
        .set(&VaultKey::DepositorList, list);
    env.storage()
        .persistent()
        .extend_ttl(&VaultKey::DepositorList, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn add_depositor(env: &Env, depositor: &Address) {
    // O(1) existence check via a per-depositor boolean flag (fixes #19).
    let flag_key = VaultKey::DepositorFlag(depositor.clone());
    if env
        .storage()
        .persistent()
        .get::<VaultKey, bool>(&flag_key)
        .unwrap_or(false)
    {
        return; // already registered
    }
    // Mark as registered before touching the list.
    env.storage().persistent().set(&flag_key, &true);
    env.storage()
        .persistent()
        .extend_ttl(&flag_key, BUMP_THRESHOLD, BUMP_TARGET);

    let mut list = get_depositor_list(env);
    list.push_back(depositor.clone());
    save_depositor_list(env, &list);
}

pub fn remove_depositor(env: &Env, depositor: &Address) {
    // Clear the O(1) flag so a future re-deposit is correctly re-registered.
    let flag_key = VaultKey::DepositorFlag(depositor.clone());
    env.storage().persistent().remove(&flag_key);

    let list = get_depositor_list(env);
    let mut new_list: Vec<Address> = Vec::new(env);
    for addr in list.iter() {
        if &addr != depositor {
            new_list.push_back(addr);
        }
    }
    save_depositor_list(env, &new_list);
}

pub fn get_depositor_count(env: &Env) -> u32 {
    get_depositor_list(env).len()
}

// ----------------------------------------------------------------
//  Paused flag helpers
// ----------------------------------------------------------------

pub fn set_paused(env: &Env, paused: bool) {
    env.storage().persistent().set(&VaultKey::Paused, &paused);
    env.storage()
        .persistent()
        .extend_ttl(&VaultKey::Paused, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn is_paused(env: &Env) -> bool {
    env.storage()
        .persistent()
        .get::<VaultKey, bool>(&VaultKey::Paused)
        .unwrap_or(false)
}

pub fn get_depositors_page(env: &Env, offset: u32, limit: u32) -> Vec<Address> {
    let list = get_depositor_list(env);
    let len = list.len();
    let mut page: Vec<Address> = Vec::new(env);
    let end = (offset + limit).min(len);
    for i in offset..end {
        page.push_back(list.get(i).unwrap());
    }
    page
}

// ----------------------------------------------------------------
//  Admin authorization check helper
// ----------------------------------------------------------------

/// Returns `Ok(())` if `caller` matches the stored admin, else `Err(VaultError::Unauthorized)`.
pub fn require_admin(env: &Env, caller: &Address) -> Result<(), crate::errors::VaultError> {
    match get_admin(env) {
        Some(ref stored) if stored == caller => Ok(()),
        _ => Err(crate::errors::VaultError::Unauthorized),
    }
}

// ----------------------------------------------------------------
//  Privacy storage helpers
// ----------------------------------------------------------------

use crate::types::PrivateVaultEntry;

/// Enable privacy mode for `depositor`.
pub fn set_privacy_enabled(env: &Env, depositor: &Address) {
    let key = VaultKey::PrivacyEnabled(depositor.clone());
    env.storage().persistent().set(&key, &true);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

/// Returns `true` if `depositor` has opted in to privacy mode.
pub fn is_privacy_enabled(env: &Env, depositor: &Address) -> bool {
    let key = VaultKey::PrivacyEnabled(depositor.clone());
    env.storage()
        .persistent()
        .get::<VaultKey, bool>(&key)
        .unwrap_or(false)
}

/// Store a `PrivateVaultEntry`. Uses the same deposit-ID counter as public deposits.
pub fn set_private_deposit(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
    entry: &PrivateVaultEntry,
) {
    let key = VaultKey::PrivateDeposit(depositor.clone(), deposit_id);
    env.storage().persistent().set(&key, entry);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
    // Register in the active-ID list so `get_deposit_ids` includes private deposits.
    add_active_deposit_id(env, depositor, deposit_id);
}

/// Read a `PrivateVaultEntry` without bumping TTL (for read-only queries).
pub fn get_private_deposit(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
) -> Option<PrivateVaultEntry> {
    let key = VaultKey::PrivateDeposit(depositor.clone(), deposit_id);
    let entry: Option<PrivateVaultEntry> = env.storage().persistent().get(&key);
    if entry.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
    }
    entry
}

/// Remove a `PrivateVaultEntry` (called after a successful withdrawal).
pub fn remove_private_deposit(env: &Env, depositor: &Address, deposit_id: u32) {
    let key = VaultKey::PrivateDeposit(depositor.clone(), deposit_id);
    env.storage().persistent().remove(&key);
    remove_active_deposit_id(env, depositor, deposit_id);
}

/// Returns `true` if this nullifier has been spent.
pub fn is_nullifier_used(env: &Env, nullifier: &soroban_sdk::Bytes) -> bool {
    let key = VaultKey::Nullifier(nullifier.clone());
    env.storage()
        .persistent()
        .get::<VaultKey, bool>(&key)
        .unwrap_or(false)
}

/// Mark a nullifier as spent so it cannot be reused.
pub fn spend_nullifier(env: &Env, nullifier: &soroban_sdk::Bytes) {
    let key = VaultKey::Nullifier(nullifier.clone());
    env.storage().persistent().set(&key, &true);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn set_auditor(env: &Env, auditor: &Address, enabled: bool) {
    let key = VaultKey::Auditor(auditor.clone());
    env.storage().persistent().set(&key, &enabled);
    env.storage()
        .persistent()
        .extend_ttl(&key, BUMP_THRESHOLD, BUMP_TARGET);
}

pub fn is_auditor(env: &Env, auditor: &Address) -> bool {
    let key = VaultKey::Auditor(auditor.clone());
    env.storage()
        .persistent()
        .get::<VaultKey, bool>(&key)
        .unwrap_or(false)
}
