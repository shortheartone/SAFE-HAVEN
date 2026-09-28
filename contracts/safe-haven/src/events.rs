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

pub fn tax_loss_harvested(
    env: &Env,
    depositor: &Address,
    original_token: &Address,
    replacement_token: &Address,
    realized_loss: i128,
    tax_benefit: i128,
    original_deposit_id: u32,
    replacement_deposit_id: u32,
    wash_sale_until: u64,
) {
    let topics = (Symbol::new(env, "tax_loss_harvested"), depositor.clone());
    env.events().publish(
        topics,
        (
            original_token.clone(),
            replacement_token.clone(),
            realized_loss,
            tax_benefit,
            original_deposit_id,
            replacement_deposit_id,
            wash_sale_until,
        ),
    );
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

pub fn circuit_breaker_tripped(
    env: &Env,
    admin: &Address,
    ledger: u32,
    amount: i128,
    threshold: i128,
) {
    let topics = (Symbol::new(env, "CircuitBreakerTripped"), admin.clone(), ledger);
    env.events().publish(topics, (amount, threshold));
}

pub fn token_proposed(env: &Env, token: &Address, proposer: &Address) {
    let topics = (Symbol::new(env, "token_proposed"), token.clone());
    env.events().publish(topics, proposer.clone());
}

pub fn token_reviewed(env: &Env, token: &Address, reviewer: &Address, passed: bool) {
    let topics = (Symbol::new(env, "token_reviewed"), token.clone());
    env.events().publish(topics, (reviewer.clone(), passed));
}

pub fn token_approved(env: &Env, token: &Address, approver: &Address) {
    let topics = (Symbol::new(env, "token_approved"), token.clone());
    env.events().publish(topics, approver.clone());
}

pub fn proposal_created(env: &Env, proposal_id: u32, proposer: &Address) {
    let topics = (Symbol::new(env, "ProposalCreated"), proposal_id);
    env.events().publish(topics, proposer.clone());
}

pub fn proposal_voted(env: &Env, proposal_id: u32, voter: &Address, support: bool, weight: i128) {
    let topics = (Symbol::new(env, "Voted"), proposal_id, voter.clone());
    env.events().publish(topics, (support, weight));
}

pub fn proposal_executed(env: &Env, proposal_id: u32) {
    let topics = (Symbol::new(env, "ProposalExecuted"), proposal_id);
    env.events().publish(topics, ());
}

pub fn governance_proposed(env: &Env, proposal_id: u32, proposer: &Address) {
    proposal_created(env, proposal_id, proposer);
}

pub fn governance_voted(env: &Env, proposal_id: u32, voter: &Address, support: bool, weight: i128) {
    proposal_voted(env, proposal_id, voter, support, weight);
}

pub fn governance_executed(env: &Env, proposal_id: u32) {
    proposal_executed(env, proposal_id);
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

/// Emitted when a deposit NFT evolves to a new stage.
pub fn nft_evolved(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
    old_stage: crate::nft::EvolutionStage,
    new_stage: crate::nft::EvolutionStage,
    rarity: crate::nft::RarityTier,
) {
    let topics = (Symbol::new(env, "nft_evolved"), depositor.clone());
    env.events()
        .publish(topics, (deposit_id, old_stage, new_stage, rarity));
}

/// Emitted when a deposit is blocked by rate limiting (issue #492).
pub fn deposit_rate_limited(env: &Env, depositor: &Address, deposit_count: u32, window_start: u64) {
    let topics = (Symbol::new(env, "rate_limited"), depositor.clone());
    env.events().publish(topics, (deposit_count, window_start));
}

/// Emitted when a deposit is scheduled (issue #494).
pub fn deposit_scheduled(
    env: &Env,
    depositor: &Address,
    token: &Address,
    amount: i128,
    execute_after: u64,
    schedule_id: u32,
) {
    let topics = (Symbol::new(env, "dep_scheduled"), depositor.clone(), token.clone());
    env.events().publish(topics, (amount, execute_after, schedule_id));
}

/// Emitted when a scheduled deposit is executed (issue #494).
pub fn scheduled_deposit_executed(
    env: &Env,
    depositor: &Address,
    schedule_id: u32,
    deposit_id: u32,
) {
    let topics = (Symbol::new(env, "sched_executed"), depositor.clone());
    env.events().publish(topics, (schedule_id, deposit_id));
}

/// Emitted when a scheduled deposit is cancelled (issue #494).
pub fn scheduled_deposit_cancelled(env: &Env, depositor: &Address, schedule_id: u32) {
    let topics = (Symbol::new(env, "sched_cancel"), depositor.clone());
    env.events().publish(topics, schedule_id);
}

/// Emitted when a penalty is split between fee recipient and stakers (or insurance pool).
pub fn penalty_split(
    env: &Env,
    depositor: &Address,
    penalty: i128,
    fee_recipient_share: i128,
    stakers_share: i128,
    deposit_id: u32,
) {
    let topics = (Symbol::new(env, "penalty_split"), depositor.clone());
    env.events()
        .publish(topics, (penalty, fee_recipient_share, stakers_share, deposit_id));
}

/// Emitted when a staker registers or updates their stake.
pub fn staker_registered(env: &Env, staker: &Address, amount: i128) {
    let topics = (Symbol::new(env, "staker_reg"), staker.clone());
    env.events().publish(topics, amount);
}

/// Emitted when a staker claims their rewards.
pub fn rewards_claimed(env: &Env, staker: &Address, amount: i128) {
    let topics = (Symbol::new(env, "rewards_claimed"), staker.clone());
    env.events().publish(topics, amount);
}

// ================================================================
//  Insurance Pool Events (issue #493)
// ================================================================

/// Emitted when an insurance claim is filed (issue #493).
pub fn insurance_claim_filed(
    env: &Env,
    claimant: &Address,
    token: &Address,
    claim_id: u32,
    amount_requested: i128,
) {
    let topics = (Symbol::new(env, "ins_claim_filed"), claimant.clone(), token.clone());
    env.events().publish(topics, (claim_id, amount_requested));
}

/// Emitted when an insurance claim is approved (issue #493).
pub fn insurance_claim_approved(
    env: &Env,
    admin: &Address,
    claimant: &Address,
    claim_id: u32,
    amount_disbursed: i128,
) {
    let topics = (Symbol::new(env, "ins_claim_appvd"), admin.clone());
    env.events().publish(topics, (claimant.clone(), claim_id, amount_disbursed));
}

/// Emitted when an insurance claim is denied (issue #493).
pub fn insurance_claim_denied(env: &Env, admin: &Address, claim_id: u32) {
    let topics = (Symbol::new(env, "ins_claim_denied"), admin.clone());
    env.events().publish(topics, claim_id);
}

/// Emitted when the insurance pool receives funds (issue #493).
pub fn insurance_pool_funded(env: &Env, token: &Address, amount: i128, new_balance: i128) {
    let topics = (Symbol::new(env, "ins_pool_funded"), token.clone());
    env.events().publish(topics, (amount, new_balance));
}
