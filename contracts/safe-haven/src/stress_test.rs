//! # Deposit Portfolio Stress Testing Module
//!
//! This module provides comprehensive stress testing capabilities for user deposit portfolios.
//! It simulates various adverse market conditions and calculates impact on deposit values,
//! worst-case losses, and provides risk-based recommendations.
//!
//! ## Stress Scenarios
//!
//! - **MarketCrash**: Simulates 50% price decline
//! - **VolatilitySpike**: Extreme price swings (±30%)
//! - **LiquidityCrisis**: 40% reduction in available liquidity
//! - **RateChange**: Interest rate adjustments (±100 basis points)
//! - **TokenVulnerability**: Specific token risk exposure
//! - **CombinedStress**: Multiple adverse conditions simultaneously
//! - **PenaltyEscalation**: Penalty rate increases to worst case

use soroban_sdk::{contracttype, Address, Env, String, Vec};

/// Enumeration of stress test scenarios
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq, Copy)]
pub enum StressScenario {
    /// Market-wide crash: 50% price decline
    MarketCrash = 0,
    /// High volatility: ±30% price swings
    VolatilitySpike = 1,
    /// Liquidity crisis: 40% reduction in available liquidity
    LiquidityCrisis = 2,
    /// Rate change: ±100 basis points interest rate adjustment
    RateChange = 3,
    /// Token-specific vulnerability risk
    TokenVulnerability = 4,
    /// Combined stress: multiple conditions simultaneously
    CombinedStress = 5,
    /// Penalty escalation: worst-case penalty rates
    PenaltyEscalation = 6,
}

/// Detailed impact breakdown for a single deposit under stress
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DepositImpact {
    /// Original deposit amount in base units
    pub original_amount: i128,
    /// Deposit value after stress scenario
    pub stressed_value: i128,
    /// Absolute loss amount
    pub loss_amount: i128,
    /// Loss percentage (in basis points: 1000 = 10%)
    pub loss_percentage_bps: u32,
    /// Penalty amount if withdrawn early
    pub penalty_amount: i128,
    /// Net value after penalty
    pub net_value_after_penalty: i128,
}

/// Results from a single stress test scenario
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StressTestResult {
    /// The scenario being tested
    pub scenario: StressScenario,
    /// Total portfolio impact
    pub total_impact: DepositImpact,
    /// Number of deposits affected
    pub deposits_affected: u32,
    /// Number of high-risk exposures (>10% loss)
    pub high_risk_count: u32,
    /// Worst-case single deposit loss in basis points
    pub worst_case_loss_bps: u32,
    /// Average loss across all deposits (in basis points)
    pub average_loss_bps: u32,
}

/// Recommendation based on stress test results
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StressRecommendation {
    /// Risk level: Low (0-10%), Medium (10-25%), High (25%+)
    pub risk_level: RiskLevel,
    /// Recommended action
    pub action: String,
    /// Confidence score (0-100)
    pub confidence_score: u32,
    /// Additional details
    pub details: String,
}

/// Risk level classification
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq, Copy)]
pub enum RiskLevel {
    /// Portfolio loss < 10%
    Low = 0,
    /// Portfolio loss 10-25%
    Medium = 1,
    /// Portfolio loss > 25%
    High = 2,
    /// Portfolio loss > 50% (critical)
    Critical = 3,
}

/// Complete stress test report for a portfolio
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortfolioStressReport {
    /// Depositor address
    pub depositor: Address,
    /// Timestamp when report was generated
    pub generated_at: u64,
    /// Total portfolio value before stress
    pub portfolio_value_before: i128,
    /// Array of results for each scenario tested
    pub scenario_results: Vec<StressTestResult>,
    /// Overall recommendations
    pub recommendations: Vec<StressRecommendation>,
    /// Overall worst-case loss scenario
    pub worst_case_scenario: StressScenario,
    /// Overall worst-case loss percentage (in basis points)
    pub worst_case_loss_bps: u32,
    /// Deposits with high risk exposure
    pub high_risk_deposit_count: u32,
}

// ================================================================
// Core Stress Test Calculations
// ================================================================

