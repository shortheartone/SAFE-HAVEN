// ============================================================
//  SAFE-HAVEN — Performance Analytics Module
//  Deposit performance metrics, benchmarks, time-weighted returns
// ============================================================

use soroban_sdk::{Address, Env, Vec};

use crate::types::{
    BenchmarkIndex, DepositPerformance, DepositorPerformanceSummary, PerformanceComparison,
    VaultEntry, MultiTokenVaultEntry, LedgerVaultEntry,
};

/// Annual interest rate in basis points (500 = 5% per annum)
const ANNUAL_INTEREST_BPS: u128 = 500;

/// Seconds per year (non-leap year)
const SECS_PER_YEAR: u128 = 31_536_000;

/// Basis points per whole percent
const BASIS_POINTS_PER_PERCENT: u32 = 10_000;

/// Estimated gas cost for a simple deposit operation (in stroops)
const ESTIMATED_DEPOSIT_GAS_COST: i128 = 100_000;

/// Estimated gas cost for a withdraw operation (in stroops)
const ESTIMATED_WITHDRAW_GAS_COST: i128 = 150_000;

// ================================================================
// Utility Functions
// ================================================================

/// Clamp a value between min and max
fn clamp(value: i128, min: i128, max: i128) -> i128 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Safe division that avoids overflow and division by zero
/// Returns (numerator / denominator) * scale, or 0 if denominator is 0
fn safe_div_with_scale(numerator: i128, denominator: i128, scale: u32) -> u32 {
    if denominator == 0 {
        return 0;
    }

    let num = numerator as i128;
    let denom = denominator as i128;

    if denom == 0 {
        return 0;
    }

    // Perform division with scale: (num / denom) * scale
    let scaled = (num as u128)
        .saturating_mul(scale as u128)
        .saturating_div(denom as u128);

    // Clamp to u32 range
    clamp(scaled as i128, 0, u32::MAX as i128) as u32
}

/// Calculate benchmark return based on the selected index and time elapsed
/// Returns return percentage in basis points
fn calculate_benchmark_return(index: &BenchmarkIndex, time_elapsed_secs: u64) -> u32 {
    if time_elapsed_secs == 0 {
        return 0;
    }

    let benchmark_bps: u32 = match index {
        BenchmarkIndex::ContractDefault => 500,     // 5% p.a.
        BenchmarkIndex::StellarInflation => 100,    // 1% p.a.
        BenchmarkIndex::MoneyMarket => 200,         // 2% p.a.
        BenchmarkIndex::SAndP500 => 1000,           // 10% p.a.
        BenchmarkIndex::Custom(bps) => *bps,        // Custom rate
    };

    // Calculate pro-rata return: (benchmark_bps / BASIS_POINTS_PER_PERCENT) * (time_elapsed_secs / SECS_PER_YEAR)
    let time_fraction: u128 = (time_elapsed_secs as u128)
        .saturating_mul(BASIS_POINTS_PER_PERCENT as u128)
        .saturating_div(SECS_PER_YEAR);

    let return_bps: u128 = (benchmark_bps as u128)
        .saturating_mul(time_fraction)
        .saturating_div(BASIS_POINTS_PER_PERCENT as u128);

    clamp(return_bps as i128, 0, u32::MAX as i128) as u32
}

// ================================================================
// Single Deposit Performance Calculations
// ================================================================

