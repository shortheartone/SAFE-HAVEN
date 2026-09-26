// ============================================================
//  Yield Farming Integration Tests
//  Issue: #XXX
// ============================================================

#[cfg(test)]
mod yield_farming_tests {
    use crate::*;
    use soroban_sdk::{testutils::*, vec, Address, Env, String};

    fn setup() -> (Env, Address, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        env.budget().reset_unlimited();

        let admin = Address::random(&env);
        let depositor = Address::random(&env);
        let token = Address::random(&env);
        let protocol = Address::random(&env);

        let contract_id = env.register_contract(None, contract::SafeHaven);
        let client = contract::SafeHavenClient::new(&env, &contract_id);

        // Initialize contract
        client.initialize(
            &admin,
            &admin,
            &1_000_000_000_000_000,
            &157_788_000,
        );

        // Register token
        register_test_token(&env, &token);

        // Mint tokens to depositor
        give_tokens(&env, &token, &depositor, &1_000_000_000);

        (env, admin, depositor, token, protocol)
    }

    fn register_test_token(env: &Env, token: &Address) {
        let contract_id = env.register_stellar_asset_contract(token.clone());
        // Asset is ready to use
    }

    fn give_tokens(env: &Env, token: &Address, to: &Address, amount: &i128) {
        let token_client = token::Client::new(env, token);
        token_client.mint(to, amount);
    }

    fn advance_time(env: &Env, seconds: u64) {
        env.ledger().with_mut(|ledger| {
            ledger.timestamp = ledger.timestamp.saturating_add(seconds);
        });
    }

    // ================================================================
    //  Test: Farming Configuration
    // ================================================================

    #[test]
    fn test_init_yield_farming_success() {
        let (env, admin, _depositor, _token, _protocol) = setup();
        let contract_id = env.register_contract(None, contract::SafeHaven);
        let client = contract::SafeHavenClient::new(&env, &contract_id);

        client.initialize(&admin, &admin, &1_000_000_000_000_000, &157_788_000);
        let result = client.init_yield_farming(&admin);

        assert!(result.is_ok());

        // Verify config was created
        let config = client.get_farming_config();
        assert!(config.is_some());
        let cfg = config.unwrap();
        assert!(cfg.enabled);
        assert_eq!(cfg.min_farming_amount, crate::constants::MIN_FARMING_AMOUNT);
        assert_eq!(cfg.max_farming_proportion_bps, crate::constants::MAX_FARMING_PROPORTION_BPS);
        assert_eq!(cfg.risk_level, crate::constants::FARMING_RISK_LEVEL);
    }

    #[test]
    fn test_add_farming_protocol_success() {
        let (env, admin, _depositor, _token, protocol) = setup();
        let contract_id = env.register_contract(None, contract::SafeHaven);
        let client = contract::SafeHavenClient::new(&env, &contract_id);

        client.initialize(&admin, &admin, &1_000_000_000_000_000, &157_788_000);
        client.init_yield_farming(&admin).unwrap();

        let result = client.add_farming_protocol(&admin, &protocol);
        assert!(result.is_ok());

        let config = client.get_farming_config().unwrap();
        assert_eq!(config.approved_protocols.len(), 1);
    }

    #[test]
    fn test_add_farming_protocol_duplicate_rejected() {
        let (env, admin, _depositor, _token, protocol) = setup();
        let contract_id = env.register_contract(None, contract::SafeHaven);
        let client = contract::SafeHavenClient::new(&env, &contract_id);

        client.initialize(&admin, &admin, &1_000_000_000_000_000, &157_788_000);
        client.init_yield_farming(&admin).unwrap();
        client.add_farming_protocol(&admin, &protocol).unwrap();

        // Try adding same protocol again
        let result = client.add_farming_protocol(&admin, &protocol);
        assert!(result.is_err());
    }

    // ================================================================
    //  Test: Enable Farming
    // ================================================================