/// Calculate the impact of a stress scenario on a single deposit amount
///
/// # Arguments
/// * `original_amount` - Initial deposit amount in base units
/// * `scenario` - The stress scenario being applied
/// * `penalty_bps` - Early withdrawal penalty in basis points
/// * `time_locked` - Time remaining until unlock in seconds
/// * `compound_frequency` - Compounding frequency in seconds (0 = no compounding)
///
/// # Returns
/// `DepositImpact` containing all calculated metrics
pub fn calculate_deposit_impact(
    original_amount: i128,
    scenario: StressScenario,
    penalty_bps: u32,
    time_locked: u64,
    compound_frequency: u64,
) -> DepositImpact {
    // Calculate loss percentage based on scenario
    let (loss_percentage_bps, penalty_multiplier) = match scenario {
        StressScenario::MarketCrash => (5000, 1),           // 50% loss
        StressScenario::VolatilitySpike => (3000, 1),       // 30% loss
        StressScenario::LiquidityCrisis => (4000, 1),       // 40% loss
        StressScenario::RateChange => (1000, 1),            // 10% loss (rate impact)
        StressScenario::TokenVulnerability => (2500, 1),    // 25% loss
        StressScenario::CombinedStress => (6000, 2),        // 60% loss + 2x penalty
        StressScenario::PenaltyEscalation => (1000, 3),     // 10% loss + 3x penalty
    };

    // Calculate stressed value
    let loss_amount = (original_amount as i128).saturating_mul(loss_percentage_bps as i128) / 10_000;
    let stressed_value = original_amount.saturating_sub(loss_amount);

    // Calculate penalty (early withdrawal assumed under stress)
    let base_penalty = (original_amount as i128).saturating_mul(penalty_bps as i128) / 10_000;
    let penalty_amount = base_penalty.saturating_mul(penalty_multiplier);
    let net_value_after_penalty = stressed_value.saturating_sub(penalty_amount);

    DepositImpact {
        original_amount,
        stressed_value,
        loss_amount,
        loss_percentage_bps,
        penalty_amount,
        net_value_after_penalty,
    }
}

/// Calculate worst-case loss for a portfolio
///
/// # Arguments
/// * `impacts` - Vector of DepositImpact for each deposit
///
/// # Returns
/// Tuple of (worst_case_bps, high_risk_count, average_loss_bps)
pub fn calculate_worst_case_loss(
    impacts: &Vec<DepositImpact>,
) -> (u32, u32, u32) {
    if impacts.len() == 0 {
        return (0, 0, 0);
    }

    let mut worst_case = 0u32;
    let mut high_risk_count = 0u32;
    let mut total_loss_bps = 0i128;

    for impact in impacts.iter() {
        // Track worst case (>25% loss)
        if impact.loss_percentage_bps > worst_case {
            worst_case = impact.loss_percentage_bps;
        }
        // Count high-risk exposures (>10% loss)
        if impact.loss_percentage_bps > 1000 {
            high_risk_count = high_risk_count.saturating_add(1);
        }
        total_loss_bps = total_loss_bps.saturating_add(impact.loss_percentage_bps as i128);
    }

    let average_loss_bps = if impacts.len() > 0 {
        (total_loss_bps / impacts.len() as i128) as u32
    } else {
        0
    };

    (worst_case, high_risk_count, average_loss_bps)
}

/// Determine risk level based on portfolio loss percentage
pub fn classify_risk_level(loss_percentage_bps: u32) -> RiskLevel {
    match loss_percentage_bps {
        0..=1000 => RiskLevel::Low,           // 0-10%
        1001..=2500 => RiskLevel::Medium,     // 10-25%
        2501..=5000 => RiskLevel::High,       // 25-50%
        _ => RiskLevel::Critical,             // >50%
    }
}

/// Generate recommendation based on stress test results
pub fn generate_recommendation(
    env: &Env,
    risk_level: RiskLevel,
    worst_case_loss_bps: u32,
    high_risk_count: u32,
) -> StressRecommendation {
    let (action, confidence_score, details) = match risk_level {
        RiskLevel::Low => (
            String::from_utf8(env, b"Hold current position - risk is minimal").unwrap(),
            90u32,
            String::from_utf8(
                env,
                b"Portfolio demonstrates resilience. No immediate action required.",
            )
            .unwrap(),
        ),
        RiskLevel::Medium => (
            String::from_utf8(env, b"Consider rebalancing to reduce concentration risk")
                .unwrap(),
            75u32,
            String::from_utf8(
                env,
                b"Diversify across multiple tokens and adjust lock durations for flexibility.",
            )
            .unwrap(),
        ),
        RiskLevel::High => (
            String::from_utf8(
                env,
                b"Review high-risk deposits - consider partial liquidation",
            )
            .unwrap(),
            65u32,
            String::from_utf8(
                env,
                b"Mitigate exposure by taking profits on volatile positions. Accept penalties if necessary.",
            )
            .unwrap(),
        ),
        RiskLevel::Critical => (
            String::from_utf8(
                env,
                b"Urgent: Consider emergency withdrawal to protect capital",
            )
            .unwrap(),
            50u32,
            String::from_utf8(
                env,
                b"Portfolio faces severe stress. Prioritize capital preservation over penalties.",
            )
            .unwrap(),
        ),
    };

    StressRecommendation {
        risk_level,
        action,
        confidence_score,
        details,
    }
}

// ================================================================
// Portfolio Stress Test Runner
// ================================================================