/// Calculate performance metrics for a timestamp-based single-token deposit
pub fn calculate_deposit_performance(
    _env: &Env,
    deposit: &VaultEntry,
    current_time: u64,
    benchmark: &BenchmarkIndex,
) -> DepositPerformance {
    let principal = deposit.amount;
    let is_unlocked = current_time >= deposit.unlock_time;
    let time_remaining_secs = if is_unlocked {
        0
    } else {
        deposit.unlock_time.saturating_sub(current_time)
    };

    // Calculate accrued interest from compound accumulation
    let current_value = calculate_accrued_amount(
        principal,
        deposit.compound_frequency_secs,
        deposit.last_accrual_timestamp,
        current_time,
    );

    let absolute_gain = current_value.saturating_sub(principal);
    let compound_interest = if deposit.compound_frequency_secs > 0 {
        absolute_gain
    } else {
        0
    };

    // Calculate simple return percentage in basis points
    let return_bps = safe_div_with_scale(absolute_gain, principal, BASIS_POINTS_PER_PERCENT);

    // Calculate time-weighted return: adjust for time held
    let deposit_created_time = deposit.last_accrual_timestamp;
    let time_held = if deposit_created_time > 0 {
        current_time.saturating_sub(deposit_created_time)
    } else {
        0
    };

    let time_weighted_return_bps = if time_held > 0 {
        // TWR = (current_value / principal) ^ (1 year / time_held) - 1, in basis points
        // Approximation: TWR ≈ (return_bps / time_held) * SECS_PER_YEAR
        let annualized = (return_bps as u128)
            .saturating_mul(SECS_PER_YEAR)
            .saturating_div(time_held.max(1) as u128);
        clamp(annualized as i128, 0, u32::MAX as i128) as u32
    } else {
        0
    };

    // Calculate benchmark performance
    let benchmark_return_bps = calculate_benchmark_return(benchmark, time_held);
    let outperformance_bps: i32 = (return_bps as i32).saturating_sub(benchmark_return_bps as i32);

    // Estimate gas cost and return on gas
    let estimated_gas_cost = ESTIMATED_DEPOSIT_GAS_COST + ESTIMATED_WITHDRAW_GAS_COST;
    let return_on_gas = if estimated_gas_cost > 0 {
        absolute_gain.saturating_div(estimated_gas_cost)
    } else {
        0
    };

    DepositPerformance {
        principal,
        current_value,
        absolute_gain,
        return_bps,
        time_weighted_return_bps,
        accrued_interest: absolute_gain,
        compound_interest,
        total_fees_paid: 0, // No fees on successful deposits
        is_unlocked,
        time_remaining_secs,
        benchmark_return_bps,
        outperformance_bps,
        estimated_gas_cost,
        return_on_gas,
    }
}

/// Calculate performance metrics for a multi-token deposit
pub fn calculate_multi_token_deposit_performance(
    _env: &Env,
    deposit: &MultiTokenVaultEntry,
    current_time: u64,
    benchmark: &BenchmarkIndex,
) -> DepositPerformance {
    let is_unlocked = current_time >= deposit.unlock_time;
    let time_remaining_secs = if is_unlocked {
        0
    } else {
        deposit.unlock_time.saturating_sub(current_time)
    };

    // Sum all token amounts for multi-token deposits
    let mut principal: i128 = 0;
    let mut current_value: i128 = 0;

    for token_deposit in deposit.tokens.iter() {
        principal = principal.saturating_add(token_deposit.amount);
        let accrued = calculate_accrued_amount(
            token_deposit.amount,
            deposit.compound_frequency_secs,
            deposit.last_accrual_timestamp,
            current_time,
        );
        current_value = current_value.saturating_add(accrued);
    }

    let absolute_gain = current_value.saturating_sub(principal);
    let compound_interest = if deposit.compound_frequency_secs > 0 {
        absolute_gain
    } else {
        0
    };

    let return_bps = safe_div_with_scale(absolute_gain, principal, BASIS_POINTS_PER_PERCENT);

    let deposit_created_time = deposit.last_accrual_timestamp;
    let time_held = if deposit_created_time > 0 {
        current_time.saturating_sub(deposit_created_time)
    } else {
        0
    };

    let time_weighted_return_bps = if time_held > 0 {
        let annualized = (return_bps as u128)
            .saturating_mul(SECS_PER_YEAR)
            .saturating_div(time_held.max(1) as u128);
        clamp(annualized as i128, 0, u32::MAX as i128) as u32
    } else {
        0
    };

    let benchmark_return_bps = calculate_benchmark_return(benchmark, time_held);
    let outperformance_bps: i32 = (return_bps as i32).saturating_sub(benchmark_return_bps as i32);

    let estimated_gas_cost = (ESTIMATED_DEPOSIT_GAS_COST + ESTIMATED_WITHDRAW_GAS_COST)
        .saturating_mul(deposit.tokens.len() as i128);
    let return_on_gas = if estimated_gas_cost > 0 {
        absolute_gain.saturating_div(estimated_gas_cost)
    } else {
        0
    };

    DepositPerformance {
        principal,
        current_value,
        absolute_gain,
        return_bps,
        time_weighted_return_bps,
        accrued_interest: absolute_gain,
        compound_interest,
        total_fees_paid: 0,
        is_unlocked,
        time_remaining_secs,
        benchmark_return_bps,
        outperformance_bps,
        estimated_gas_cost,
        return_on_gas,
    }
}

