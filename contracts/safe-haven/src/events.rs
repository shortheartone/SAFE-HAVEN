use soroban_sdk::{symbol_short, Address, Env, Symbol, Vec};

pub fn contract_initialized(
    env: &Env,
    admin: &Address,
    fee_recipient: &Address,
    max_deposit: i128,
    max_lock_secs: u64,
) {
    let topics = (Symbol::new(env, "initialized"),);
    env.events().publish(
        topics,
        (
            admin.clone(),
            fee_recipient.clone(),
            max_deposit,
            max_lock_secs,
        ),
    );
}

pub fn deposit(
    env: &Env,
    depositor: &Address,
    token: &Address,
    amount: i128,
    unlock_time: u64,
    deposit_id: u32,
) {
    let topics = (symbol_short!("deposit"), depositor.clone(), token.clone());
    env.events()
        .publish(topics, (amount, unlock_time, deposit_id));
}

pub fn deposit_by_ledger(
    env: &Env,
    depositor: &Address,
    token: &Address,
    amount: i128,
    unlock_ledger: u32,
    deposit_id: u32,
) {
    let topics = (
        Symbol::new(env, "dep_by_ledger"),
        depositor.clone(),
        token.clone(),
    );
    env.events()
        .publish(topics, (amount, unlock_ledger, deposit_id));
}

/// Emitted when a multi-token deposit is created (issue #330).
/// `token_count` is the number of distinct tokens in the vault.
pub fn multi_deposit(
    env: &Env,
    depositor: &Address,
    token_count: u32,
    unlock_time: u64,
    deposit_id: u32,
) {
    let topics = (Symbol::new(env, "multi_deposit"), depositor.clone());
    env.events()
        .publish(topics, (token_count, unlock_time, deposit_id));
}

pub fn withdraw(env: &Env, depositor: &Address, token: &Address, amount: i128, deposit_id: u32) {
    let topics = (symbol_short!("withdraw"), depositor.clone(), token.clone());
    env.events().publish(topics, (amount, deposit_id));
}

/// Emitted when a multi-token deposit is withdrawn (issue #330).
pub fn multi_withdraw(env: &Env, depositor: &Address, recipient: &Address, deposit_id: u32, token_count: u32) {
    let topics = (Symbol::new(env, "multi_wdraw"), depositor.clone());
    env.events().publish(topics, (recipient.clone(), deposit_id, token_count));
}

pub fn emergency_withdraw(
    env: &Env,
    admin: &Address,
    depositor: &Address,
    token: &Address,
    amount: i128,
    deposit_id: u32,
) {
    let topics = (Symbol::new(env, "emrg_wdraw"), depositor.clone());
    env.events()
        .publish(topics, (admin.clone(), token.clone(), amount, deposit_id));
}

pub fn admin_transfer_initiated(env: &Env, current_admin: &Address, pending_admin: &Address) {
    let topics = (Symbol::new(env, "adm_xfr_init"), current_admin.clone());
    env.events().publish(topics, pending_admin.clone());
}

pub fn admin_transfer_cancelled(env: &Env, current_admin: &Address, pending_admin: &Address) {
    let topics = (Symbol::new(env, "adm_xfr_cancel"), current_admin.clone());
    env.events().publish(topics, pending_admin.clone());
}

pub fn admin_transfer_accepted(env: &Env, new_admin: &Address) {
    let topics = (Symbol::new(env, "adm_xfr_done"), new_admin.clone());
    env.events().publish(topics, ());
}

pub fn admin_renounced(env: &Env, former_admin: &Address) {
    let topics = (Symbol::new(env, "adm_renounce"), former_admin.clone());
    env.events().publish(topics, ());
}

pub fn lock_extended(env: &Env, depositor: &Address, old_unlock_time: u64, new_unlock_time: u64) {
    let topics = (Symbol::new(env, "lock_extended"), depositor.clone());
    env.events()
        .publish(topics, (old_unlock_time, new_unlock_time));
}

pub fn deposit_cancelled(
    env: &Env,
    depositor: &Address,
    token: &Address,
    amount: i128,
    penalty: i128,
    deposit_id: u32,
) {
    let topics = (
        Symbol::new(env, "dep_cancel"),
        depositor.clone(),
        token.clone(),
    );
    env.events().publish(topics, (amount, penalty, deposit_id));
}

pub fn paused(env: &Env, admin: &Address) {
    let topics = (Symbol::new(env, "paused"), admin.clone());
    env.events().publish(topics, ());
}

pub fn unpaused(env: &Env, admin: &Address) {
    let topics = (Symbol::new(env, "unpaused"), admin.clone());
    env.events().publish(topics, ());
}

pub fn withdraw_to(
    env: &Env,
    depositor: &Address,
    recipient: &Address,
    token: &Address,
    amount: i128,
) {
    let topics = (
        Symbol::new(env, "withdraw_to"),
        depositor.clone(),
        token.clone(),
    );
    env.events().publish(topics, (recipient.clone(), amount));
}

/// Emitted when the withdrawal whitelist is set for a deposit (issue #331).
pub fn whitelist_set(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
    whitelist: &Vec<Address>,
) {
    let topics = (Symbol::new(env, "wl_set"), depositor.clone());
    env.events().publish(topics, (deposit_id, whitelist.len()));
}

/// Emitted when compound interest is accrued (issue #332).
pub fn interest_accrued(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
    old_amount: i128,
    new_amount: i128,
) {
    let topics = (Symbol::new(env, "interest"), depositor.clone());
    env.events()
        .publish(topics, (deposit_id, old_amount, new_amount));
}


/// Emitted when auto-renewal is enabled for a deposit.
pub fn auto_renewal_enabled(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
    renewal_duration_secs: u64,
) {
    let topics = (Symbol::new(env, "auto_rnw_en"), depositor.clone());
    env.events()
        .publish(topics, (deposit_id, renewal_duration_secs));
}

/// Emitted when auto-renewal is disabled for a deposit.
pub fn auto_renewal_disabled(env: &Env, depositor: &Address, deposit_id: u32) {
    let topics = (Symbol::new(env, "auto_rnw_dis"), depositor.clone());
    env.events().publish(topics, deposit_id);
}

/// Emitted when a deposit is automatically renewed.
pub fn deposit_auto_renewed(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
    old_unlock_time: u64,
    new_unlock_time: u64,
    renewal_count: u32,
) {
    let topics = (Symbol::new(env, "auto_rnw"), depositor.clone());
    env.events()
        .publish(topics, (deposit_id, old_unlock_time, new_unlock_time, renewal_count));
}