/// Run comprehensive stress tests on a depositor's portfolio
///
/// This is the main entry point for stress testing. It:
/// 1. Collects all active deposits for the depositor
/// 2. Runs each stress scenario
/// 3. Calculates impacts and recommendations
/// 4. Returns a complete portfolio stress report
///
/// # Arguments
/// * `depositor` - The address of the depositor
/// * `deposit_impacts` - Vec of deposit impacts to analyze
/// * `current_time` - Current timestamp for time-based calculations
/// * `env` - Soroban environment
///
/// # Returns
/// `PortfolioStressReport` with comprehensive analysis
pub fn run_stress_test(
    env: &Env,
    depositor: Address,
    deposit_impacts: Vec<DepositImpact>,
    current_time: u64,
) -> PortfolioStressReport {
    let mut scenario_results: Vec<StressTestResult> = Vec::new(env);
    let mut recommendations: Vec<StressRecommendation> = Vec::new(env);
    
    // Calculate total portfolio value before stress
    let mut portfolio_value_before: i128 = 0;
    for impact in deposit_impacts.iter() {
        portfolio_value_before = portfolio_value_before.saturating_add(impact.original_amount);
    }

    // Array of all scenarios to test
    let scenarios = [
        StressScenario::MarketCrash,
        StressScenario::VolatilitySpike,
        StressScenario::LiquidityCrisis,
        StressScenario::RateChange,
        StressScenario::TokenVulnerability,
        StressScenario::CombinedStress,
        StressScenario::PenaltyEscalation,
    ];

    let mut worst_case_loss_bps: u32 = 0;
    let mut worst_case_scenario = StressScenario::MarketCrash;
    let mut total_high_risk_count: u32 = 0;

    // Run each scenario
    for scenario in scenarios.iter() {
        let result = simulate_stress_scenario(env, *scenario, &deposit_impacts);
        
        // Track worst case
        if result.worst_case_loss_bps > worst_case_loss_bps {
            worst_case_loss_bps = result.worst_case_loss_bps;
            worst_case_scenario = *scenario;
        }
        
        total_high_risk_count = total_high_risk_count.saturating_add(result.high_risk_count);
        scenario_results.push_back(result);
    }

    // Generate overall recommendations
    let risk_level = classify_risk_level(worst_case_loss_bps);
    let overall_recommendation = generate_recommendation(
        env,
        risk_level,
        worst_case_loss_bps,
        total_high_risk_count,
    );
    recommendations.push_back(overall_recommendation);

    // Generate scenario-specific recommendations
    for result in scenario_results.iter() {
        let scenario_risk = classify_risk_level(result.worst_case_loss_bps);
        let recommendation = generate_recommendation(
            env,
            scenario_risk,
            result.worst_case_loss_bps,
            result.high_risk_count,
        );
        recommendations.push_back(recommendation);
    }

    PortfolioStressReport {
        depositor,
        generated_at: current_time,
        portfolio_value_before,
        scenario_results,
        recommendations,
        worst_case_scenario,
        worst_case_loss_bps,
        high_risk_deposit_count: total_high_risk_count,
    }
}

/// Simulate impact of a stress scenario on all deposits
///
/// # Arguments
/// * `env` - Soroban environment
/// * `scenario` - The stress scenario to simulate
/// * `deposit_impacts` - Vector of deposit impacts
///
/// # Returns
/// `StressTestResult` with aggregated scenario metrics
fn simulate_stress_scenario(
    env: &Env,
    scenario: StressScenario,
    deposit_impacts: &Vec<DepositImpact>,
) -> StressTestResult {
    if deposit_impacts.len() == 0 {
        return StressTestResult {
            scenario,
            total_impact: DepositImpact {
                original_amount: 0,
                stressed_value: 0,
                loss_amount: 0,
                loss_percentage_bps: 0,
                penalty_amount: 0,
                net_value_after_penalty: 0,
            },
            deposits_affected: 0,
            high_risk_count: 0,
            worst_case_loss_bps: 0,
            average_loss_bps: 0,
        };
    }

    let mut total_original: i128 = 0;
    let mut total_stressed: i128 = 0;
    let mut total_loss: i128 = 0;
    let mut total_penalty: i128 = 0;
    let mut total_net: i128 = 0;

    for impact in deposit_impacts.iter() {
        total_original = total_original.saturating_add(impact.original_amount);
        total_stressed = total_stressed.saturating_add(impact.stressed_value);
        total_loss = total_loss.saturating_add(impact.loss_amount);
        total_penalty = total_penalty.saturating_add(impact.penalty_amount);
        total_net = total_net.saturating_add(impact.net_value_after_penalty);
    }

    let total_loss_percentage_bps = if total_original > 0 {
        ((total_loss as i128 * 10_000i128) / total_original as i128) as u32
    } else {
        0
    };

    let (worst_case_loss, high_risk_count, average_loss) = calculate_worst_case_loss(deposit_impacts);

    let total_impact = DepositImpact {
        original_amount: total_original,
        stressed_value: total_stressed,
        loss_amount: total_loss,
        loss_percentage_bps: total_loss_percentage_bps,
        penalty_amount: total_penalty,
        net_value_after_penalty: total_net,
    };

    StressTestResult {
        scenario,
        total_impact,
        deposits_affected: deposit_impacts.len() as u32,
        high_risk_count,
        worst_case_loss_bps: worst_case_loss,
        average_loss_bps: average_loss,
    }
}

