// ================================================================
//  Liquidation Protection Module
//
//  Provides health ratio calculation, grace period management,
//  and liquidation checks for collateralized deposits.
// ================================================================

use soroban_sdk::{Env, Address};

use crate::{
    constants::{
        DEFAULT_LIQUIDATION_THRESHOLD_BPS, DEFAULT_WARNING_THRESHOLD_BPS,
        DEFAULT_GRACE_PERIOD_SECS, MIN_GRACE_PERIOD_SECS, MAX_GRACE_PERIOD_SECS,
    },
    errors::VaultError,
    types::{HealthStatus, HealthRatio, LiquidationProtection},
};

/// Calculates the health ratio for a collateralized deposit.
///
/// Health ratio = (collateral_amount / deposit_amount) * 10000
/// Expressed in basis points where 10000 = 1.0x.
///
/// Returns a HealthRatio struct containing:
/// - ratio_bps: The computed health ratio in basis points
/// - status: The health status based on thresholds
/// - Current collateral and deposit amounts
/// - Time remaining in grace period (if applicable)
///
/// # Arguments
/// * `collateral_amount` - Amount of collateral backing the deposit
/// * `deposit_amount` - The deposit/liability amount
/// * `liquidation_threshold_bps` - Liquidation threshold in basis points
/// * `warning_threshold_bps` - Warning threshold in basis points
/// * `grace_period_start` - Timestamp when grace period started (0 if none)
/// * `grace_period_secs` - Duration of grace period in seconds
/// * `now` - Current timestamp
///
/// # Example
/// If collateral = 2000 and deposit = 1000:
/// health_ratio = (2000 / 1000) * 10000 = 20000 (2.0x health)
pub fn calculate_health_ratio(
    collateral_amount: i128,
    deposit_amount: i128,
    liquidation_threshold_bps: u32,
    warning_threshold_bps: u32,
    grace_period_start: u64,
    grace_period_secs: u64,
    now: u64,
) -> HealthRatio {
    let ratio_bps = if deposit_amount <= 0 {
        u32::MAX // Infinite health if no deposit
    } else {
        let ratio_u128 = (collateral_amount as u128)
            .saturating_mul(10_000)
            .checked_div(deposit_amount as u128)
            .unwrap_or(u32::MAX as u128);
        (ratio_u128.min(u32::MAX as u128)) as u32
    };

    // Determine health status
    let (status, grace_remaining) = if grace_period_start > 0 {
        let grace_end = grace_period_start.saturating_add(grace_period_secs);
        if now < grace_end {
            // In grace period
            let remaining = grace_end.saturating_sub(now);
            (HealthStatus::GracePeriod, remaining)
        } else {
            // Grace period expired
            (HealthStatus::Liquidatable, 0)
        }
    } else if ratio_bps <= liquidation_threshold_bps {
        // Critical risk without grace period
        (HealthStatus::CriticalRisk, 0)
    } else if ratio_bps <= warning_threshold_bps {
        // Warning level
        (HealthStatus::Warning, 0)
    } else {
        // Healthy
        (HealthStatus::Healthy, 0)
    };

    HealthRatio {
        ratio_bps,
        status,
        collateral_amount,
        deposit_amount,
        grace_period_remaining_secs: grace_remaining,
    }
}

/// Validates liquidation threshold configuration.
///
/// Valid range: 10000-50000 basis points (1.0x - 5.0x)
///
/// Returns Err if threshold is outside valid range.
pub fn validate_liquidation_threshold(threshold_bps: u32) -> Result<(), VaultError> {
    if threshold_bps < 10_000 || threshold_bps > 50_000 {
        return Err(VaultError::InvalidLiquidationThreshold);
    }
    Ok(())
}

/// Validates grace period configuration.
///
/// Valid range: MIN_GRACE_PERIOD_SECS to MAX_GRACE_PERIOD_SECS
///
/// Returns Err if grace period is outside valid range.
pub fn validate_grace_period(grace_period_secs: u64) -> Result<(), VaultError> {
    if grace_period_secs < MIN_GRACE_PERIOD_SECS || grace_period_secs > MAX_GRACE_PERIOD_SECS {
        return Err(VaultError::InvalidGracePeriod);
    }
    Ok(())
}