    #[test]
    fn test_enable_farming_success() {
        let (env, admin, depositor, token, protocol) = setup();
        let contract_id = env.register_contract(None, contract::SafeHaven);
        let client = contract::SafeHavenClient::new(&env, &contract_id);

        client.initialize(&admin, &admin, &1_000_000_000_000_000, &157_788_000);
        client.init_yield_farming(&admin).unwrap();
        client.add_farming_protocol(&admin, &protocol).unwrap();

        // Create a deposit
        let unlock_time = env.ledger().timestamp() + 86400;
        let deposit_id = client.deposit(&depositor, &token, &100_000_000, &unlock_time, &0);

        // Enable farming
        let result = client.enable_farming(&depositor, &deposit_id, &0u8, &protocol);
        assert!(result.is_ok());

        // Verify farming state
        let (enabled, deployed, rewards) = client.get_farming_info(&depositor, &deposit_id).unwrap();
        assert!(enabled);
        assert_eq!(deployed, 90_000_000); // 90% of 100M
        assert_eq!(rewards, 0);
    }

    #[test]
    fn test_enable_farming_below_minimum_fails() {
        let (env, admin, depositor, token, protocol) = setup();
        let contract_id = env.register_contract(None, contract::SafeHaven);
        let client = contract::SafeHavenClient::new(&env, &contract_id);

        client.initialize(&admin, &admin, &1_000_000_000_000_000, &157_788_000);
        client.init_yield_farming(&admin).unwrap();
        client.add_farming_protocol(&admin, &protocol).unwrap();

        // Create a small deposit
        let unlock_time = env.ledger().timestamp() + 86400;
        let deposit_id = client.deposit(&depositor, &token, &100, &unlock_time, &0);

        // Try to enable farming with insufficient amount
        let result = client.enable_farming(&depositor, &deposit_id, &0u8, &protocol);
        assert!(result.is_err());
    }

    #[test]
    fn test_enable_farming_unapproved_protocol_fails() {
        let (env, admin, depositor, token, _protocol) = setup();
        let contract_id = env.register_contract(None, contract::SafeHaven);
        let client = contract::SafeHavenClient::new(&env, &contract_id);

        client.initialize(&admin, &admin, &1_000_000_000_000_000, &157_788_000);
        client.init_yield_farming(&admin).unwrap();

        // Don't add the protocol
        let unlock_time = env.ledger().timestamp() + 86400;
        let deposit_id = client.deposit(&depositor, &token, &100_000_000, &unlock_time, &0);

        let unapproved_protocol = Address::random(&env);
        let result = client.enable_farming(&depositor, &deposit_id, &0u8, &unapproved_protocol);
        assert!(result.is_err());
    }

    #[test]
    fn test_enable_farming_already_enabled_fails() {
        let (env, admin, depositor, token, protocol) = setup();
        let contract_id = env.register_contract(None, contract::SafeHaven);
        let client = contract::SafeHavenClient::new(&env, &contract_id);

        client.initialize(&admin, &admin, &1_000_000_000_000_000, &157_788_000);
        client.init_yield_farming(&admin).unwrap();
        client.add_farming_protocol(&admin, &protocol).unwrap();

        let unlock_time = env.ledger().timestamp() + 86400;
        let deposit_id = client.deposit(&depositor, &token, &100_000_000, &unlock_time, &0);

        client.enable_farming(&depositor, &deposit_id, &0u8, &protocol).unwrap();

        // Try enabling again
        let result = client.enable_farming(&depositor, &deposit_id, &0u8, &protocol);
        assert!(result.is_err());
    }

    // ================================================================
    //  Test: Claim Rewards
    // ================================================================

    #[test]
    fn test_claim_farming_rewards_accumulates() {
        let (env, admin, depositor, token, protocol) = setup();
        let contract_id = env.register_contract(None, contract::SafeHaven);
        let client = contract::SafeHavenClient::new(&env, &contract_id);

        client.initialize(&admin, &admin, &1_000_000_000_000_000, &157_788_000);
        client.init_yield_farming(&admin).unwrap();
        client.add_farming_protocol(&admin, &protocol).unwrap();

        let unlock_time = env.ledger().timestamp() + 86400;
        let deposit_id = client.deposit(&depositor, &token, &100_000_000, &unlock_time, &0);

        client.enable_farming(&depositor, &deposit_id, &0u8, &protocol).unwrap();

        // Advance time by 1 year
        advance_time(&env, 365 * 24 * 60 * 60);

        // Claim rewards
        let claimed = client.claim_farming_rewards(&depositor, &deposit_id).unwrap();

        // Expected: 100M * 0.03 (3% annual) = 3M
        // Deployed: 90M * 0.03 = 2.7M
        assert!(claimed > 0);

        // Verify total claimed is tracked
        let total_claimed = client.get_total_rewards_claimed(&depositor);
        assert_eq!(total_claimed, claimed);
    }