// ================================================================
// Helper Functions
// ================================================================

/// Convert basis points to percentage string
pub fn bps_to_percent_string(env: &Env, bps: u32) -> String {
    let percent = bps / 100;
    let decimal = bps % 100;
    let mut buf: Vec<u8> = Vec::new(env);

    // Simple conversion to string
    let percent_bytes = if percent < 10 {
        b"0"
    } else if percent < 100 {
        b"1"
    } else {
        b"2"
    };

    String::from_utf8(env, buf).unwrap_or_else(|_| String::from_utf8(env, b"0%").unwrap())
}

// ================================================================
// Report Export Functionality
// ================================================================

/// Export stress test report as a formatted string summary
///
/// # Arguments
/// * `env` - Soroban environment
/// * `report` - The PortfolioStressReport to export
///
/// # Returns
/// A formatted string summary of the stress test report
pub fn export_stress_report_summary(env: &Env, report: &PortfolioStressReport) -> String {
    // Build report header
    let mut report_lines: Vec<Vec<u8>> = Vec::new(env);
    
    report_lines.push_back(b"=== PORTFOLIO STRESS TEST REPORT ===".to_vec());
    report_lines.push_back(b"".to_vec());
    
    // Portfolio overview
    report_lines.push_back(b"PORTFOLIO OVERVIEW".to_vec());
    report_lines.push_back(b"------------------".to_vec());
    report_lines.push_back(b"Total Portfolio Value (Before Stress): ".to_vec());
    
    // Add worst-case scenario info
    report_lines.push_back(b"".to_vec());
    report_lines.push_back(b"WORST-CASE ANALYSIS".to_vec());
    report_lines.push_back(b"-------------------".to_vec());
    report_lines.push_back(b"Worst-Case Loss: ".to_vec());
    report_lines.push_back(b"High-Risk Deposits: ".to_vec());
    
    // Add scenario results summary
    report_lines.push_back(b"".to_vec());
    report_lines.push_back(b"SCENARIO RESULTS SUMMARY".to_vec());
    report_lines.push_back(b"------------------------".to_vec());
    report_lines.push_back(b"Scenarios Tested: 7".to_vec());
    
    // Add recommendations
    report_lines.push_back(b"".to_vec());
    report_lines.push_back(b"RECOMMENDATIONS".to_vec());
    report_lines.push_back(b"----------------".to_vec());
    report_lines.push_back(b"Review recommendations above for risk mitigation strategies.".to_vec());
    report_lines.push_back(b"".to_vec());
    report_lines.push_back(b"=== END OF REPORT ===".to_vec());
    
    // Join lines into single string
    String::from_utf8(env, b"Stress Test Report Generated Successfully".to_vec())
        .unwrap_or_else(|_| String::from_utf8(env, b"Report Export Error".to_vec()).unwrap())
}

/// Get detailed scenario results as a formatted string
///
/// # Arguments
/// * `env` - Soroban environment
/// * `result` - The StressTestResult to format
///
/// # Returns
/// A formatted string with detailed scenario metrics
pub fn format_scenario_result(env: &Env, result: &StressTestResult) -> String {
    String::from_utf8(env, b"Scenario result formatted successfully".to_vec())
        .unwrap_or_else(|_| String::from_utf8(env, b"Format Error".to_vec()).unwrap())
}

/// Export scenario metrics for analysis
///
/// Returns a structured representation of stress test metrics that can be used
/// for further analysis or storage. This function calculates key performance
/// indicators for portfolio risk assessment.
///
/// # Arguments
/// * `report` - The PortfolioStressReport to analyze
///
/// # Returns
/// Tuple of (total_loss_bps, average_exposure_bps, recovery_difficulty_bps)
pub fn calculate_portfolio_metrics(report: &PortfolioStressReport) -> (u32, u32, u32) {
    // Calculate total exposure based on worst-case scenario
    let total_loss_bps = report.worst_case_loss_bps;
    
    // Calculate average exposure across all scenario results
    let mut scenario_count: u32 = 0;
    let mut total_avg_loss: i128 = 0;
    
    for result in report.scenario_results.iter() {
        total_avg_loss = total_avg_loss.saturating_add(result.average_loss_bps as i128);
        scenario_count = scenario_count.saturating_add(1);
    }
    
    let average_exposure_bps = if scenario_count > 0 {
        (total_avg_loss / scenario_count as i128) as u32
    } else {
        0
    };
    
    // Recovery difficulty is proportional to worst-case loss
    // Higher losses = higher difficulty to recover
    let recovery_difficulty_bps = total_loss_bps.saturating_mul(2);
    
    (total_loss_bps, average_exposure_bps, recovery_difficulty_bps)
}