/// Calculate performance metrics for a ledger-based deposit
pub fn calculate_ledger_deposit_performance(
    env: &Env,
    deposit: &LedgerVaultEntry,
    _current_time: u64,
    current_ledger: u32,
    benchmark: &BenchmarkIndex,
) -> DepositPerformance {
    let principal = deposit.amount;
    let is_unlocked = current_ledger >= deposit.unlock_ledger;
    
    // Estimate time remaining using 5 seconds per ledger
    let time_remaining_secs = if is_unlocked {
        0
    } else {
        (deposit.unlock_ledger.saturating_sub(current_ledger) as u64) * 5
    };

    // For ledger-based deposits, we don't accrue compound interest (simplified model)
    let current_value = principal;
    let absolute_gain = 0;
    let return_bps = 0;
    let compound_interest = 0;

    let benchmark_return_bps = calculate_benchmark_return(benchmark, time_remaining_secs);
    let outperformance_bps: i32 = 0i32.saturating_sub(benchmark_return_bps as i32);

    let estimated_gas_cost = ESTIMATED_DEPOSIT_GAS_COST + ESTIMATED_WITHDRAW_GAS_COST;
    let return_on_gas = 0;

    DepositPerformance {
        principal,
        current_value,
        absolute_gain,
        return_bps,
        time_weighted_return_bps: 0,
        accrued_interest: 0,
        compound_interest,
        total_fees_paid: 0,
        is_unlocked,
        time_remaining_secs,
        benchmark_return_bps,
        outperformance_bps,
        estimated_gas_cost,
        return_on_gas,
    }
}

// ================================================================
// Aggregated Performance Calculations
// ================================================================