/// Creates a new LiquidationProtection structure with given parameters.
///
/// Uses default thresholds if not overridden:
/// - Liquidation threshold: 150 (1.5x)
/// - Warning threshold: 200 (2.0x)
/// - Grace period: 7 days
///
/// # Arguments
/// * `collateral_amount` - Initial collateral amount
/// * `liquidation_threshold_bps` - Custom threshold, or 0 to use default
/// * `warning_threshold_bps` - Custom threshold, or 0 to use default
/// * `grace_period_secs` - Custom grace period, or 0 to use default
///
/// # Returns
/// LiquidationProtection struct or VaultError if validation fails
pub fn create_liquidation_protection(
    collateral_amount: i128,
    liquidation_threshold_bps: u32,
    warning_threshold_bps: u32,
    grace_period_secs: u64,
) -> Result<LiquidationProtection, VaultError> {
    // Use defaults if not specified
    let liq_threshold = if liquidation_threshold_bps == 0 {
        DEFAULT_LIQUIDATION_THRESHOLD_BPS
    } else {
        liquidation_threshold_bps
    };

    let warn_threshold = if warning_threshold_bps == 0 {
        DEFAULT_WARNING_THRESHOLD_BPS
    } else {
        warning_threshold_bps
    };

    let grace_secs = if grace_period_secs == 0 {
        DEFAULT_GRACE_PERIOD_SECS
    } else {
        grace_period_secs
    };

    // Validate thresholds
    validate_liquidation_threshold(liq_threshold)?;
    validate_grace_period(grace_secs)?;

    // Warning threshold should be > liquidation threshold
    if warn_threshold <= liq_threshold {
        return Err(VaultError::InvalidLiquidationThreshold);
    }

    Ok(LiquidationProtection {
        collateral_amount,
        liquidation_threshold_bps: liq_threshold,
        warning_threshold_bps: warn_threshold,
        grace_period_secs: grace_secs,
        grace_period_start: 0,
        warning_emitted: false,
        last_grace_period_reset: 0,
    })
}

/// Starts the grace period for a liquidation protection.
///
/// Sets grace_period_start to current timestamp.
/// Grace period expires grace_period_secs after start.
///
/// # Arguments
/// * `mut_liq_prot` - Mutable reference to LiquidationProtection
/// * `now` - Current timestamp
pub fn start_grace_period(
    liq_prot: &mut LiquidationProtection,
    now: u64,
) {
    liq_prot.grace_period_start = now;
    liq_prot.last_grace_period_reset = now;
}

/// Checks if grace period has expired.
///
/// Returns true if:
/// - Grace period was started (grace_period_start > 0), AND
/// - Current time >= grace_period_start + grace_period_secs
pub fn is_grace_period_expired(
    liq_prot: &LiquidationProtection,
    now: u64,
) -> bool {
    if liq_prot.grace_period_start == 0 {
        return false;
    }
    let grace_end = liq_prot.grace_period_start.saturating_add(liq_prot.grace_period_secs);
    now >= grace_end
}

/// Checks if a deposit is in active grace period.
///
/// Returns true if grace period was started and hasn't expired yet.
pub fn is_in_grace_period(
    liq_prot: &LiquidationProtection,
    now: u64,
) -> bool {
    if liq_prot.grace_period_start == 0 {
        return false;
    }
    !is_grace_period_expired(liq_prot, now)
}

/// Resets the grace period, clearing its started timestamp.
///
/// Used when collateral is added and health improves above warning threshold.
pub fn reset_grace_period(liq_prot: &mut LiquidationProtection) {
    liq_prot.grace_period_start = 0;
    liq_prot.warning_emitted = false;
}

/// Adds collateral to a liquidation protection.
///
/// # Arguments
/// * `liq_prot` - Mutable reference to LiquidationProtection
/// * `additional_collateral` - Amount to add
///
/// # Returns
/// New total collateral amount
pub fn add_collateral(
    liq_prot: &mut LiquidationProtection,
    additional_collateral: i128,
) -> i128 {
    liq_prot.collateral_amount = liq_prot.collateral_amount.saturating_add(additional_collateral);
    liq_prot.collateral_amount
}

