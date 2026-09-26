// ============================================================
//  Yield Farming Integration Module
//  Issue: #XXX
// ============================================================

use soroban_sdk::{token, Address, Env};

use crate::{
    constants::{
        MIN_FARMING_AMOUNT, MAX_FARMING_PROPORTION_BPS, FARMING_RISK_LEVEL,
        FARMING_EXPECTED_ANNUAL_YIELD_BPS,
    },
    errors::VaultError,
    events, storage,
    types::{FarmingConfig, FarmingState, FarmingStrategy, VaultEntry},
};

/// Initialize yield farming configuration (admin only)
pub fn initialize_farming_config(
    env: &Env,
    admin: &Address,
) -> Result<(), VaultError> {
    admin.require_auth();
    storage::require_admin(env, admin)?;

    // Create default farming configuration
    let config = FarmingConfig {
        enabled: true,
        approved_protocols: soroban_sdk::Vec::new(env),
        min_farming_amount: MIN_FARMING_AMOUNT,
        max_farming_proportion_bps: MAX_FARMING_PROPORTION_BPS,
        risk_level: FARMING_RISK_LEVEL,
    };

    storage::set_farming_config(env, &config);
    Ok(())
}

/// Add an approved farming protocol (admin only)
pub fn add_farming_protocol(
    env: &Env,
    admin: &Address,
    protocol_address: &Address,
) -> Result<(), VaultError> {
    admin.require_auth();
    storage::require_admin(env, admin)?;

    let mut config = storage::get_farming_config(env)
        .ok_or(VaultError::InvalidParameter)?;

    // Check if protocol already added
    for p in config.approved_protocols.iter() {
        if p == *protocol_address {
            return Err(VaultError::ProtocolAlreadyApproved);
        }
    }

    config.approved_protocols.push_back(protocol_address.clone());
    storage::set_farming_config(env, &config);

    Ok(())
}

/// Enable yield farming for a deposit
pub fn enable_yield_farming(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
    strategy: FarmingStrategy,
    protocol_address: &Address,
) -> Result<(), VaultError> {
    depositor.require_auth();

    // Check if farming is globally enabled
    let config = storage::get_farming_config(env)
        .ok_or(VaultError::FarmingNotConfigured)?;

    if !config.enabled {
        return Err(VaultError::FarmingDisabled);
    }

    // Verify protocol is approved
    let mut protocol_found = false;
    for approved in config.approved_protocols.iter() {
        if approved == *protocol_address {
            protocol_found = true;
            break;
        }
    }
    if !protocol_found {
        return Err(VaultError::UnapprovedFarmingProtocol);
    }

    // Get the deposit
    let deposit = storage::get_deposit(env, depositor, deposit_id)
        .ok_or(VaultError::DepositNotFound)?;

    // Ensure deposit is not already farmed
    if let Some(farming_state) = storage::get_farming_state_readonly(env, depositor, deposit_id) {
        if farming_state.enabled {
            return Err(VaultError::FarmingAlreadyEnabled);
        }
    }

    // Validate deposit amount for farming
    if deposit.amount < config.min_farming_amount {
        return Err(VaultError::InsufficientFundsForFarming);
    }

    // Calculate deployment amount (capped at max proportion)
    let max_deployable = (deposit.amount as u128)
        .saturating_mul(config.max_farming_proportion_bps as u128)
        / 10_000u128;
    let deployed_amount = (max_deployable as i128).min(deposit.amount);

    // Create farming state
    let farming_state = FarmingState {
        enabled: true,
        strategy: Some(strategy.clone()),
        deployed_amount,
        total_rewards: 0,
        enabled_at: env.ledger().timestamp(),
        last_claim_time: env.ledger().timestamp(),
        protocol_address: Some(protocol_address.clone()),
    };

    // Store farming state
    storage::set_farming_state(env, depositor, deposit_id, &farming_state);

    // Emit event
    let strategy_byte = match strategy {
        FarmingStrategy::DirectStaking => 0u8,
        FarmingStrategy::LiquidityProvision => 1u8,
        FarmingStrategy::LendingYield => 2u8,
    };

    events::farming_enabled(
        env,
        depositor,
        deposit_id,
        strategy_byte,
        deployed_amount,
        protocol_address,
    );

    Ok(())
}