    #[test]
    fn test_claim_farming_rewards_no_farming_fails() {
        let (env, admin, depositor, token, _protocol) = setup();
        let contract_id = env.register_contract(None, contract::SafeHaven);
        let client = contract::SafeHavenClient::new(&env, &contract_id);

        client.initialize(&admin, &admin, &1_000_000_000_000_000, &157_788_000);

        let unlock_time = env.ledger().timestamp() + 86400;
        let deposit_id = client.deposit(&depositor, &token, &100_000_000, &unlock_time, &0);

        // Try claiming without farming enabled
        let result = client.claim_farming_rewards(&depositor, &deposit_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_claim_farming_rewards_no_accrual_yet() {
        let (env, admin, depositor, token, protocol) = setup();
        let contract_id = env.register_contract(None, contract::SafeHaven);
        let client = contract::SafeHavenClient::new(&env, &contract_id);

        client.initialize(&admin, &admin, &1_000_000_000_000_000, &157_788_000);
        client.init_yield_farming(&admin).unwrap();
        client.add_farming_protocol(&admin, &protocol).unwrap();

        let unlock_time = env.ledger().timestamp() + 86400;
        let deposit_id = client.deposit(&depositor, &token, &100_000_000, &unlock_time, &0);

        client.enable_farming(&depositor, &deposit_id, &0u8, &protocol).unwrap();

        // Claim immediately - no time passed
        let claimed = client.claim_farming_rewards(&depositor, &deposit_id).unwrap();
        assert_eq!(claimed, 0);
    }

    #[test]
    fn test_claim_multiple_times() {
        let (env, admin, depositor, token, protocol) = setup();
        let contract_id = env.register_contract(None, contract::SafeHaven);
        let client = contract::SafeHavenClient::new(&env, &contract_id);

        client.initialize(&admin, &admin, &1_000_000_000_000_000, &157_788_000);
        client.init_yield_farming(&admin).unwrap();
        client.add_farming_protocol(&admin, &protocol).unwrap();

        let unlock_time = env.ledger().timestamp() + 86400;
        let deposit_id = client.deposit(&depositor, &token, &100_000_000, &unlock_time, &0);

        client.enable_farming(&depositor, &deposit_id, &0u8, &protocol).unwrap();

        // Advance 6 months and claim
        advance_time(&env, 180 * 24 * 60 * 60);
        let first_claim = client.claim_farming_rewards(&depositor, &deposit_id).unwrap();

        // Advance another 6 months and claim again
        advance_time(&env, 180 * 24 * 60 * 60);
        let second_claim = client.claim_farming_rewards(&depositor, &deposit_id).unwrap();

        // Each should be roughly similar (3% annual / 2 periods)
        assert!(first_claim > 0);
        assert!(second_claim > 0);

        let total = client.get_total_rewards_claimed(&depositor);
        assert_eq!(total, first_claim + second_claim);
    }

    // ================================================================
    //  Test: Disable Farming
    // ================================================================

    #[test]
    fn test_disable_farming_success() {
        let (env, admin, depositor, token, protocol) = setup();
        let contract_id = env.register_contract(None, contract::SafeHaven);
        let client = contract::SafeHavenClient::new(&env, &contract_id);

        client.initialize(&admin, &admin, &1_000_000_000_000_000, &157_788_000);
        client.init_yield_farming(&admin).unwrap();
        client.add_farming_protocol(&admin, &protocol).unwrap();

        let unlock_time = env.ledger().timestamp() + 86400;
        let deposit_id = client.deposit(&depositor, &token, &100_000_000, &unlock_time, &0);

        client.enable_farming(&depositor, &deposit_id, &0u8, &protocol).unwrap();

        // Advance time and disable
        advance_time(&env, 30 * 24 * 60 * 60);
        let (deployed, rewards) = client.disable_farming(&depositor, &deposit_id).unwrap();

        assert_eq!(deployed, 90_000_000); // Was deployed amount
        assert!(rewards >= 0);

        // Verify farming is disabled
        let (enabled, _, _) = client.get_farming_info(&depositor, &deposit_id).unwrap();
        assert!(!enabled);
    }

    #[test]
    fn test_disable_farming_not_enabled_fails() {
        let (env, admin, depositor, token, _protocol) = setup();
        let contract_id = env.register_contract(None, contract::SafeHaven);
        let client = contract::SafeHavenClient::new(&env, &contract_id);

        client.initialize(&admin, &admin, &1_000_000_000_000_000, &157_788_000);

        let unlock_time = env.ledger().timestamp() + 86400;
        let deposit_id = client.deposit(&depositor, &token, &100_000_000, &unlock_time, &0);

        // Try disabling without farming enabled
        let result = client.disable_farming(&depositor, &deposit_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_disable_farming_finalizes_rewards() {
        let (env, admin, depositor, token, protocol) = setup();
        let contract_id = env.register_contract(None, contract::SafeHaven);
        let client = contract::SafeHavenClient::new(&env, &contract_id);

        client.initialize(&admin, &admin, &1_000_000_000_000_000, &157_788_000);
        client.init_yield_farming(&admin).unwrap();
        client.add_farming_protocol(&admin, &protocol).unwrap();

        let unlock_time = env.ledger().timestamp() + 86400;
        let deposit_id = client.deposit(&depositor, &token, &100_000_000, &unlock_time, &0);

        client.enable_farming(&depositor, &deposit_id, &0u8, &protocol).unwrap();

        // Advance 1 year
        advance_time(&env, 365 * 24 * 60 * 60);

        // Disable and get final rewards
        let (_, final_rewards) = client.disable_farming(&depositor, &deposit_id).unwrap();

        // Should have accumulated rewards
        assert!(final_rewards > 0);

        let total_claimed = client.get_total_rewards_claimed(&depositor);
        assert_eq!(total_claimed, final_rewards);
    }

    // ================================================================
    //  Test: Principal Protection
    // ================================================================

    #[test]
    fn test_farming_does_not_affect_deposit_withdrawal() {
        let (env, admin, depositor, token, protocol) = setup();
        let contract_id = env.register_contract(None, contract::SafeHaven);
        let client = contract::SafeHavenClient::new(&env, &contract_id);

        client.initialize(&admin, &admin, &1_000_000_000_000_000, &157_788_000);
        client.init_yield_farming(&admin).unwrap();
        client.add_farming_protocol(&admin, &protocol).unwrap();

        let original_amount = 100_000_000i128;
        let unlock_time = env.ledger().timestamp() + 86400;
        let deposit_id = client.deposit(&depositor, &token, &original_amount, &unlock_time, &0);

        client.enable_farming(&depositor, &deposit_id, &0u8, &protocol).unwrap();

        // Advance time
        advance_time(&env, 180 * 24 * 60 * 60);

        // Claim some rewards (not withdrawal, just claim)
        client.claim_farming_rewards(&depositor, &deposit_id).unwrap();

        // Time to unlock
        advance_time(&env, 86400);

        // Withdraw - should get full original amount, not affected by farming
        client.withdraw(&depositor, &token, &original_amount, &deposit_id);

        // Verify principal is intact
        let vault = client.get_vault(&depositor, &deposit_id);
        assert!(vault.is_none()); // Withdrawn
    }

    #[test]
    fn test_farming_deployment_respects_max_proportion() {
        let (env, admin, depositor, token, protocol) = setup();
        let contract_id = env.register_contract(None, contract::SafeHaven);
        let client = contract::SafeHavenClient::new(&env, &contract_id);

        client.initialize(&admin, &admin, &1_000_000_000_000_000, &157_788_000);
        client.init_yield_farming(&admin).unwrap();
        client.add_farming_protocol(&admin, &protocol).unwrap();

        let original_amount = 100_000_000i128;
        let unlock_time = env.ledger().timestamp() + 86400;
        let deposit_id = client.deposit(&depositor, &token, &original_amount, &unlock_time, &0);

        client.enable_farming(&depositor, &deposit_id, &0u8, &protocol).unwrap();

        let (_, deployed, _) = client.get_farming_info(&depositor, &deposit_id).unwrap();

        // Max proportion is 90% (9000 bps)
        let max_deployable = (original_amount as u128 * 9000) / 10000;
        assert_eq!(deployed as u128, max_deployable);
    }

    // ================================================================
    //  Test: Multiple Strategies
    // ================================================================

    #[test]
    fn test_different_farming_strategies() {
        let (env, admin, depositor, token, _protocol) = setup();
        let contract_id = env.register_contract(None, contract::SafeHaven);
        let client = contract::SafeHavenClient::new(&env, &contract_id);

        client.initialize(&admin, &admin, &1_000_000_000_000_000, &157_788_000);
        client.init_yield_farming(&admin).unwrap();

        // Add multiple protocols
        let protocol1 = Address::random(&env);
        let protocol2 = Address::random(&env);
        let protocol3 = Address::random(&env);

        client.add_farming_protocol(&admin, &protocol1).unwrap();
        client.add_farming_protocol(&admin, &protocol2).unwrap();
        client.add_farming_protocol(&admin, &protocol3).unwrap();

        // Create multiple deposits with different strategies
        let unlock_time = env.ledger().timestamp() + 86400;
        let amount = 100_000_000i128;

        let dep1 = client.deposit(&depositor, &token, &amount, &unlock_time, &0);
        let dep2 = client.deposit(&depositor, &token, &amount, &unlock_time, &0);
        let dep3 = client.deposit(&depositor, &token, &amount, &unlock_time, &0);

        // Enable different strategies
        client.enable_farming(&depositor, &dep1, &0u8, &protocol1).unwrap(); // DirectStaking
        client.enable_farming(&depositor, &dep2, &1u8, &protocol2).unwrap(); // LiquidityProvision
        client.enable_farming(&depositor, &dep3, &2u8, &protocol3).unwrap(); // LendingYield

        // All should be enabled
        let (enabled1, _, _) = client.get_farming_info(&depositor, &dep1).unwrap();
        let (enabled2, _, _) = client.get_farming_info(&depositor, &dep2).unwrap();
        let (enabled3, _, _) = client.get_farming_info(&depositor, &dep3).unwrap();

        assert!(enabled1);
        assert!(enabled2);
        assert!(enabled3);
    }

    // ================================================================
    //  Test: Auditing and Tracking
    // ================================================================

    #[test]
    fn test_total_rewards_claimed_tracking() {
        let (env, admin, depositor, token, protocol) = setup();
        let contract_id = env.register_contract(None, contract::SafeHaven);
        let client = contract::SafeHavenClient::new(&env, &contract_id);

        client.initialize(&admin, &admin, &1_000_000_000_000_000, &157_788_000);
        client.init_yield_farming(&admin).unwrap();
        client.add_farming_protocol(&admin, &protocol).unwrap();

        let unlock_time = env.ledger().timestamp() + 86400 * 2;
        let dep1 = client.deposit(&depositor, &token, &100_000_000, &unlock_time, &0);
        let dep2 = client.deposit(&depositor, &token, &100_000_000, &unlock_time, &0);

        client.enable_farming(&depositor, &dep1, &0u8, &protocol).unwrap();
        client.enable_farming(&depositor, &dep2, &0u8, &protocol).unwrap();

        advance_time(&env, 365 * 24 * 60 * 60);

        client.claim_farming_rewards(&depositor, &dep1).unwrap();
        client.claim_farming_rewards(&depositor, &dep2).unwrap();

        let total_claimed = client.get_total_rewards_claimed(&depositor);
        assert!(total_claimed > 0);
    }

    #[test]
    fn test_contract_paused_prevents_farming() {
        let (env, admin, depositor, token, protocol) = setup();
        let contract_id = env.register_contract(None, contract::SafeHaven);
        let client = contract::SafeHavenClient::new(&env, &contract_id);

        client.initialize(&admin, &admin, &1_000_000_000_000_000, &157_788_000);
        client.init_yield_farming(&admin).unwrap();
        client.add_farming_protocol(&admin, &protocol).unwrap();

        let unlock_time = env.ledger().timestamp() + 86400;
        let deposit_id = client.deposit(&depositor, &token, &100_000_000, &unlock_time, &0);

        client.pause(&admin).unwrap();

        // Try enabling farming while paused
        let result = client.enable_farming(&depositor, &deposit_id, &0u8, &protocol);
        assert!(result.is_err());
    }
}