/// Calculate aggregated performance summary across multiple deposits
pub fn calculate_depositor_summary(
    env: &Env,
    performances: &Vec<DepositPerformance>,
) -> DepositorPerformanceSummary {
    let mut total_principal: i128 = 0;
    let mut total_current_value: i128 = 0;
    let mut total_absolute_gain: i128 = 0;
    let mut total_compound_interest: i128 = 0;
    let mut total_fees_paid: i128 = 0;
    let mut unlocked_count: u32 = 0;
    let mut locked_count: u32 = 0;
    let mut total_unlocked_value: i128 = 0;
    let mut total_benchmark_return: u128 = 0;
    let mut total_outperformance: i128 = 0;
    let mut total_gas_cost: i128 = 0;
    let mut total_return_on_gas: i128 = 0;

    let deposit_count = performances.len();

    for perf in performances.iter() {
        total_principal = total_principal.saturating_add(perf.principal);
        total_current_value = total_current_value.saturating_add(perf.current_value);
        total_absolute_gain = total_absolute_gain.saturating_add(perf.absolute_gain);
        total_compound_interest = total_compound_interest.saturating_add(perf.compound_interest);
        total_fees_paid = total_fees_paid.saturating_add(perf.total_fees_paid);
        total_gas_cost = total_gas_cost.saturating_add(perf.estimated_gas_cost);
        total_return_on_gas = total_return_on_gas.saturating_add(perf.return_on_gas);

        if perf.is_unlocked {
            unlocked_count += 1;
            total_unlocked_value = total_unlocked_value.saturating_add(perf.current_value);
        } else {
            locked_count += 1;
        }

        total_benchmark_return = total_benchmark_return.saturating_add(perf.benchmark_return_bps as u128);
        total_outperformance = total_outperformance.saturating_add(perf.outperformance_bps as i128);
    }

    // Calculate weighted average return
    let weighted_avg_return_bps = if total_principal > 0 && deposit_count > 0 {
        safe_div_with_scale(total_absolute_gain, total_principal, BASIS_POINTS_PER_PERCENT)
    } else {
        0
    };

    // Calculate average time-weighted return
    let avg_time_weighted_return_bps = if deposit_count > 0 {
        performances
            .iter()
            .map(|p| p.time_weighted_return_bps as u128)
            .fold(0, |acc, x| acc.saturating_add(x))
            / (deposit_count as u128).max(1)
    } else {
        0
    } as u32;

    // Calculate average benchmark return
    let avg_benchmark_return_bps = if deposit_count > 0 {
        (total_benchmark_return / (deposit_count as u128)) as u32
    } else {
        0
    };

    // Calculate portfolio outperformance
    let portfolio_outperformance_bps: i32 = if deposit_count > 0 {
        (weighted_avg_return_bps as i128).saturating_sub(avg_benchmark_return_bps as i128) as i32
    } else {
        0
    };

    // Calculate average return on gas
    let avg_return_on_gas = if deposit_count > 0 && total_gas_cost > 0 {
        total_return_on_gas / (deposit_count as i128)
    } else {
        0
    };

    DepositorPerformanceSummary {
        deposit_count: deposit_count as u32,
        total_principal,
        total_current_value,
        total_absolute_gain,
        weighted_avg_return_bps,
        avg_time_weighted_return_bps,
        total_accrued_interest: total_absolute_gain,
        total_compound_interest,
        total_fees_paid,
        unlocked_deposit_count: unlocked_count,
        total_unlocked_value,
        locked_deposit_count: locked_count,
        avg_benchmark_return_bps,
        portfolio_outperformance_bps,
        total_estimated_gas_cost: total_gas_cost,
        avg_return_on_gas,
    }
}

/// Compare performance between two scenarios
pub fn compare_performance(first_return_bps: u32, second_return_bps: u32) -> PerformanceComparison {
    let difference_bps: i32 = (first_return_bps as i32).saturating_sub(second_return_bps as i32);
    let first_outperforms = difference_bps > 0;

    PerformanceComparison {
        first_return_bps,
        second_return_bps,
        difference_bps,
        first_outperforms,
    }
}

// ================================================================
// Helper: Calculate Accrued Amount (From Contract)
// ================================================================

/// Compute the accrued balance for a deposit as of `now`.
/// Uses compound interest applied period-by-period.
fn calculate_accrued_amount(amount: i128, entry_freq: u64, last_accrual: u64, now: u64) -> i128 {
    if entry_freq == 0 || now <= last_accrual {
        return amount;
    }

    let elapsed = now.saturating_sub(last_accrual);
    let periods = elapsed / entry_freq;

    if periods == 0 {
        return amount;
    }

    let denominator: u128 = SECS_PER_YEAR.saturating_mul(10_000);
    let freq_u128 = entry_freq as u128;

    let mut balance = amount;
    let periods_capped = periods.min(2_628_000); // 5 years ÷ 60 s/period
    for _ in 0..periods_capped {
        let interest = (balance as u128)
            .saturating_mul(ANNUAL_INTEREST_BPS)
            .saturating_mul(freq_u128)
            / denominator;
        balance = balance.saturating_add(interest as i128);
    }
    balance
}