/// Calculate accrued farming rewards based on time and strategy
/// Expected annual yield: 3% (300 bps)
pub fn calculate_farming_rewards(
    env: &Env,
    farming_state: &FarmingState,
    now: u64,
) -> i128 {
    if !farming_state.enabled {
        return 0;
    }

    let elapsed = now.saturating_sub(farming_state.last_claim_time);
    if elapsed == 0 {
        return 0;
    }

    // Simple pro-rata calculation: amount × rate × time / year_in_seconds
    let denominator: u128 = 365 * 24 * 60 * 60; // seconds per year
    let rewards = (farming_state.deployed_amount as u128)
        .saturating_mul(FARMING_EXPECTED_ANNUAL_YIELD_BPS)
        .saturating_mul(elapsed as u128)
        / (denominator * 10_000u128);

    rewards as i128
}

/// Claim farming rewards for a deposit
pub fn claim_farming_rewards(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
) -> Result<i128, VaultError> {
    depositor.require_auth();

    // Get farming state
    let mut farming_state = storage::get_farming_state(env, depositor, deposit_id)
        .ok_or(VaultError::FarmingNotEnabled)?;

    if !farming_state.enabled {
        return Err(VaultError::FarmingNotEnabled);
    }

    // Calculate accrued rewards
    let now = env.ledger().timestamp();
    let accrued_rewards = calculate_farming_rewards(env, &farming_state, now);

    if accrued_rewards <= 0 {
        return Ok(0); // Nothing to claim
    }

    // Update farming state
    farming_state.total_rewards = farming_state.total_rewards.saturating_add(accrued_rewards);
    farming_state.last_claim_time = now;
    storage::set_farming_state(env, depositor, deposit_id, &farming_state);

    // Track claimed rewards for auditing
    storage::add_farming_rewards_claimed(env, depositor, accrued_rewards);

    // Get deposit to retrieve token
    let deposit = storage::get_deposit(env, depositor, deposit_id)
        .ok_or(VaultError::DepositNotFound)?;

    // Transfer rewards to depositor (from farming protocol)
    // In a real implementation, this would involve withdrawing from the actual farming protocol
    // For now, we simulate this by assuming rewards are credited
    // In production: token::Client::new(env, &deposit.token)
    //   .transfer(&farming_state.protocol_address.unwrap(), depositor, &accrued_rewards);

    let strategy_byte = match farming_state.strategy {
        Some(FarmingStrategy::DirectStaking) => 0u8,
        Some(FarmingStrategy::LiquidityProvision) => 1u8,
        Some(FarmingStrategy::LendingYield) => 2u8,
        None => 0u8,
    };

    events::rewards_claimed(env, depositor, deposit_id, accrued_rewards, strategy_byte);

    Ok(accrued_rewards)
}

/// Disable farming for a deposit and withdraw deployed funds
pub fn disable_farming(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
) -> Result<(i128, i128), VaultError> {
    depositor.require_auth();

    let mut farming_state = storage::get_farming_state(env, depositor, deposit_id)
        .ok_or(VaultError::FarmingNotEnabled)?;

    if !farming_state.enabled {
        return Err(VaultError::FarmingNotEnabled);
    }

    // Calculate final rewards before disabling
    let now = env.ledger().timestamp();
    let final_rewards = calculate_farming_rewards(env, &farming_state, now);

    let deployed = farming_state.deployed_amount;
    let total_rewards = farming_state.total_rewards.saturating_add(final_rewards);

    // Disable farming
    farming_state.enabled = false;
    storage::set_farming_state(env, depositor, deposit_id, &farming_state);

    // Track final rewards
    if final_rewards > 0 {
        storage::add_farming_rewards_claimed(env, depositor, final_rewards);
    }

    // Emit event
    events::farming_disabled(env, depositor, deposit_id, deployed, total_rewards);

    Ok((deployed, total_rewards))
}

/// Get farming info for a deposit
pub fn get_farming_info(
    env: &Env,
    depositor: &Address,
    deposit_id: u32,
) -> Result<(bool, i128, i128), VaultError> {
    let farming_state = storage::get_farming_state_readonly(env, depositor, deposit_id)
        .ok_or(VaultError::FarmingNotEnabled)?;

    let now = env.ledger().timestamp();
    let accrued = calculate_farming_rewards(env, &farming_state, now);
    let total = farming_state.total_rewards.saturating_add(accrued);

    Ok((farming_state.enabled, farming_state.deployed_amount, total))
}