/// Allows adding collateral during grace period.
///
/// When a deposit is in grace period (health ratio has fallen below liquidation threshold),
/// the depositor can add more collateral to improve their health ratio and avoid liquidation.
///
/// # Arguments
/// * `liq_prot` - Mutable reference to LiquidationProtection
/// * `additional_collateral` - Amount to add
/// * `deposit_amount` - The deposit/liability amount (needed to calculate new health)
///
/// # Returns
/// Result with new health ratio in basis points if successful, or error
pub fn add_collateral_during_grace_period(
    liq_prot: &mut LiquidationProtection,
    additional_collateral: i128,
    deposit_amount: i128,
) -> Result<u32, VaultError> {
    if additional_collateral <= 0 {
        return Err(VaultError::InvalidAmount);
    }

    // Add collateral
    add_collateral(liq_prot, additional_collateral);

    // Calculate new health ratio
    let new_health_bps = if deposit_amount <= 0 {
        u32::MAX
    } else {
        let ratio_u128 = (liq_prot.collateral_amount as u128)
            .saturating_mul(10_000)
            .checked_div(deposit_amount as u128)
            .unwrap_or(u32::MAX as u128);
        (ratio_u128.min(u32::MAX as u128)) as u32
    };

    // If health improves above warning threshold, reset grace period
    if new_health_bps > liq_prot.warning_threshold_bps {
        reset_grace_period(liq_prot);
    }

    Ok(new_health_bps)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_health_ratio_healthy() {
        // Collateral 2000, Deposit 1000 -> 2.0x = 20000 bps
        let ratio = calculate_health_ratio(
            2000,
            1000,
            15_000,
            20_000,
            0,
            604_800,
            1000,
        );
        assert_eq!(ratio.ratio_bps, 20_000);
        assert_eq!(ratio.status, HealthStatus::Healthy);
    }

    #[test]
    fn test_calculate_health_ratio_warning() {
        // Collateral 1800, Deposit 1000 -> 1.8x = 18000 bps
        let ratio = calculate_health_ratio(
            1800,
            1000,
            15_000,
            20_000,
            0,
            604_800,
            1000,
        );
        assert_eq!(ratio.ratio_bps, 18_000);
        assert_eq!(ratio.status, HealthStatus::Warning);
    }

    #[test]
    fn test_calculate_health_ratio_critical() {
        // Collateral 1400, Deposit 1000 -> 1.4x = 14000 bps
        let ratio = calculate_health_ratio(
            1400,
            1000,
            15_000,
            20_000,
            0,
            604_800,
            1000,
        );
        assert_eq!(ratio.ratio_bps, 14_000);
        assert_eq!(ratio.status, HealthStatus::CriticalRisk);
    }

    #[test]
    fn test_calculate_health_ratio_in_grace_period() {
        let grace_start = 1000;
        let grace_secs = 604_800;
        let now = 1000 + 300_000; // In grace period

        let ratio = calculate_health_ratio(
            1400,
            1000,
            15_000,
            20_000,
            grace_start,
            grace_secs,
            now,
        );
        assert_eq!(ratio.status, HealthStatus::GracePeriod);
        assert!(ratio.grace_period_remaining_secs > 0);
    }

    #[test]
    fn test_calculate_health_ratio_grace_expired() {
        let grace_start = 1000;
        let grace_secs = 604_800;
        let now = grace_start.saturating_add(grace_secs).saturating_add(1); // Grace expired

        let ratio = calculate_health_ratio(
            1400,
            1000,
            15_000,
            20_000,
            grace_start,
            grace_secs,
            now,
        );
        assert_eq!(ratio.status, HealthStatus::Liquidatable);
        assert_eq!(ratio.grace_period_remaining_secs, 0);
    }

    #[test]
    fn test_validate_liquidation_threshold_valid() {
        assert!(validate_liquidation_threshold(15_000).is_ok());
        assert!(validate_liquidation_threshold(10_000).is_ok());
        assert!(validate_liquidation_threshold(50_000).is_ok());
    }

    #[test]
    fn test_validate_liquidation_threshold_invalid() {
        assert!(validate_liquidation_threshold(9_999).is_err());
        assert!(validate_liquidation_threshold(50_001).is_err());
    }

    #[test]
    fn test_create_liquidation_protection_defaults() {
        let liq = create_liquidation_protection(1000, 0, 0, 0).unwrap();
        assert_eq!(liq.collateral_amount, 1000);
        assert_eq!(liq.liquidation_threshold_bps, DEFAULT_LIQUIDATION_THRESHOLD_BPS);
        assert_eq!(liq.warning_threshold_bps, DEFAULT_WARNING_THRESHOLD_BPS);
        assert_eq!(liq.grace_period_secs, DEFAULT_GRACE_PERIOD_SECS);
    }

    #[test]
    fn test_is_grace_period_expired() {
        let mut liq = create_liquidation_protection(1000, 0, 0, 3600).unwrap();
        let start_time = 1000;
        start_grace_period(&mut liq, start_time);

        // Not expired yet
        assert!(!is_grace_period_expired(&liq, start_time + 1800));

        // Expired
        assert!(is_grace_period_expired(&liq, start_time + 3600));
    }

    #[test]
    fn test_add_collateral() {
        let mut liq = create_liquidation_protection(1000, 0, 0, 0).unwrap();
        let new_amount = add_collateral(&mut liq, 500);
        assert_eq!(new_amount, 1500);
        assert_eq!(liq.collateral_amount, 1500);
    }

    #[test]
    fn test_add_collateral_during_grace_period_improves_health() {
        let mut liq = create_liquidation_protection(1400, 0, 0, 3600).unwrap();
        let start_time = 1000;
        start_grace_period(&mut liq, start_time);

        // Current health: 1400/1000 = 1.4 (14000 bps) - below warning (20000 bps)
        // Add 600 collateral
        let new_health_bps = add_collateral_during_grace_period(&mut liq, 600, 1000).unwrap();

        // New health: 2000/1000 = 2.0 (20000 bps) - at warning level
        assert_eq!(new_health_bps, 20_000);

        // Grace period should be reset since we're at warning threshold
        assert_eq!(liq.grace_period_start, 0);
        assert!(!liq.warning_emitted);
    }

    #[test]
    fn test_add_collateral_invalid_amount() {
        let mut liq = create_liquidation_protection(1000, 0, 0, 0).unwrap();
        let result = add_collateral_during_grace_period(&mut liq, 0, 1000);
        assert!(result.is_err());
    }

    #[test]
    fn test_grace_period_flow() {
        // Create protection with 30 minute grace period
        let mut liq = create_liquidation_protection(1000, 15_000, 20_000, 1800).unwrap();

        // Start time
        let now = 10_000;

        // Verify not in grace period initially
        assert_eq!(liq.grace_period_start, 0);
        assert!(!is_in_grace_period(&liq, now));

        // Start grace period
        start_grace_period(&mut liq, now);
        assert_eq!(liq.grace_period_start, now);
        assert!(is_in_grace_period(&liq, now));
        assert!(!is_grace_period_expired(&liq, now));

        // Grace period active
        assert!(is_in_grace_period(&liq, now + 900)); // 15 min
        assert!(!is_grace_period_expired(&liq, now + 900));

        // Grace period expired
        assert!(!is_in_grace_period(&liq, now + 1801));
        assert!(is_grace_period_expired(&liq, now + 1801));
    }

    #[test]
    fn test_warning_threshold_validation() {
        // Warning threshold must be > liquidation threshold
        let result = create_liquidation_protection(
            1000,
            20_000, // liquidation threshold
            15_000, // warning threshold (too low!)
            0,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_collateral_additions() {
        let mut liq = create_liquidation_protection(1000, 0, 0, 0).unwrap();

        // Add collateral multiple times
        let amt1 = add_collateral(&mut liq, 200);
        assert_eq!(amt1, 1200);

        let amt2 = add_collateral(&mut liq, 300);
        assert_eq!(amt2, 1500);

        let amt3 = add_collateral(&mut liq, 100);
        assert_eq!(amt3, 1600);

        assert_eq!(liq.collateral_amount, 1600);
    }

    #[test]
    fn test_health_status_transitions() {
        // Healthy -> Warning -> Critical -> Grace Period -> Liquidatable

        // Start with 2.5x health (healthy)
        let mut health = calculate_health_ratio(
            2500, 1000,
            15_000, 20_000,
            0, 0, 1000,
        );
        assert_eq!(health.status, HealthStatus::Healthy);

        // Drop to 1.8x (warning)
        health = calculate_health_ratio(
            1800, 1000,
            15_000, 20_000,
            0, 0, 1000,
        );
        assert_eq!(health.status, HealthStatus::Warning);

        // Drop to 1.2x (critical)
        health = calculate_health_ratio(
            1200, 1000,
            15_000, 20_000,
            0, 0, 1000,
        );
        assert_eq!(health.status, HealthStatus::CriticalRisk);

        // In grace period
        health = calculate_health_ratio(
            1200, 1000,
            15_000, 20_000,
            1000, 604_800, 1000, // just started
        );
        assert_eq!(health.status, HealthStatus::GracePeriod);

        // Grace period expired
        health = calculate_health_ratio(
            1200, 1000,
            15_000, 20_000,
            1000, 604_800, 1000 + 604_801, // expired
        );
        assert_eq!(health.status, HealthStatus::Liquidatable);
    }