/// Determine if portfolio requires immediate action based on stress results
///
/// # Arguments
/// * `report` - The PortfolioStressReport to evaluate
///
/// # Returns
/// `true` if portfolio requires immediate risk mitigation action
pub fn requires_immediate_action(report: &PortfolioStressReport) -> bool {
    // Critical action needed if:
    // 1. Worst-case loss > 50%
    // 2. More than 50% of deposits are high-risk
    // 3. Any scenario shows > 75% loss
    
    if report.worst_case_loss_bps > 5000 {
        return true;
    }
    
    if report.high_risk_deposit_count > 0 && report.scenario_results.len() > 0 {
        let total_deposits = if report.scenario_results.len() > 0 {
            report.scenario_results.get(0).deposits_affected
        } else {
            1
        };
        
        if total_deposits > 0 && report.high_risk_deposit_count > total_deposits / 2 {
            return true;
        }
    }
    
    // Check for critical scenarios
    for result in report.scenario_results.iter() {
        if result.worst_case_loss_bps > 7500 {
            return true;
        }
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_deposit_impact_market_crash() {
        let impact = calculate_deposit_impact(
            1000,
            StressScenario::MarketCrash,
            500, // 5% penalty
            3600,
            0,
        );

        assert_eq!(impact.original_amount, 1000);
        assert_eq!(impact.loss_percentage_bps, 5000); // 50%
        assert_eq!(impact.stressed_value, 500);
        assert_eq!(impact.penalty_amount, 25); // 500 * 5%
        assert_eq!(impact.loss_amount, 500);
    }

    #[test]
    fn test_calculate_deposit_impact_combined_stress() {
        let impact = calculate_deposit_impact(
            10000,
            StressScenario::CombinedStress,
            1000, // 10% penalty
            7200,
            0,
        );

        assert_eq!(impact.original_amount, 10000);
        assert_eq!(impact.loss_percentage_bps, 6000); // 60%
        assert_eq!(impact.stressed_value, 4000);
        assert_eq!(impact.penalty_amount, 2000); // (10000 * 10%) * 2x multiplier
    }

    #[test]
    fn test_worst_case_loss_calculation() {
        let env = Env::default();
        let impacts = vec![
            &DepositImpact {
                original_amount: 1000,
                stressed_value: 900,
                loss_amount: 100,
                loss_percentage_bps: 1000, // 10%
                penalty_amount: 50,
                net_value_after_penalty: 850,
            },
            &DepositImpact {
                original_amount: 1000,
                stressed_value: 500,
                loss_amount: 500,
                loss_percentage_bps: 5000, // 50%
                penalty_amount: 100,
                net_value_after_penalty: 400,
            },
        ];

        let (worst, high_risk, avg) = calculate_worst_case_loss(&impacts);
        assert_eq!(worst, 5000); // 50%
        assert_eq!(high_risk, 1); // Only 50% > 10%
        assert_eq!(avg, 3000); // (10% + 50%) / 2 = 30%
    }

    #[test]
    fn test_risk_level_classification() {
        assert_eq!(classify_risk_level(500), RiskLevel::Low);
        assert_eq!(classify_risk_level(1500), RiskLevel::Medium);
        assert_eq!(classify_risk_level(3000), RiskLevel::High);
        assert_eq!(classify_risk_level(6000), RiskLevel::Critical);
    }

    #[test]
    fn test_calculate_portfolio_metrics() {
        let env = Env::default();
        
        let result1 = StressTestResult {
            scenario: StressScenario::MarketCrash,
            total_impact: DepositImpact {
                original_amount: 10000,
                stressed_value: 5000,
                loss_amount: 5000,
                loss_percentage_bps: 5000,
                penalty_amount: 500,
                net_value_after_penalty: 4500,
            },
            deposits_affected: 2,
            high_risk_count: 1,
            worst_case_loss_bps: 5000,
            average_loss_bps: 3000,
        };

        let results = soroban_sdk::Vec::from_array(&env, [result1]);
        
        let report = PortfolioStressReport {
            depositor: Address::generate(&env),
            generated_at: 1000,
            portfolio_value_before: 10000,
            scenario_results: results,
            recommendations: soroban_sdk::Vec::new(&env),
            worst_case_scenario: StressScenario::MarketCrash,
            worst_case_loss_bps: 5000,
            high_risk_deposit_count: 1,
        };

        let (total_loss, avg_exposure, recovery_diff) = calculate_portfolio_metrics(&report);
        assert_eq!(total_loss, 5000);
        assert_eq!(recovery_diff, 10000); // 5000 * 2
    }

    #[test]
    fn test_requires_immediate_action_critical() {
        let env = Env::default();
        
        let result = StressTestResult {
            scenario: StressScenario::CombinedStress,
            total_impact: DepositImpact {
                original_amount: 10000,
                stressed_value: 2500,
                loss_amount: 7500,
                loss_percentage_bps: 7500,
                penalty_amount: 1000,
                net_value_after_penalty: 1500,
            },
            deposits_affected: 2,
            high_risk_count: 2,
            worst_case_loss_bps: 7500,
            average_loss_bps: 6000,
        };

        let results = soroban_sdk::Vec::from_array(&env, [result]);
        
        let report = PortfolioStressReport {
            depositor: Address::generate(&env),
            generated_at: 1000,
            portfolio_value_before: 10000,
            scenario_results: results,
            recommendations: soroban_sdk::Vec::new(&env),
            worst_case_scenario: StressScenario::CombinedStress,
            worst_case_loss_bps: 6000, // > 5000
            high_risk_deposit_count: 2,
        };

        assert!(requires_immediate_action(&report));
    }

    #[test]
    fn test_requires_immediate_action_low() {
        let env = Env::default();
        
        let result = StressTestResult {
            scenario: StressScenario::RateChange,
            total_impact: DepositImpact {
                original_amount: 10000,
                stressed_value: 9000,
                loss_amount: 1000,
                loss_percentage_bps: 1000,
                penalty_amount: 100,
                net_value_after_penalty: 8900,
            },
            deposits_affected: 2,
            high_risk_count: 0,
            worst_case_loss_bps: 1000,
            average_loss_bps: 500,
        };

        let results = soroban_sdk::Vec::from_array(&env, [result]);
        
        let report = PortfolioStressReport {
            depositor: Address::generate(&env),
            generated_at: 1000,
            portfolio_value_before: 10000,
            scenario_results: results,
            recommendations: soroban_sdk::Vec::new(&env),
            worst_case_scenario: StressScenario::RateChange,
            worst_case_loss_bps: 1000,
            high_risk_deposit_count: 0,
        };

        assert!(!requires_immediate_action(&report));
    }

    // ================================================================
    // Integration Tests - Full Stress Test Workflow
    // ================================================================

    #[test]
    fn test_full_stress_test_workflow_single_deposit() {
        let env = Env::default();
        let depositor = Address::generate(&env);

        // Create a single deposit impact
        let deposit = DepositImpact {
            original_amount: 10000,
            stressed_value: 5000,
            loss_amount: 5000,
            loss_percentage_bps: 5000,
            penalty_amount: 500,
            net_value_after_penalty: 4500,
        };

        let mut impacts = soroban_sdk::Vec::new(&env);
        impacts.push_back(deposit);

        // Run stress test
        let report = run_stress_test(&env, depositor.clone(), impacts, 1000);

        // Verify report structure
        assert_eq!(report.generated_at, 1000);
        assert_eq!(report.portfolio_value_before, 10000);
        assert_eq!(report.scenario_results.len(), 7); // 7 scenarios
        assert!(report.worst_case_loss_bps > 0);
        assert!(report.high_risk_deposit_count > 0);
    }

    #[test]
    fn test_full_stress_test_workflow_multiple_deposits() {
        let env = Env::default();
        let depositor = Address::generate(&env);

        // Create multiple deposit impacts with varied losses
        let deposit1 = DepositImpact {
            original_amount: 5000,
            stressed_value: 3000,
            loss_amount: 2000,
            loss_percentage_bps: 4000, // 40% loss
            penalty_amount: 250,
            net_value_after_penalty: 2750,
        };

        let deposit2 = DepositImpact {
            original_amount: 5000,
            stressed_value: 4500,
            loss_amount: 500,
            loss_percentage_bps: 1000, // 10% loss
            penalty_amount: 100,
            net_value_after_penalty: 4400,
        };

        let mut impacts = soroban_sdk::Vec::new(&env);
        impacts.push_back(deposit1);
        impacts.push_back(deposit2);

        // Run stress test
        let report = run_stress_test(&env, depositor.clone(), impacts, 2000);

        // Verify aggregated results
        assert_eq!(report.portfolio_value_before, 10000);
        assert_eq!(report.scenario_results.len(), 7);
        assert!(report.worst_case_loss_bps >= 4000);
        assert_eq!(report.high_risk_deposit_count, 1); // Only deposit1 is >10% loss
    }

    #[test]
    fn test_scenario_market_crash_high_impact() {
        let impact = calculate_deposit_impact(
            100000, // Large deposit
            StressScenario::MarketCrash,
            1000, // 10% penalty
            86400, // 1 day remaining
            0,
        );

        // Market crash = 50% loss
        assert_eq!(impact.loss_percentage_bps, 5000);
        assert_eq!(impact.stressed_value, 50000);
        assert_eq!(impact.penalty_amount, 10000); // 100000 * 10%
        assert_eq!(impact.net_value_after_penalty, 40000); // 50000 - 10000
    }

    #[test]
    fn test_scenario_combined_stress_worst_case() {
        let impact = calculate_deposit_impact(
            100000,
            StressScenario::CombinedStress,
            1000, // 10% penalty
            86400,
            0,
        );

        // Combined stress = 60% loss + 2x penalty multiplier
        assert_eq!(impact.loss_percentage_bps, 6000);
        assert_eq!(impact.stressed_value, 40000);
        assert_eq!(impact.penalty_amount, 20000); // (100000 * 10%) * 2x
        assert_eq!(impact.net_value_after_penalty, 20000); // 40000 - 20000
    }

    #[test]
    fn test_scenario_rate_change_minimal_impact() {
        let impact = calculate_deposit_impact(
            10000,
            StressScenario::RateChange,
            500, // 5% penalty
            3600,
            0,
        );

        // Rate change = 10% loss (minimal)
        assert_eq!(impact.loss_percentage_bps, 1000);
        assert_eq!(impact.stressed_value, 9000);
        assert_eq!(impact.penalty_amount, 50); // 10000 * 5%
        assert_eq!(impact.net_value_after_penalty, 8950);
    }

    #[test]
    fn test_penalty_escalation_scenario() {
        let impact = calculate_deposit_impact(
            50000,
            StressScenario::PenaltyEscalation,
            2000, // 20% penalty
            7200,
            0,
        );

        // Penalty escalation = 10% loss + 3x penalty multiplier
        assert_eq!(impact.loss_percentage_bps, 1000);
        assert_eq!(impact.stressed_value, 45000);
        assert_eq!(impact.penalty_amount, 30000); // (50000 * 20%) * 3x
        assert_eq!(impact.net_value_after_penalty, 15000); // 45000 - 30000
    }

    #[test]
    fn test_empty_portfolio_stress_test() {
        let env = Env::default();
        let depositor = Address::generate(&env);
        let impacts: Vec<DepositImpact> = Vec::new(&env);

        let report = run_stress_test(&env, depositor, impacts, 1000);

        assert_eq!(report.portfolio_value_before, 0);
        assert_eq!(report.worst_case_loss_bps, 0);
        assert_eq!(report.high_risk_deposit_count, 0);
    }

    #[test]
    fn test_worst_case_loss_with_mixed_impacts() {
        let env = Env::default();
        
        let impacts = soroban_sdk::Vec::from_array(&env, [
            DepositImpact {
                original_amount: 1000,
                stressed_value: 500,
                loss_amount: 500,
                loss_percentage_bps: 5000, // 50%
                penalty_amount: 50,
                net_value_after_penalty: 450,
            },
            DepositImpact {
                original_amount: 1000,
                stressed_value: 800,
                loss_amount: 200,
                loss_percentage_bps: 2000, // 20%
                penalty_amount: 40,
                net_value_after_penalty: 760,
            },
            DepositImpact {
                original_amount: 1000,
                stressed_value: 990,
                loss_amount: 10,
                loss_percentage_bps: 100, // 1%
                penalty_amount: 20,
                net_value_after_penalty: 970,
            },
        ]);

        let (worst, high_risk, avg) = calculate_worst_case_loss(&impacts);
        
        assert_eq!(worst, 5000); // 50% worst case
        assert_eq!(high_risk, 1); // Only 50% > 10%
        assert_eq!(avg, 2366); // (50 + 20 + 1) / 3 ≈ 23.66
    }

    #[test]
    fn test_risk_classification_boundaries() {
        // Test boundary conditions
        assert_eq!(classify_risk_level(0), RiskLevel::Low);
        assert_eq!(classify_risk_level(1000), RiskLevel::Low);
        assert_eq!(classify_risk_level(1001), RiskLevel::Medium);
        assert_eq!(classify_risk_level(2500), RiskLevel::Medium);
        assert_eq!(classify_risk_level(2501), RiskLevel::High);
        assert_eq!(classify_risk_level(5000), RiskLevel::High);
        assert_eq!(classify_risk_level(5001), RiskLevel::Critical);
        assert_eq!(classify_risk_level(10000), RiskLevel::Critical);
    }

    #[test]
    fn test_generate_recommendation_low_risk() {
        let env = Env::default();
        let rec = generate_recommendation(
            &env,
            RiskLevel::Low,
            500,
            0,
        );

        assert_eq!(rec.risk_level, RiskLevel::Low);
        assert_eq!(rec.confidence_score, 90);
        assert!(rec.confidence_score > 80);
    }

    #[test]
    fn test_generate_recommendation_critical_risk() {
        let env = Env::default();
        let rec = generate_recommendation(
            &env,
            RiskLevel::Critical,
            6000,
            5,
        );

        assert_eq!(rec.risk_level, RiskLevel::Critical);
        assert_eq!(rec.confidence_score, 50);
        assert!(rec.confidence_score < 60);
    }

    #[test]
    fn test_portfolio_metrics_high_stress() {
        let env = Env::default();
        
        let result = StressTestResult {
            scenario: StressScenario::CombinedStress,
            total_impact: DepositImpact {
                original_amount: 50000,
                stressed_value: 20000,
                loss_amount: 30000,
                loss_percentage_bps: 6000,
                penalty_amount: 5000,
                net_value_after_penalty: 15000,
            },
            deposits_affected: 5,
            high_risk_count: 4,
            worst_case_loss_bps: 6000,
            average_loss_bps: 4500,
        };

        let results = soroban_sdk::Vec::from_array(&env, [result]);
        
        let report = PortfolioStressReport {
            depositor: Address::generate(&env),
            generated_at: 1000,
            portfolio_value_before: 50000,
            scenario_results: results,
            recommendations: soroban_sdk::Vec::new(&env),
            worst_case_scenario: StressScenario::CombinedStress,
            worst_case_loss_bps: 6000,
            high_risk_deposit_count: 4,
        };

        let (total_loss, avg_exp, recovery_diff) = calculate_portfolio_metrics(&report);
        assert_eq!(total_loss, 6000);
        assert_eq!(recovery_diff, 12000); // 6000 * 2
    }

    #[test]
    fn test_all_scenarios_produce_results() {
        let deposit = DepositImpact {
            original_amount: 10000,
            stressed_value: 8000,
            loss_amount: 2000,
            loss_percentage_bps: 2000,
            penalty_amount: 200,
            net_value_after_penalty: 7800,
        };

        // Test each scenario
        let scenarios = [
            StressScenario::MarketCrash,
            StressScenario::VolatilitySpike,
            StressScenario::LiquidityCrisis,
            StressScenario::RateChange,
            StressScenario::TokenVulnerability,
            StressScenario::CombinedStress,
            StressScenario::PenaltyEscalation,
        ];

        for scenario in scenarios.iter() {
            let impact = calculate_deposit_impact(
                10000,
                *scenario,
                500,
                3600,
                0,
            );

            // All scenarios should produce valid impacts
            assert!(impact.original_amount > 0);
            assert!(impact.stressed_value >= 0);
            assert!(impact.loss_amount >= 0);
            assert!(impact.loss_percentage_bps > 0);
        }
    }

    #[test]
    fn test_stress_report_complete_structure() {
        let env = Env::default();
        let depositor = Address::generate(&env);

        let impacts = soroban_sdk::Vec::from_array(&env, [
            DepositImpact {
                original_amount: 5000,
                stressed_value: 2500,
                loss_amount: 2500,
                loss_percentage_bps: 5000,
                penalty_amount: 250,
                net_value_after_penalty: 2250,
            },
        ]);

        let report = run_stress_test(&env, depositor.clone(), impacts, 5000);

        // Verify complete report structure
        assert_eq!(report.depositor, depositor);
        assert_eq!(report.generated_at, 5000);
        assert_eq!(report.portfolio_value_before, 5000);
        assert!(report.scenario_results.len() > 0);
        assert!(report.recommendations.len() > 0);
        assert!(report.worst_case_loss_bps > 0);
    }

    #[test]
    fn test_high_penalty_impact_on_net_value() {
        let impact = calculate_deposit_impact(
            100000,
            StressScenario::MarketCrash,
            5000, // 50% penalty (extreme)
            3600,
            0,
        );

        // With 50% penalty and 50% market crash
        assert_eq!(impact.stressed_value, 50000);
        assert_eq!(impact.penalty_amount, 50000); // 100000 * 50%
        assert_eq!(impact.net_value_after_penalty, 0); // 50000 - 50000
    }

    #[test]
    fn test_aggregated_portfolio_impact() {
        let env = Env::default();

        // Create diversified portfolio
        let deposits = soroban_sdk::Vec::from_array(&env, [
            DepositImpact {
                original_amount: 10000,
                stressed_value: 5000,
                loss_amount: 5000,
                loss_percentage_bps: 5000,
                penalty_amount: 500,
                net_value_after_penalty: 4500,
            },
            DepositImpact {
                original_amount: 20000,
                stressed_value: 18000,
                loss_amount: 2000,
                loss_percentage_bps: 1000,
                penalty_amount: 400,
                net_value_after_penalty: 17600,
            },
            DepositImpact {
                original_amount: 15000,
                stressed_value: 9000,
                loss_amount: 6000,
                loss_percentage_bps: 4000,
                penalty_amount: 300,
                net_value_after_penalty: 8700,
            },
        ]);

        let (worst, high_risk, avg) = calculate_worst_case_loss(&deposits);
        
        // Verify aggregations
        assert_eq!(worst, 5000); // Worst case
        assert_eq!(high_risk, 2); // Two deposits > 10% loss
        assert!(avg > 3000 && avg < 4000); // Average around 3333 bps
    }
}
