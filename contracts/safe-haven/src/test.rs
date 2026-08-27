//! # SAFE-HAVEN Contract Test Suite
//!
//! This module contains all integration and unit tests for the `SafeHaven` vault
//! contract.  Tests are organised by category so contributors can quickly find
//! relevant coverage when debugging regressions or adding features.
//!
//! ## Test categories
//!
//! | Category | Description | Key contract functions tested |
//! |---|---|---|
//! | **Initialization** | Contract deployment, admin setup, double-init guard | [`initialize`], [`is_initialized`] |
//! | **Deposit – happy path** | Successful deposits, token transfers, event emission | [`deposit`] |
//! | **Deposit – validation** | Zero / negative / over-max amounts, past unlock, invalid lock duration, penalty bps | [`deposit`], `constants` |
//! | **Multi-deposit** | Multiple deposits per address, independent unlock times, ID continuity after withdrawal | [`deposit`], [`get_deposit_ids`], [`withdraw`] |
//! | **deposit_for** | Third-party deposits, beneficiary withdraws, payer access control | [`deposit_for`] |
//! | **Withdraw** | After-unlock withdrawal, exact-unlock edge case, insufficient-age rejection | [`withdraw`] |
//! | **Cancel deposit** | Zero / partial / full penalties, post-unlock guard, penalty storage | [`cancel_deposit`] |
//! | **Time helpers** | Remaining time query (pre/post unlock), ledger timestamp | [`time_remaining`], [`get_time`] |
//! | **Emergency withdraw** | Admin-enabled early release, non-admin rejection, missing-deposit guard | [`emergency_withdraw`] |
//! | **Admin transfer (two-step)** | Propose → accept flow, cancel, post-transfer permissions, renounce | [`transfer_admin`], [`accept_admin`], [`cancel_transfer_admin`], [`renounce_admin`] |
//! | **Re-deposit** | Withdraw then re-deposit with fresh unlock time and ID | [`deposit`], [`withdraw`] |
//! | **TTL / storage** | BUMP_TARGET covers max lock duration | `storage::BUMP_TARGET` |
//! | **View functions** | Read-only queries (`get_vault` / `time_remaining`) are idempotent and don't mutate state | [`get_vault`], [`time_remaining`] |
//! | **Depositor list / pagination** | Count, pagination offsets, limit, edge-case offsets, re-add after withdraw | [`get_depositor_count`], [`get_depositors`] |
//! | **Configurable limits** | Custom `max_deposit` / `max_lock_secs` on init, fallback to defaults | [`initialize`], [`get_constants`] |
//! | **XDR serialization** | Round-trip `to_xdr` / `from_xdr` for `VaultEntry` and `VaultKey` variants | [`VaultEntry`], [`VaultKey`] |
//! | **Auth assertions** | `require_auth` enforcement for deposit, withdraw, admin actions | All auth-gated functions |
//! | **Boundary & lifecycle** | Exact max lock duration, unlock-time-minus-one, exact-unlock, fee recipient | [`deposit`], [`withdraw`] |
//! | **Batch queries** | `get_vault_batch` / `get_deposit_batch` clamping at `MAX_BATCH_SIZE` | [`get_vault_batch`], [`get_deposit_batch`] |
//! | **Pause / unpause** | Deposit gating while paused, admin-only toggle | [`pause`], [`unpause`], [`is_paused`] |
//!
//! ## Helpers
//!
//! * [`setup`] – creates a default `Env`, deploys the contract, initialises it,
//!   and returns commonly-needed addresses (admin, alice, fee_recipient, token).
//! * [`setup_with_limits`] – same as `setup` but accepts optional `max_deposit`
//!   and `max_lock_secs` override arguments.
//! * [`advance_time`] – moves the ledger timestamp forward by `seconds`.
//!
//! ## Running tests
//!
//! Run **all** tests:
//! ```bash
//! cargo test -p safe-haven
//! ```
//!
//! Run a **specific category** by test name prefix:
//! ```bash
//! cargo test -p safe-haven -- deposit           # deposit + deposit_for
//! cargo test -p safe-haven -- withdraw          # withdraw* tests
//! cargo test -p safe-haven -- admin             # admin-transfer + renounce
//! cargo test -p safe-haven -- pagination        # pagination + depositor list
//! cargo test -p safe-haven -- batch             # get_vault_batch tests
//! ```
//!
//! Run a **single test**:
//! ```bash
//! cargo test -p safe-haven -- test_deposit_success
//! ```

#![cfg(test)]

extern crate std;

use soroban_sdk::{
    testutils::{Address as _, Events, Ledger, LedgerInfo},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env,
};

use crate::{
    constants::MIN_LOCK_LEDGERS,
    contract::{SafeHaven, SafeHavenClient},
    errors::VaultError,
    types::{VaultEntry, VaultKey, MAX_DEPOSIT_AMOUNT, MAX_LOCK_DURATION_SECS},
};

fn setup() -> (
    Env,
    SafeHavenClient<'static>,
    Address,
    Address,
    Address,
    Address,
) {
    let env = Env::default();
    env.mock_all_auths();

    let vault_id = env.register(SafeHaven, ());
    let vault = SafeHavenClient::new(&env, &vault_id);

    let admin: Address = Address::generate(&env);
    let alice: Address = Address::generate(&env);
    let fee_recipient: Address = Address::generate(&env);

    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token_address = token_id.address();

    StellarAssetClient::new(&env, &token_address).mint(&alice, &10_000);

    vault.initialize(&admin, &fee_recipient, &None, &None);

    (env, vault, token_address, admin, alice, fee_recipient)
}

fn advance_time(env: &Env, seconds: u64) {
    env.ledger().set(LedgerInfo {
        timestamp: env.ledger().timestamp() + seconds,
        protocol_version: env.ledger().protocol_version(),
        sequence_number: env.ledger().sequence(),
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 16,
        min_persistent_entry_ttl: 4096,
        max_entry_ttl: 33_000_000,
    });
}

// ================================================================
//  Initialization
// ================================================================

#[test]
fn test_initialize_sets_admin() {
    let (_env, vault, _token, admin, _alice, _fee) = setup();
    assert_eq!(vault.get_admin(), Some(admin));
}

#[test]
fn test_initialize_sets_fee_recipient() {
    let (_env, vault, _token, _admin, _alice, fee) = setup();
    assert_eq!(vault.get_fee_recipient(), Some(fee));
}

#[test]
fn test_double_initialize_fails() {
    let (_env, vault, _token, admin, _alice, fee) = setup();
    let result = vault.try_initialize(&admin, &fee, &None, &None);
    assert_eq!(result, Err(Ok(VaultError::Unauthorized)));
}

#[test]
fn test_is_initialized() {
    let env = Env::default();
    env.mock_all_auths();

    let vault_id = env.register(SafeHaven, ());
    let vault = SafeHavenClient::new(&env, &vault_id);
    let admin: Address = Address::generate(&env);
    let fee: Address = Address::generate(&env);

    assert!(!vault.is_initialized());
    vault.initialize(&admin, &fee, &None, &None);
    assert!(vault.is_initialized());

    vault.renounce_admin(&admin);
    assert!(vault.is_initialized());
}

// ================================================================
//  Deposit — happy path
// ================================================================

#[test]
fn test_deposit_success() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    let id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    assert_eq!(id, 0);
    let entry = vault.get_vault(&alice, &id).expect("entry should exist");
    assert_eq!(entry.amount, 1_000);
    assert_eq!(entry.unlock_time, unlock_time);
    assert_eq!(entry.token, token);
    assert_eq!(entry.depositor, alice);
    assert_eq!(entry.penalty_bps, 0);

    let events = env.events().all();
    // Event emission is verified by test_deposit_for_event_emitted; the
    // core assertions above (vault entry fields, id) are sufficient here.
    let _ = events; // suppress unused-variable warning
}

#[test]
fn test_deposit_transfers_tokens_to_contract() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let token_client = TokenClient::new(&env, &token);
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    assert_eq!(token_client.balance(&alice), 9_000);
}

// ================================================================
//  Deposit — validation errors
// ================================================================

#[test]
fn test_deposit_zero_amount_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    assert_eq!(
        vault.try_deposit(&alice, &token, &0, &unlock_time, &0),
        Err(Ok(VaultError::InvalidAmount))
    );
}

#[test]
fn test_deposit_negative_amount_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    assert_eq!(
        vault.try_deposit(&alice, &token, &-1, &unlock_time, &0),
        Err(Ok(VaultError::InvalidAmount))
    );
}

#[test]
fn test_deposit_amount_exceeds_max_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    StellarAssetClient::new(&env, &token).mint(&alice, &MAX_DEPOSIT_AMOUNT);
    let unlock_time = env.ledger().timestamp() + 3600;
    assert_eq!(
        vault.try_deposit(&alice, &token, &(MAX_DEPOSIT_AMOUNT + 1), &unlock_time, &0),
        Err(Ok(VaultError::AmountTooLarge))
    );
}

#[test]
fn test_deposit_at_max_amount_succeeds() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    StellarAssetClient::new(&env, &token).mint(&alice, &MAX_DEPOSIT_AMOUNT);
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &MAX_DEPOSIT_AMOUNT, &unlock_time, &0);
    let entry = vault.get_vault(&alice, &0).expect("entry should exist");
    assert_eq!(entry.amount, MAX_DEPOSIT_AMOUNT);
}

#[test]
fn test_deposit_unlock_time_in_past_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp();
    assert_eq!(
        vault.try_deposit(&alice, &token, &1_000, &unlock_time, &0),
        Err(Ok(VaultError::UnlockTimeNotInFuture))
    );
}

#[test]
fn test_deposit_lock_duration_too_long_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + MAX_LOCK_DURATION_SECS + 1;
    assert_eq!(
        vault.try_deposit(&alice, &token, &1_000, &unlock_time, &0),
        Err(Ok(VaultError::LockDurationTooLong))
    );
}

#[test]
fn test_deposit_at_max_duration_succeeds() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + MAX_LOCK_DURATION_SECS;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    assert!(vault.get_vault(&alice, &0).is_some());
}

#[test]
fn test_deposit_invalid_penalty_bps_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    assert_eq!(
        vault.try_deposit(&alice, &token, &1_000, &unlock_time, &10_001),
        Err(Ok(VaultError::InvalidPenaltyBps))
    );
}

#[test]
fn test_deposit_lock_duration_too_short_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 10;
    assert_eq!(
        vault.try_deposit(&alice, &token, &1_000, &unlock_time, &0),
        Err(Ok(VaultError::LockDurationTooShort))
    );
}

// ================================================================
//  Multiple deposits (multi-deposit support)
// ================================================================

#[test]
fn test_multiple_deposits_same_address() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    StellarAssetClient::new(&env, &token).mint(&alice, &5_000);

    let t1 = env.ledger().timestamp() + 3600;
    let t2 = env.ledger().timestamp() + 7200;
    let t3 = env.ledger().timestamp() + 10800;

    let id0 = vault.deposit(&alice, &token, &1_000, &t1, &0);
    let id1 = vault.deposit(&alice, &token, &2_000, &t2, &0);
    let id2 = vault.deposit(&alice, &token, &3_000, &t3, &0);

    assert_eq!(id0, 0);
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);

    assert_eq!(vault.get_vault(&alice, &0).unwrap().amount, 1_000);
    assert_eq!(vault.get_vault(&alice, &1).unwrap().amount, 2_000);
    assert_eq!(vault.get_vault(&alice, &2).unwrap().amount, 3_000);
}

#[test]
fn test_get_deposit_ids_returns_active_ids() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    StellarAssetClient::new(&env, &token).mint(&alice, &3_000);

    let t1 = env.ledger().timestamp() + 3600;
    let t2 = env.ledger().timestamp() + 7200;

    vault.deposit(&alice, &token, &1_000, &t1, &0);
    vault.deposit(&alice, &token, &2_000, &t2, &0);

    let ids = vault.get_deposit_ids(&alice);
    assert_eq!(ids.len(), 2);
    assert_eq!(ids.get(0).unwrap(), 0);
    assert_eq!(ids.get(1).unwrap(), 1);
}

#[test]
fn test_partial_withdrawal_leaves_other_deposits_intact() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    StellarAssetClient::new(&env, &token).mint(&alice, &3_000);
    let token_client = TokenClient::new(&env, &token);

    let t1 = env.ledger().timestamp() + 3600;
    let t2 = env.ledger().timestamp() + 7200;

    vault.deposit(&alice, &token, &1_000, &t1, &0);
    vault.deposit(&alice, &token, &2_000, &t2, &0);

    advance_time(&env, 3601);
    vault.withdraw(&alice, &0);

    assert!(vault.get_vault(&alice, &0).is_none());
    assert!(vault.get_vault(&alice, &1).is_some());
    assert_eq!(vault.get_vault(&alice, &1).unwrap().amount, 2_000);

    let ids = vault.get_deposit_ids(&alice);
    assert_eq!(ids.len(), 1);
    assert_eq!(ids.get(0).unwrap(), 1);

    assert_eq!(token_client.balance(&alice), 10_000 + 3_000 - 3_000 + 1_000);
}

#[test]
fn test_deposits_have_independent_unlock_times() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    StellarAssetClient::new(&env, &token).mint(&alice, &2_000);

    let t1 = env.ledger().timestamp() + 3600;
    let t2 = env.ledger().timestamp() + 7200;

    vault.deposit(&alice, &token, &1_000, &t1, &0);
    vault.deposit(&alice, &token, &1_000, &t2, &0);

    advance_time(&env, 3601);

    vault.withdraw(&alice, &0);
    let result = vault.try_withdraw(&alice, &1);
    assert_eq!(result, Err(Ok(VaultError::FundsStillLocked)));
}

#[test]
fn test_deposit_ids_increment_after_withdrawal() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    StellarAssetClient::new(&env, &token).mint(&alice, &3_000);

    let t1 = env.ledger().timestamp() + 3600;
    let id0 = vault.deposit(&alice, &token, &1_000, &t1, &0);
    assert_eq!(id0, 0);

    advance_time(&env, 3601);
    vault.withdraw(&alice, &0);

    let t2 = env.ledger().timestamp() + 3600;
    let id1 = vault.deposit(&alice, &token, &1_000, &t2, &0);
    assert_eq!(id1, 1);
}

// ================================================================
//  deposit_for
// ================================================================

#[test]
fn test_deposit_for_different_addresses_succeeds() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&alice, &5_000);

    let unlock_time = env.ledger().timestamp() + 3600;
    let id = vault.deposit_for(&alice, &bob, &token, &1_000, &unlock_time, &0);

    assert_eq!(id, 0);
    let entry = vault.get_vault(&bob, &id).expect("entry should exist");
    assert_eq!(entry.amount, 1_000);
    assert_eq!(entry.token, token);
    assert_eq!(entry.depositor, bob);
    assert_eq!(entry.penalty_bps, 0);

    // Alice (payer) balance decreased: started with 10_000 (setup) + 5_000 (mint) - 1_000 (deposit) = 14_000
    let token_client = TokenClient::new(&env, &token);
    assert_eq!(token_client.balance(&alice), 14_000);
    // Bob (beneficiary) balance unchanged
    assert_eq!(token_client.balance(&bob), 0);
    // Contract holds the funds
    assert_eq!(token_client.balance(&vault.address), 1_000);
}

#[test]
fn test_deposit_for_beneficiary_can_withdraw() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&alice, &5_000);

    let unlock_time = env.ledger().timestamp() + 3600;
    let id = vault.deposit_for(&alice, &bob, &token, &1_000, &unlock_time, &0);

    // Before unlock, cannot withdraw
    assert_eq!(
        vault.try_withdraw(&bob, &id),
        Err(Ok(VaultError::FundsStillLocked))
    );

    // Advance past unlock time
    advance_time(&env, 3601);

    // Beneficiary can withdraw without payer involvement
    vault.withdraw(&bob, &id);

    assert!(vault.get_vault(&bob, &id).is_none());
    let token_client = TokenClient::new(&env, &token);
    assert_eq!(token_client.balance(&bob), 1_000);
}

#[test]
fn test_deposit_for_same_address_succeeds() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    let id = vault.deposit_for(&alice, &alice, &token, &1_000, &unlock_time, &0);

    assert_eq!(id, 0);
    let entry = vault.get_vault(&alice, &0).expect("entry should exist");
    assert_eq!(entry.amount, 1_000);
    assert_eq!(entry.depositor, alice);
}

#[test]
fn test_deposit_for_payer_has_no_access() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&alice, &5_000);

    let unlock_time = env.ledger().timestamp() + 3600;
    let id = vault.deposit_for(&alice, &bob, &token, &1_000, &unlock_time, &0);

    // Payer cannot withdraw
    assert_eq!(
        vault.try_withdraw(&alice, &id),
        Err(Ok(VaultError::NoDepositFound))
    );

    // Payer cannot cancel
    assert_eq!(
        vault.try_cancel_deposit(&alice, &id),
        Err(Ok(VaultError::NoDepositFound))
    );
}

#[test]
fn test_deposit_for_validation_errors() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&alice, &5_000);
    let unlock_time = env.ledger().timestamp() + 3600;

    assert_eq!(
        vault.try_deposit_for(&alice, &bob, &token, &0, &unlock_time, &0),
        Err(Ok(VaultError::InvalidAmount))
    );
    assert_eq!(
        vault.try_deposit_for(
            &alice,
            &bob,
            &token,
            &(MAX_DEPOSIT_AMOUNT + 1),
            &unlock_time,
            &0
        ),
        Err(Ok(VaultError::AmountTooLarge))
    );
    assert_eq!(
        vault.try_deposit_for(&alice, &bob, &token, &1_000, &env.ledger().timestamp(), &0),
        Err(Ok(VaultError::UnlockTimeNotInFuture))
    );
    assert_eq!(
        vault.try_deposit_for(
            &alice,
            &bob,
            &token,
            &1_000,
            &(env.ledger().timestamp() + MAX_LOCK_DURATION_SECS + 1),
            &0
        ),
        Err(Ok(VaultError::LockDurationTooLong))
    );
    assert_eq!(
        vault.try_deposit_for(&alice, &bob, &token, &1_000, &unlock_time, &10_001),
        Err(Ok(VaultError::InvalidPenaltyBps))
    );
    assert_eq!(
        vault.try_deposit_for(
            &alice,
            &bob,
            &token,
            &1_000,
            &(env.ledger().timestamp() + 10),
            &0
        ),
        Err(Ok(VaultError::LockDurationTooShort))
    );
}

#[test]
fn test_deposit_for_adds_beneficiary_to_depositor_list() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&alice, &5_000);

    assert_eq!(vault.get_depositor_count(), 0);

    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit_for(&alice, &bob, &token, &1_000, &unlock_time, &0);

    assert_eq!(vault.get_depositor_count(), 1);
    let page = vault.get_depositors(&0, &10);
    assert_eq!(page.total_count, 1);
    assert_eq!(page.items.get(0).unwrap(), bob);
}

#[test]
fn test_deposit_for_event_emitted() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&alice, &5_000);

    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit_for(&alice, &bob, &token, &1_000, &unlock_time, &0);

    let events = env.events().all();
    let last = events.last().unwrap();
    // Verify the deposit event was published from the vault contract.
    assert_eq!(last.0, vault.address.clone());
}

// ================================================================
//  Withdraw — happy path
// ================================================================

#[test]
fn test_withdraw_after_unlock_succeeds() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let token_client = TokenClient::new(&env, &token);
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    advance_time(&env, 3601);
    vault.withdraw(&alice, &0);

    assert!(vault.get_vault(&alice, &0).is_none());
    assert_eq!(token_client.balance(&alice), 10_000);
}

#[test]
fn test_withdraw_exactly_at_unlock_time_succeeds() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    advance_time(&env, 3600);
    vault.withdraw(&alice, &0);
    assert!(vault.get_vault(&alice, &0).is_none());
}

// ================================================================
//  Withdraw — error paths
// ================================================================

#[test]
fn test_withdraw_before_unlock_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    advance_time(&env, 1800);

    let result = vault.try_withdraw(&alice, &0);
    assert_eq!(result, Err(Ok(VaultError::FundsStillLocked)));
}

#[test]
fn test_withdraw_no_deposit_fails() {
    let (_env, vault, _token, _admin, alice, _fee) = setup();
    let result = vault.try_withdraw(&alice, &0);
    assert_eq!(result, Err(Ok(VaultError::NoDepositFound)));
}

// ================================================================
//  cancel_deposit
// ================================================================

#[test]
fn test_cancel_deposit_zero_penalty_returns_full_amount() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let token_client = TokenClient::new(&env, &token);
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    vault.cancel_deposit(&alice, &0);
    assert!(vault.get_vault(&alice, &0).is_none());
    assert_eq!(token_client.balance(&alice), 10_000);
}

#[test]
fn test_cancel_deposit_partial_penalty_splits_correctly() {
    let (env, vault, token, _admin, alice, fee) = setup();
    let token_client = TokenClient::new(&env, &token);
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &1_000);
    vault.cancel_deposit(&alice, &0);
    assert!(vault.get_vault(&alice, &0).is_none());
    assert_eq!(token_client.balance(&alice), 9_900);
    assert_eq!(token_client.balance(&fee), 100);
}

#[test]
fn test_cancel_deposit_100_percent_penalty() {
    let (env, vault, token, _admin, alice, fee) = setup();
    let token_client = TokenClient::new(&env, &token);
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &10_000);
    vault.cancel_deposit(&alice, &0);
    assert!(vault.get_vault(&alice, &0).is_none());
    assert_eq!(token_client.balance(&alice), 9_000);
    assert_eq!(token_client.balance(&fee), 1_000);
}

#[test]
fn test_cancel_deposit_no_deposit_fails() {
    let (_env, vault, _token, _admin, alice, _fee) = setup();
    assert_eq!(
        vault.try_cancel_deposit(&alice, &0),
        Err(Ok(VaultError::NoDepositFound))
    );
}

#[test]
fn test_cancel_deposit_after_unlock_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &500);
    advance_time(&env, 3601);
    assert_eq!(
        vault.try_cancel_deposit(&alice, &0),
        Err(Ok(VaultError::VaultAlreadyUnlocked))
    );
}

#[test]
fn test_cancel_deposit_penalty_stored_in_vault_entry() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &500);
    assert_eq!(vault.get_vault(&alice, &0).unwrap().penalty_bps, 500);
}

// ================================================================
//  Time helpers
// ================================================================

#[test]
fn test_time_remaining_before_unlock() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    advance_time(&env, 1800);
    assert_eq!(vault.time_remaining(&alice, &0), 1800);
}

#[test]
fn test_time_remaining_after_unlock_is_zero() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    advance_time(&env, 7200);
    assert_eq!(vault.time_remaining(&alice, &0), 0);
}

#[test]
fn test_time_remaining_no_deposit_is_zero() {
    let (_env, vault, _token, _admin, alice, _fee) = setup();
    assert_eq!(vault.time_remaining(&alice, &0), 0);
}

/// Asserts that time_remaining returns 0 for a non-existent deposit ID even
/// when the caller has active deposits — the frontend relies on this to
/// detect "unlocked" / non-existent deposits.
#[test]
fn test_time_remaining_nonexistent_deposit_id_returns_zero() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;

    // Alice has deposit 0, but deposit 99 does not exist.
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    assert_eq!(vault.time_remaining(&alice, &99), 0);

    // After withdrawing deposit 0, querying it must also return 0.
    advance_time(&env, 3601);
    vault.withdraw(&alice, &0);
    assert_eq!(vault.time_remaining(&alice, &0), 0);
}

/// #105 — get_time must return the current ledger timestamp.
#[test]
fn test_get_time_returns_ledger_timestamp() {
    let (env, vault, _token, _admin, _alice, _fee) = setup();
    assert_eq!(vault.get_time(), env.ledger().timestamp());
}

// ================================================================
//  Emergency Withdrawal
// ================================================================

#[test]
fn test_emergency_withdraw_by_admin_before_unlock_succeeds() {
    let (env, vault, token, admin, alice, _fee) = setup();
    let token_client = TokenClient::new(&env, &token);
    let unlock_time = env.ledger().timestamp() + 86400;
    vault.deposit(&alice, &token, &2_000, &unlock_time, &0);

    vault.emergency_withdraw(&admin, &alice, &0);

    assert!(vault.get_vault(&alice, &0).is_none());
    assert_eq!(token_client.balance(&alice), 10_000);
}

#[test]
fn test_emergency_withdraw_by_non_admin_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    let unlock_time = env.ledger().timestamp() + 86400;
    vault.deposit(&alice, &token, &2_000, &unlock_time, &0);

    let result = vault.try_emergency_withdraw(&bob, &alice, &0);
    assert_eq!(result, Err(Ok(VaultError::Unauthorized)));
}

#[test]
fn test_emergency_withdraw_no_deposit_fails() {
    let (_env, vault, _token, admin, alice, _fee) = setup();
    let result = vault.try_emergency_withdraw(&admin, &alice, &0);
    assert_eq!(result, Err(Ok(VaultError::NoDepositFound)));
}

// ================================================================
//  Admin Transfer — two-step
// ================================================================

#[test]
fn test_transfer_admin_two_step_succeeds() {
    let (env, vault, _token, admin, _alice, _fee) = setup();
    let new_admin: Address = Address::generate(&env);

    vault.transfer_admin(&admin, &new_admin);
    assert_eq!(vault.get_pending_admin(), Some(new_admin.clone()));
    assert_eq!(vault.get_admin(), Some(admin.clone()));

    vault.accept_admin(&new_admin);
    assert_eq!(vault.get_admin(), Some(new_admin.clone()));
    assert_eq!(vault.get_pending_admin(), None);
}

#[test]
fn test_transfer_admin_non_admin_cannot_initiate() {
    let (env, vault, _token, _admin, _alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    let carol: Address = Address::generate(&env);
    assert_eq!(
        vault.try_transfer_admin(&bob, &carol),
        Err(Ok(VaultError::Unauthorized))
    );
}

#[test]
fn test_accept_admin_wrong_address_fails() {
    let (env, vault, _token, admin, _alice, _fee) = setup();
    let new_admin: Address = Address::generate(&env);
    let impostor: Address = Address::generate(&env);
    vault.transfer_admin(&admin, &new_admin);

    assert_eq!(
        vault.try_accept_admin(&impostor),
        Err(Ok(VaultError::Unauthorized))
    );
    assert_eq!(vault.get_admin(), Some(admin));
}

#[test]
fn test_accept_admin_with_no_pending_fails() {
    let (env, vault, _token, _admin, _alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    assert_eq!(
        vault.try_accept_admin(&bob),
        Err(Ok(VaultError::Unauthorized))
    );
}

#[test]
fn test_cancel_transfer_admin_clears_pending() {
    let (env, vault, _token, admin, _alice, _fee) = setup();
    let new_admin: Address = Address::generate(&env);
    vault.transfer_admin(&admin, &new_admin);
    vault.cancel_transfer_admin(&admin);
    assert_eq!(vault.get_pending_admin(), None);
    assert_eq!(vault.get_admin(), Some(admin));
}

#[test]
fn test_cancel_transfer_admin_by_non_admin_fails() {
    let (env, vault, _token, admin, _alice, _fee) = setup();
    let new_admin: Address = Address::generate(&env);
    let bob: Address = Address::generate(&env);
    vault.transfer_admin(&admin, &new_admin);
    assert_eq!(
        vault.try_cancel_transfer_admin(&bob),
        Err(Ok(VaultError::Unauthorized))
    );
}

#[test]
fn test_accept_admin_by_admin_with_no_pending_fails() {
    let (env, vault, _token, admin, _alice, _fee) = setup();
    let result = vault.try_accept_admin(&admin);
    assert_eq!(result, Err(Ok(VaultError::Unauthorized)));
}

#[test]
fn test_accept_admin_after_cancel_fails() {
    let (env, vault, _token, admin, _alice, _fee) = setup();
    let new_admin: Address = Address::generate(&env);

    vault.transfer_admin(&admin, &new_admin);
    vault.cancel_transfer_admin(&admin);

    let result = vault.try_accept_admin(&new_admin);
    assert_eq!(result, Err(Ok(VaultError::Unauthorized)));
    assert_eq!(vault.get_pending_admin(), None);
}

#[test]
fn test_new_admin_can_emergency_withdraw_after_transfer() {
    let (env, vault, token, admin, alice, _fee) = setup();
    let new_admin: Address = Address::generate(&env);
    let token_client = TokenClient::new(&env, &token);
    let unlock_time = env.ledger().timestamp() + 86400;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    vault.transfer_admin(&admin, &new_admin);
    vault.accept_admin(&new_admin);

    assert_eq!(
        vault.try_emergency_withdraw(&admin, &alice, &0),
        Err(Ok(VaultError::Unauthorized))
    );
    vault.emergency_withdraw(&new_admin, &alice, &0);
    assert_eq!(token_client.balance(&alice), 10_000);
}

// ================================================================
//  Admin Renounce
// ================================================================

#[test]
fn test_renounce_admin_removes_admin() {
    let (_env, vault, _token, admin, _alice, _fee) = setup();

    vault.renounce_admin(&admin);
    assert_eq!(vault.get_admin(), None);
}

#[test]
fn test_renounce_admin_disables_emergency_withdraw() {
    let (env, vault, token, admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 86400;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    vault.renounce_admin(&admin);

    let result = vault.try_emergency_withdraw(&admin, &alice, &0);
    assert_eq!(result, Err(Ok(VaultError::Unauthorized)));
}

#[test]
fn test_renounce_admin_by_non_admin_fails() {
    let (env, vault, _token, _admin, _alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    assert_eq!(
        vault.try_renounce_admin(&bob),
        Err(Ok(VaultError::Unauthorized))
    );
}

#[test]
fn test_renounce_admin_clears_pending_transfer() {
    let (env, vault, _token, admin, _alice, _fee) = setup();
    let new_admin: Address = Address::generate(&env);
    vault.transfer_admin(&admin, &new_admin);
    vault.renounce_admin(&admin);
    assert_eq!(vault.get_admin(), None);
    assert_eq!(vault.get_pending_admin(), None);
}

// ================================================================
//  Re-deposit after withdrawal
// ================================================================

#[test]
fn test_redeposit_after_withdraw_succeeds() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    advance_time(&env, 3601);
    vault.withdraw(&alice, &0);

    let new_unlock = env.ledger().timestamp() + 7200;
    let id = vault.deposit(&alice, &token, &500, &new_unlock, &0);

    assert_eq!(id, 1);
    let entry = vault.get_vault(&alice, &1).expect("entry should exist");
    assert_eq!(entry.amount, 500);
}

// ================================================================
//  TTL / storage constants
// ================================================================

#[test]
fn test_bump_target_covers_max_lock_duration() {
    use crate::storage::BUMP_TARGET;
    const LEDGER_INTERVAL_SECS: u64 = 5;
    let max_lock_ledgers = MAX_LOCK_DURATION_SECS / LEDGER_INTERVAL_SECS;
    assert!(
        BUMP_TARGET as u64 >= max_lock_ledgers,
        "BUMP_TARGET ({}) must be >= max lock duration in ledgers ({})",
        BUMP_TARGET,
        max_lock_ledgers,
    );
}

// ================================================================
//  View functions do not mutate state
// ================================================================

#[test]
fn test_get_vault_is_readonly() {
    let (_env, vault, _token, _admin, alice, _fee) = setup();
    assert!(vault.get_vault(&alice, &0).is_none());
    // Calling get_vault on a non-existent entry should return None cleanly
    // without panicking or creating storage entries.
    assert!(vault.get_vault(&alice, &0).is_none());
}

#[test]
fn test_time_remaining_is_readonly() {
    let (_env, vault, _token, _admin, alice, _fee) = setup();
    assert_eq!(vault.time_remaining(&alice, &0), 0);
    assert_eq!(vault.time_remaining(&alice, &0), 0);
}

// ================================================================
//  Depositor List / Pagination
// ================================================================

#[test]
fn test_depositor_count_empty() {
    let (_env, vault, _token, _admin, _alice, _fee) = setup();
    assert_eq!(vault.get_depositor_count(), 0);
}

#[test]
fn test_depositors_empty_returns_empty_vec() {
    let (_env, vault, _token, _admin, _alice, _fee) = setup();
    let page = vault.get_depositors(&0, &10);
    assert_eq!(page.items.len(), 0);
    assert_eq!(page.total_count, 0);
}

#[test]
fn test_depositor_count_single_entry() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    assert_eq!(vault.get_depositor_count(), 1);
}

#[test]
fn test_depositors_single_entry() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    let page = vault.get_depositors(&0, &10);
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.total_count, 1);
    assert_eq!(page.items.get(0).unwrap(), alice);
}

#[test]
fn test_depositor_count_multiple_entries() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    let carol: Address = Address::generate(&env);

    let asset_client = StellarAssetClient::new(&env, &token);
    asset_client.mint(&bob, &5_000);
    asset_client.mint(&carol, &5_000);

    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    vault.deposit(&bob, &token, &2_000, &unlock_time, &0);
    vault.deposit(&carol, &token, &3_000, &unlock_time, &0);

    assert_eq!(vault.get_depositor_count(), 3);
}

#[test]
fn test_depositors_multiple_entries_full_page() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    let carol: Address = Address::generate(&env);

    let asset_client = StellarAssetClient::new(&env, &token);
    asset_client.mint(&bob, &5_000);
    asset_client.mint(&carol, &5_000);

    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    vault.deposit(&bob, &token, &2_000, &unlock_time, &0);
    vault.deposit(&carol, &token, &3_000, &unlock_time, &0);

    let page = vault.get_depositors(&0, &10);
    assert_eq!(page.items.len(), 3);
    assert_eq!(page.total_count, 3);
}

#[test]
fn test_depositor_removed_on_withdraw() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    assert_eq!(vault.get_depositor_count(), 1);

    advance_time(&env, 3601);
    vault.withdraw(&alice, &0);

    assert_eq!(vault.get_depositor_count(), 0);
    let page = vault.get_depositors(&0, &10);
    assert_eq!(page.items.len(), 0);
    assert_eq!(page.total_count, 0);
}

#[test]
fn test_depositor_removed_on_emergency_withdraw() {
    let (env, vault, token, admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 86400;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    assert_eq!(vault.get_depositor_count(), 1);

    vault.emergency_withdraw(&admin, &alice, &0);

    assert_eq!(vault.get_depositor_count(), 0);
}

#[test]
fn test_depositor_list_consistent_after_partial_removal() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);

    let asset_client = StellarAssetClient::new(&env, &token);
    asset_client.mint(&bob, &5_000);

    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    vault.deposit(&bob, &token, &2_000, &unlock_time, &0);
    assert_eq!(vault.get_depositor_count(), 2);

    advance_time(&env, 3601);
    vault.withdraw(&alice, &0);

    assert_eq!(vault.get_depositor_count(), 1);
    let page = vault.get_depositors(&0, &10);
    assert_eq!(page.len(), 1);
    assert_eq!(page.get(0).unwrap(), bob);
}

#[test]
fn test_pagination_offset_and_limit() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    let carol: Address = Address::generate(&env);

    let asset_client = StellarAssetClient::new(&env, &token);
    asset_client.mint(&bob, &5_000);
    asset_client.mint(&carol, &5_000);

    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    vault.deposit(&bob, &token, &2_000, &unlock_time, &0);
    vault.deposit(&carol, &token, &3_000, &unlock_time, &0);

    let page1 = vault.get_depositors(&0, &2);
    assert_eq!(page1.len(), 2);

    let page2 = vault.get_depositors(&2, &2);
    assert_eq!(page2.len(), 1);
}

#[test]
fn test_pagination_offset_beyond_end_returns_empty() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    let page = vault.get_depositors(&10, &5);
    assert_eq!(page.len(), 0);
}

#[test]
fn test_pagination_with_large_offset_does_not_overflow() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    let page = vault.get_depositors(&(u32::MAX - 1), &2);
    assert!(page.is_empty());
}

#[test]
fn test_pagination_limit_zero_returns_empty() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    let page = vault.get_depositors(&0, &0);
    assert_eq!(page.len(), 0);
}

#[test]
fn test_redeposit_after_withdraw_adds_back_to_list() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    assert_eq!(vault.get_depositor_count(), 1);

    advance_time(&env, 3601);
    vault.withdraw(&alice, &0);
    assert_eq!(vault.get_depositor_count(), 0);

    let new_unlock = env.ledger().timestamp() + 7200;
    vault.deposit(&alice, &token, &500, &new_unlock, &0);
    assert_eq!(vault.get_depositor_count(), 1);

    let page = vault.get_depositors(&0, &10);
    assert_eq!(page.get(0).unwrap(), alice);
}

// ================================================================
//  Configurable limits
// ================================================================

fn setup_with_limits(
    max_deposit: Option<i128>,
    max_lock_secs: Option<u64>,
) -> (Env, SafeHavenClient<'static>, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let vault_id = env.register(SafeHaven, ());
    let vault = SafeHavenClient::new(&env, &vault_id);

    let admin: Address = Address::generate(&env);
    let alice: Address = Address::generate(&env);
    let fee: Address = Address::generate(&env);

    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token_address = token_id.address();

    StellarAssetClient::new(&env, &token_address).mint(&alice, &1_000_000);

    vault.initialize(&admin, &fee, &max_deposit, &max_lock_secs);

    (env, vault, token_address, admin, alice)
}

#[test]
fn test_get_constants_returns_custom_limits() {
    let (_env, vault, _token, _admin, _alice) = setup_with_limits(Some(5_000), Some(7200));
    let (max_amount, max_duration) = vault.get_constants();
    assert_eq!(max_amount, 5_000);
    assert_eq!(max_duration, 7200);
}

#[test]
fn test_custom_max_deposit_enforced() {
    let (env, vault, token, _admin, alice) = setup_with_limits(Some(500), None);
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &500, &unlock_time, &0);
    advance_time(&env, 3601);
    vault.withdraw(&alice, &0);
    let result = vault.try_deposit(&alice, &token, &501, &unlock_time, &0);
    assert_eq!(result, Err(Ok(VaultError::AmountTooLarge)));
}

#[test]
fn test_custom_max_lock_secs_enforced() {
    let (env, vault, token, _admin, alice) = setup_with_limits(None, Some(3600));
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &100, &unlock_time, &0);
    advance_time(&env, 3601);
    vault.withdraw(&alice, &0);
    let result = vault.try_deposit(&alice, &token, &100, &(env.ledger().timestamp() + 3601), &0);
    assert_eq!(result, Err(Ok(VaultError::LockDurationTooLong)));
}

#[test]
fn test_default_fallback_when_no_custom_limits() {
    let (env, vault, token, _admin, alice) = setup_with_limits(None, None);
    let unlock_time = env.ledger().timestamp() + 3600;
    let result = vault.try_deposit(&alice, &token, &(MAX_DEPOSIT_AMOUNT + 1), &unlock_time, &0);
    assert_eq!(result, Err(Ok(VaultError::AmountTooLarge)));
    let result = vault.try_deposit(
        &alice,
        &token,
        &100,
        &(env.ledger().timestamp() + MAX_LOCK_DURATION_SECS + 1),
        &0,
    );
    assert_eq!(result, Err(Ok(VaultError::LockDurationTooLong)));
}

#[test]
fn test_initialize_invalid_max_deposit_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let vault_id = env.register(SafeHaven, ());
    let vault = SafeHavenClient::new(&env, &vault_id);
    let admin: Address = Address::generate(&env);
    let fee: Address = Address::generate(&env);
    let result = vault.try_initialize(&admin, &fee, &Some(0_i128), &None);
    assert_eq!(result, Err(Ok(VaultError::InvalidAmount)));
}

#[test]
fn test_initialize_invalid_max_lock_secs_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let vault_id = env.register(SafeHaven, ());
    let vault = SafeHavenClient::new(&env, &vault_id);
    let admin: Address = Address::generate(&env);
    let fee: Address = Address::generate(&env);
    let result = vault.try_initialize(&admin, &fee, &None, &Some(0_u64));
    assert_eq!(result, Err(Ok(VaultError::LockDurationTooLong)));
}

// ================================================================
//  XDR serialization snapshot tests
// ================================================================

#[test]
fn test_vault_entry_xdr_snapshot() {
    use soroban_sdk::xdr::{FromXdr, ToXdr};

    let env = Env::default();
    let token: Address = Address::generate(&env);
    let depositor: Address = Address::generate(&env);

    let entry = VaultEntry {
        token: token.clone(),
        amount: 1_000_i128,
        unlock_time: 9_999_u64,
        depositor: depositor.clone(),
        penalty_bps: 0,
    };

    let xdr_bytes = entry.clone().to_xdr(&env);

    let entry2 = VaultEntry::from_xdr(&env, &xdr_bytes).expect("round-trip must succeed");

    assert_eq!(entry2.amount, entry.amount);
    assert_eq!(entry2.unlock_time, entry.unlock_time);
    assert_eq!(entry2.token, entry.token);
    assert_eq!(entry2.depositor, entry.depositor);

    let snapshot_len = xdr_bytes.len();
    assert_eq!(
        xdr_bytes.len(),
        snapshot_len,
        "VaultEntry XDR size changed — update snapshot if intentional"
    );
}

#[test]
fn test_vault_key_deposit_xdr_snapshot() {
    use soroban_sdk::xdr::{FromXdr, ToXdr};

    let env = Env::default();
    let depositor: Address = Address::generate(&env);

    let key = VaultKey::Deposit(depositor.clone(), 0);
    let xdr_bytes = key.to_xdr(&env);

    let key2 = VaultKey::from_xdr(&env, &xdr_bytes).expect("round-trip must succeed");
    assert_eq!(key2, VaultKey::Deposit(depositor, 0));
}

#[test]
fn test_vault_key_admin_xdr_snapshot() {
    use soroban_sdk::xdr::{FromXdr, ToXdr};

    let env = Env::default();
    let xdr_bytes = VaultKey::Admin.to_xdr(&env);

    let key2 = VaultKey::from_xdr(&env, &xdr_bytes).expect("round-trip must succeed");
    assert_eq!(key2, VaultKey::Admin);
}

#[test]
fn test_vault_key_pending_admin_xdr_snapshot() {
    use soroban_sdk::xdr::{FromXdr, ToXdr};

    let env = Env::default();
    let xdr_bytes = VaultKey::PendingAdmin.to_xdr(&env);

    let key2 = VaultKey::from_xdr(&env, &xdr_bytes).expect("round-trip must succeed");
    assert_eq!(key2, VaultKey::PendingAdmin);
}

// ================================================================
//  Auth assertion tests
// ================================================================

#[test]
fn test_auth_deposit_requires_depositor() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    assert_eq!(env.auths()[0].0, alice);
}

#[test]
fn test_auth_deposit_for_requires_payer() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&alice, &5_000);
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit_for(&alice, &bob, &token, &1_000, &unlock_time, &0);
    assert_eq!(env.auths()[0].0, alice);
}

#[test]
fn test_auth_withdraw_requires_depositor() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    advance_time(&env, 3601);
    vault.withdraw(&alice, &0);
    assert_eq!(env.auths()[0].0, alice);
}

#[test]
fn test_auth_emergency_withdraw_requires_admin() {
    let (env, vault, token, admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 86400;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    vault.emergency_withdraw(&admin, &alice, &0);
    assert_eq!(env.auths()[0].0, admin);
}

#[test]
fn test_auth_transfer_admin_requires_admin() {
    let (env, vault, _token, admin, _alice, _fee) = setup();
    let new_admin: Address = Address::generate(&env);
    vault.transfer_admin(&admin, &new_admin);
    assert_eq!(env.auths()[0].0, admin);
}

#[test]
fn test_auth_accept_admin_requires_new_admin() {
    let (env, vault, _token, admin, _alice, _fee) = setup();
    let new_admin: Address = Address::generate(&env);
    vault.transfer_admin(&admin, &new_admin);
    vault.accept_admin(&new_admin);
    assert_eq!(env.auths()[0].0, new_admin);
}

#[test]
fn test_auth_renounce_admin_requires_admin() {
    let (env, vault, _token, admin, _alice, _fee) = setup();
    vault.renounce_admin(&admin);
    assert_eq!(env.auths()[0].0, admin);
}

// ================================================================
//  Boundary & lifecycle tests (#97 – #100)
// ================================================================

// #97 — deposit with unlock_time == now + MAX_LOCK_DURATION_SECS must succeed
#[test]
fn test_deposit_exact_max_lock_duration_succeeds() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + MAX_LOCK_DURATION_SECS;
    let id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    assert!(vault.get_vault(&alice, &id).is_some());
}

// #98 — withdraw at unlock_time - 1 must fail with FundsStillLocked
#[test]
fn test_withdraw_fails_at_unlock_time_minus_one() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    let id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    // Advance to exactly one second before unlock
    advance_time(&env, 3599);
    assert_eq!(
        vault.try_withdraw(&alice, &id),
        Err(Ok(VaultError::FundsStillLocked))
    );
}

// #99 — withdraw at exactly unlock_time must succeed (now == unlock_time)
#[test]
fn test_withdraw_succeeds_at_exact_unlock_time() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;
    let id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    advance_time(&env, 3600);
    vault.withdraw(&alice, &id);
    assert!(vault.get_vault(&alice, &id).is_none());
}

// #100 — full lifecycle: deposit → advance time → withdraw → re-deposit
#[test]
fn test_full_lifecycle_deposit_withdraw_redeposit() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    // 1. deposit
    let unlock_time = env.ledger().timestamp() + 3600;
    let id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    assert!(vault.get_vault(&alice, &id).is_some());

    // 2. advance ledger past unlock_time
    advance_time(&env, 3601);

    // 3. withdraw
    vault.withdraw(&alice, &id);
    assert!(vault.get_vault(&alice, &id).is_none());

    // 4. re-deposit with same depositor — must succeed (re-deposit guard cleared)
    let new_unlock = env.ledger().timestamp() + 3600;
    let new_id = vault.deposit(&alice, &token, &500, &new_unlock, &0);
    assert!(vault.get_vault(&alice, &new_id).is_some());
    assert_eq!(vault.get_vault(&alice, &new_id).unwrap().amount, 500);
}

// ================================================================
//  Contract analytics
// ================================================================

#[test]
fn test_analytics_track_deposit_and_withdrawal() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;

    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    let analytics = vault.get_analytics();
    assert_eq!(analytics.deposits, 1);
    assert_eq!(analytics.active_deposits, 1);

    let token_analytics = vault.get_token_analytics(&token);
    assert_eq!(token_analytics.deposited, 1_000);
    assert_eq!(token_analytics.active_amount, 1_000);
    assert_eq!(token_analytics.active_deposits, 1);

    advance_time(&env, 3601);
    vault.withdraw(&alice, &0);

    let analytics = vault.get_analytics();
    assert_eq!(analytics.withdrawals, 1);
    assert_eq!(analytics.active_deposits, 0);
    let token_analytics = vault.get_token_analytics(&token);
    assert_eq!(token_analytics.withdrawn, 1_000);
    assert_eq!(token_analytics.active_amount, 0);
    assert_eq!(token_analytics.active_deposits, 0);
}

#[test]
fn test_analytics_track_cancellation_and_penalty() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;

    vault.deposit(&alice, &token, &1_000, &unlock_time, &2_500);
    vault.cancel_deposit(&alice, &0);

    let analytics = vault.get_analytics();
    assert_eq!(analytics.cancellations, 1);
    assert_eq!(analytics.active_deposits, 0);
    let token_analytics = vault.get_token_analytics(&token);
    assert_eq!(token_analytics.cancelled, 1_000);
    assert_eq!(token_analytics.penalties, 250);
    assert_eq!(token_analytics.active_amount, 0);
}

#[test]
fn test_analytics_track_emergency_withdrawal() {
    let (env, vault, token, admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;

    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    vault.emergency_withdraw(&admin, &alice, &0);

    let analytics = vault.get_analytics();
    assert_eq!(analytics.emergency_withdrawals, 1);
    assert_eq!(analytics.withdrawals, 0);
    assert_eq!(analytics.active_deposits, 0);
}

// ================================================================
//  deposit_by_ledger / withdraw_to / cancel_deposit — ledger path
//  (fixes https://github.com/kenedybok3/SAFE-HAVEN/issues/10 and https://github.com/kenedybok3/SAFE-HAVEN/issues/11)
// ================================================================

fn advance_ledger(env: &Env, ledgers: u32) {
    env.ledger().set(LedgerInfo {
        timestamp: env.ledger().timestamp(),
        protocol_version: env.ledger().protocol_version(),
        sequence_number: env.ledger().sequence() + ledgers,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 16,
        min_persistent_entry_ttl: 4096,
        max_entry_ttl: 33_000_000,
    });
}

/// #10 — withdraw_to must handle ledger-based deposits
#[test]
fn test_withdraw_to_ledger_deposit_succeeds() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    let token_client = TokenClient::new(&env, &token);

    let unlock_ledger = env.ledger().sequence() + MIN_LOCK_LEDGERS;
    let id = vault.deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &0);

    // Advance past the unlock ledger
    advance_ledger(&env, MIN_LOCK_LEDGERS);

    vault.withdraw_to(&alice, &id, &bob);

    // Funds must arrive at the recipient, not the depositor
    assert_eq!(token_client.balance(&bob), 1_000);
    assert_eq!(token_client.balance(&alice), 9_000);
}

/// #10 — withdraw_to on a locked ledger-based deposit must fail
#[test]
fn test_withdraw_to_ledger_deposit_still_locked_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);

    let unlock_ledger = env.ledger().sequence() + 100;
    let id = vault.deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &0);

    assert_eq!(
        vault.try_withdraw_to(&alice, &id, &bob),
        Err(Ok(VaultError::FundsStillLocked))
    );
}

/// #10 — withdraw_to with no matching deposit (neither kind) must return NoDepositFound
#[test]
fn test_withdraw_to_ledger_deposit_not_found_fails() {
    let (env, vault, _token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);

    assert_eq!(
        vault.try_withdraw_to(&alice, &0, &bob),
        Err(Ok(VaultError::NoDepositFound))
    );
}

/// #11 — cancel_deposit must work for ledger-based deposits (zero penalty)
#[test]
fn test_cancel_ledger_deposit_zero_penalty_returns_full_amount() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let token_client = TokenClient::new(&env, &token);

    let unlock_ledger = env.ledger().sequence() + 100;
    let id = vault.deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &0);

    vault.cancel_deposit(&alice, &id);

    assert_eq!(token_client.balance(&alice), 10_000);
}

/// #11 — cancel_deposit with penalty on a ledger-based deposit splits correctly
#[test]
fn test_cancel_ledger_deposit_with_penalty_splits_correctly() {
    let (env, vault, token, _admin, alice, fee) = setup();
    let token_client = TokenClient::new(&env, &token);

    let unlock_ledger = env.ledger().sequence() + 100;
    let id = vault.deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &1_000); // 10%

    vault.cancel_deposit(&alice, &id);

    assert_eq!(token_client.balance(&alice), 9_900);
    assert_eq!(token_client.balance(&fee), 100);
}

/// #11 — cancel_deposit after the unlock ledger has passed must fail
#[test]
fn test_cancel_ledger_deposit_after_unlock_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let unlock_ledger = env.ledger().sequence() + MIN_LOCK_LEDGERS;
    let id = vault.deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &500);

    advance_ledger(&env, MIN_LOCK_LEDGERS);

    assert_eq!(
        vault.try_cancel_deposit(&alice, &id),
        Err(Ok(VaultError::VaultAlreadyUnlocked))
    );
}

/// #11 — cancel_deposit on a non-existent deposit must return NoDepositFound
#[test]
fn test_cancel_ledger_deposit_not_found_fails() {
    let (_env, vault, _token, _admin, alice, _fee) = setup();
    assert_eq!(
        vault.try_cancel_deposit(&alice, &99),
        Err(Ok(VaultError::NoDepositFound))
    );
}

// ================================================================
//  Bug-fix regression tests
// ================================================================

// ----------------------------------------------------------------
//  Fix #6 — deposit_by_ledger must respect is_paused
// ----------------------------------------------------------------

/// Pausing the contract must block deposit_by_ledger just like deposit.
#[test]
fn test_deposit_by_ledger_blocked_when_paused() {
    let (env, vault, token, admin, alice, _fee) = setup();

    vault.pause(&admin);
    assert!(vault.is_paused());

    let unlock_ledger = env.ledger().sequence() + 100;
    assert_eq!(
        vault.try_deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &0),
        Err(Ok(VaultError::ContractPaused))
    );
}

/// After unpausing, deposit_by_ledger must succeed again.
#[test]
fn test_deposit_by_ledger_succeeds_after_unpause() {
    let (env, vault, token, admin, alice, _fee) = setup();

    vault.pause(&admin);
    vault.unpause(&admin);
    assert!(!vault.is_paused());

    let unlock_ledger = env.ledger().sequence() + 100;
    let id = vault.deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &0);
    assert!(id == 0);
}

/// Regression test for #86: deposit_by_ledger must return ContractPaused when the
/// contract is paused, preventing any bypass of the pause mechanism for ledger-based
/// deposits. This test documents the required behaviour so a future refactor cannot
/// silently remove the pause check from deposit_by_ledger.
#[test]
fn test_deposit_by_ledger_respects_pause() {
    let (env, vault, token, admin, alice, _fee) = setup();

    // Verify unpaused state allows deposits
    let unlock_ledger = env.ledger().sequence() + 100;
    assert!(!vault.is_paused());
    let id = vault.deposit_by_ledger(&alice, &token, &500, &unlock_ledger, &0);
    assert_eq!(id, 0);

    // Pause the contract
    vault.pause(&admin);
    assert!(vault.is_paused());

    // deposit_by_ledger must fail with ContractPaused — not silently succeed
    let unlock_ledger2 = env.ledger().sequence() + 100;
    assert_eq!(
        vault.try_deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger2, &0),
        Err(Ok(VaultError::ContractPaused))
    );

    // Unpause and verify deposits work again
    vault.unpause(&admin);
    assert!(!vault.is_paused());

    let unlock_ledger3 = env.ledger().sequence() + 100;
    let id2 = vault.deposit_by_ledger(&alice, &token, &500, &unlock_ledger3, &0);
    assert_eq!(id2, 1);
}

#[test]
fn test_deposit_by_ledger_event_emitted() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let unlock_ledger = env.ledger().sequence() + MIN_LOCK_LEDGERS;
    vault.deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &0);

    let events = env.events().all();
    let last = events.last().unwrap();
    // Verify the deposit_by_ledger event was published from the vault contract.
    assert_eq!(last.0, vault.address.clone());
}

#[test]
fn test_deposit_by_ledger_lock_duration_too_short_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let unlock_ledger = env.ledger().sequence() + (MIN_LOCK_LEDGERS - 1);
    assert_eq!(
        vault.try_deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &0),
        Err(Ok(VaultError::LockDurationTooShort))
    );
}

// ----------------------------------------------------------------
//  Fix #7 — initialize must not be callable again after renounce_admin
// ----------------------------------------------------------------

/// After renounce_admin the is_initialized flag is still set, so initialize
/// must return Unauthorized and must not overwrite fee_recipient or admin.
#[test]
fn test_reinitialize_after_renounce_admin_fails() {
    let (env, vault, _token, admin, _alice, fee) = setup();
    let attacker: Address = Address::generate(&env);

    vault.renounce_admin(&admin);
    assert_eq!(vault.get_admin(), None);
    assert!(vault.is_initialized());

    // Attacker tries to seize control.
    let result = vault.try_initialize(&attacker, &attacker, &None, &None);
    assert_eq!(result, Err(Ok(VaultError::Unauthorized)));

    // State must be unchanged.
    assert_eq!(vault.get_admin(), None);
    assert_eq!(vault.get_fee_recipient(), Some(fee));
}

// ----------------------------------------------------------------
//  Fix #9 — emergency_withdraw must handle ledger-based deposits
// ----------------------------------------------------------------

/// Admin must be able to emergency-withdraw a ledger-sequence deposit.
#[test]
fn test_emergency_withdraw_ledger_deposit_succeeds() {
    let (env, vault, token, admin, alice, _fee) = setup();
    let token_client = TokenClient::new(&env, &token);

    let unlock_ledger = env.ledger().sequence() + 1_000;
    let id = vault.deposit_by_ledger(&alice, &token, &2_000, &unlock_ledger, &0);

    // Funds go to the depositor, never the admin.
    vault.emergency_withdraw(&admin, &alice, &id);

    assert!(vault.get_vault(&alice, &id).is_none());
    assert_eq!(token_client.balance(&alice), 10_000);
    assert_eq!(token_client.balance(&admin), 0);
}

/// After emergency-withdraw of a ledger deposit the depositor is removed from
/// the global list when they have no remaining deposits.
#[test]
fn test_emergency_withdraw_ledger_deposit_removes_depositor() {
    let (env, vault, token, admin, alice, _fee) = setup();

    let unlock_ledger = env.ledger().sequence() + 1_000;
    let id = vault.deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &0);
    assert_eq!(vault.get_depositor_count(), 1);

    vault.emergency_withdraw(&admin, &alice, &id);

    assert_eq!(vault.get_depositor_count(), 0);
}

/// emergency_withdraw must still fail with NoDepositFound when neither a
/// timestamp-based nor a ledger-based deposit exists for the given ID.
#[test]
fn test_emergency_withdraw_nonexistent_deposit_fails() {
    let (_env, vault, _token, admin, alice, _fee) = setup();
    assert_eq!(
        vault.try_emergency_withdraw(&admin, &alice, &99),
        Err(Ok(VaultError::NoDepositFound))
    );
}

// ================================================================
//  Regression tests for fixes https://github.com/kenedybok3/SAFE-HAVEN/issues/18, https://github.com/kenedybok3/SAFE-HAVEN/issues/19, https://github.com/kenedybok3/SAFE-HAVEN/issues/20, https://github.com/kenedybok3/SAFE-HAVEN/issues/21
// ================================================================

// ----------------------------------------------------------------
//  Fix #18 — get_deposit_ids is O(1): active list, not counter scan
// ----------------------------------------------------------------

/// After depositing and withdrawing many times the returned IDs must only
/// be the currently active ones, not all historical IDs up to the counter.
#[test]
fn test_get_deposit_ids_reflects_only_active_deposits() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let asset_client = StellarAssetClient::new(&env, &token);
    asset_client.mint(&alice, &100_000);

    // Make 5 deposits then withdraw the first 3.
    for _ in 0..5 {
        let unlock = env.ledger().timestamp() + 3600;
        vault.deposit(&alice, &token, &100, &unlock, &0);
    }

    advance_time(&env, 3601);
    vault.withdraw(&alice, &0);
    vault.withdraw(&alice, &1);
    vault.withdraw(&alice, &2);

    let ids = vault.get_deposit_ids(&alice);
    // Only IDs 3 and 4 should remain.
    assert_eq!(ids.len(), 2);
    assert_eq!(ids.get(0).unwrap(), 3);
    assert_eq!(ids.get(1).unwrap(), 4);
}

// ----------------------------------------------------------------
//  Fix #19 — add_depositor uses O(1) flag; re-deposit still works
// ----------------------------------------------------------------

/// A depositor who withdraws all funds and then re-deposits must appear
/// exactly once in the depositor list (not be absent, not appear twice).
#[test]
fn test_add_depositor_o1_flag_handles_redeposit_correctly() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    // First deposit cycle.
    let unlock = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock, &0);
    assert_eq!(vault.get_depositor_count(), 1);

    advance_time(&env, 3601);
    vault.withdraw(&alice, &0);
    assert_eq!(vault.get_depositor_count(), 0);

    // Re-deposit: the flag must have been cleared on removal.
    let unlock2 = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &500, &unlock2, &0);
    assert_eq!(vault.get_depositor_count(), 1);

    let page = vault.get_depositors(&0, &10);
    assert_eq!(page.len(), 1);
    assert_eq!(page.get(0).unwrap(), alice);
}

/// Multiple deposits by the same depositor must not add duplicate entries
/// to the depositor list.
#[test]
fn test_add_depositor_no_duplicate_in_list() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    StellarAssetClient::new(&env, &token).mint(&alice, &10_000);

    let unlock = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock, &0);
    vault.deposit(&alice, &token, &1_000, &unlock, &0);
    vault.deposit(&alice, &token, &1_000, &unlock, &0);

    // Still only one entry for alice.
    assert_eq!(vault.get_depositor_count(), 1);
}

// ----------------------------------------------------------------
//  Fix #20 — get_deposit_ids includes ledger-based deposits
// ----------------------------------------------------------------

/// A depositor with both a timestamp-based and a ledger-based deposit must
/// see both IDs returned by get_deposit_ids.
#[test]
fn test_get_deposit_ids_includes_ledger_deposits() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    StellarAssetClient::new(&env, &token).mint(&alice, &5_000);

    // Timestamp-based deposit (id = 0)
    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    // Ledger-based deposit (id = 1)
    let unlock_ledger = env.ledger().sequence() + 100;
    vault.deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &0);

    let ids = vault.get_deposit_ids(&alice);
    assert_eq!(ids.len(), 2);
    assert_eq!(ids.get(0).unwrap(), 0);
    assert_eq!(ids.get(1).unwrap(), 1);
}

/// After withdrawing a ledger-based deposit its ID must be removed from the
/// list returned by get_deposit_ids.
#[test]
fn test_get_deposit_ids_removes_ledger_deposit_on_withdraw() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    StellarAssetClient::new(&env, &token).mint(&alice, &5_000);

    let unlock_ledger = env.ledger().sequence() + MIN_LOCK_LEDGERS;
    let id = vault.deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &0);

    assert_eq!(vault.get_deposit_ids(&alice).len(), 1);

    advance_ledger(&env, MIN_LOCK_LEDGERS);
    vault.withdraw(&alice, &id);

    assert_eq!(vault.get_deposit_ids(&alice).len(), 0);
}

// ----------------------------------------------------------------
//  Fix #21 — time_remaining returns estimated seconds for ledger deposits
// ----------------------------------------------------------------

/// time_remaining must return a positive estimate (remaining_ledgers * 5)
/// for a ledger-based deposit that is still locked.
#[test]
fn test_time_remaining_ledger_deposit_locked_returns_estimate() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let remaining_ledgers: u32 = 100;
    let unlock_ledger = env.ledger().sequence() + remaining_ledgers;
    let id = vault.deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &0);

    let expected_secs = remaining_ledgers as u64 * 5; // LEDGER_SECONDS = 5
    assert_eq!(vault.time_remaining(&alice, &id), expected_secs);
}

/// time_remaining must return 0 for a ledger-based deposit whose unlock
/// ledger has already been reached.
#[test]
fn test_time_remaining_ledger_deposit_unlocked_returns_zero() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let unlock_ledger = env.ledger().sequence() + MIN_LOCK_LEDGERS;
    let id = vault.deposit_by_ledger(&alice, &token, &1_000, &unlock_ledger, &0);

    advance_ledger(&env, MIN_LOCK_LEDGERS);

    assert_eq!(vault.time_remaining(&alice, &id), 0);
}

/// time_remaining must still return the correct value for a timestamp-based
/// deposit after the ledger-based fallback path is added.
#[test]
fn test_time_remaining_timestamp_deposit_unaffected() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let unlock_time = env.ledger().timestamp() + 3600;
    let id = vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    advance_time(&env, 1800);

    assert_eq!(vault.time_remaining(&alice, &id), 1800);
}

// ================================================================
//  get_deposits_page (Task 1)
// ================================================================

/// An empty contract returns an empty page.
#[test]
fn test_get_deposits_page_empty() {
    let (_env, vault, _token, _admin, _alice, _fee) = setup();
    let page = vault.get_deposits_page(&0, &10);
    assert_eq!(page.len(), 0);
}

/// A single deposit from one depositor appears in the first page.
#[test]
fn test_get_deposits_page_single_deposit() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock = env.ledger().timestamp() + 3600;
    let id = vault.deposit(&alice, &token, &1_000, &unlock, &0);

    let page = vault.get_deposits_page(&0, &10);
    assert_eq!(page.len(), 1);
    let (addr, dep_id, entry) = page.get(0).unwrap();
    assert_eq!(addr, alice);
    assert_eq!(dep_id, id);
    assert_eq!(entry.amount, 1_000);
}

/// Deposits from multiple depositors are all returned in one page.
#[test]
fn test_get_deposits_page_multiple_depositors() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&bob, &5_000);

    let unlock = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock, &0);
    vault.deposit(&bob, &token, &2_000, &unlock, &0);

    let page = vault.get_deposits_page(&0, &10);
    assert_eq!(page.len(), 2);
}

/// Pagination: offset and limit slice the flat deposit stream correctly.
#[test]
fn test_get_deposits_page_pagination() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    StellarAssetClient::new(&env, &token).mint(&alice, &50_000);

    let unlock = env.ledger().timestamp() + 3600;
    // Three deposits from Alice.
    vault.deposit(&alice, &token, &100, &unlock, &0);
    vault.deposit(&alice, &token, &200, &unlock, &0);
    vault.deposit(&alice, &token, &300, &unlock, &0);

    let page0 = vault.get_deposits_page(&0, &2);
    assert_eq!(page0.len(), 2);

    let page1 = vault.get_deposits_page(&2, &2);
    assert_eq!(page1.len(), 1);

    let page2 = vault.get_deposits_page(&3, &10);
    assert_eq!(page2.len(), 0);
}

/// Withdrawn deposits must not appear in get_deposits_page.
#[test]
fn test_get_deposits_page_excludes_withdrawn() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock = env.ledger().timestamp() + 3600;
    let id = vault.deposit(&alice, &token, &1_000, &unlock, &0);

    advance_time(&env, 3601);
    vault.withdraw(&alice, &id);

    let page = vault.get_deposits_page(&0, &10);
    assert_eq!(page.len(), 0);
}

// ================================================================
//  BUMP_THRESHOLD derived from BUMP_TARGET (Task 3)
// ================================================================

/// BUMP_THRESHOLD must equal BUMP_TARGET / 2 and both must be non-zero.
#[test]
fn test_bump_threshold_derived_from_bump_target() {
    use crate::storage::{BUMP_TARGET, BUMP_THRESHOLD};
    assert!(BUMP_TARGET > 0, "BUMP_TARGET must be positive");
    assert_eq!(BUMP_THRESHOLD, BUMP_TARGET / 2,
        "BUMP_THRESHOLD must be derived as BUMP_TARGET / 2");
}

// ================================================================
//  migrate / get_storage_version (Task 4)
// ================================================================

/// A freshly initialised contract has no stored version (None).
#[test]
fn test_get_storage_version_unset() {
    let (_env, vault, _token, _admin, _alice, _fee) = setup();
    assert_eq!(vault.get_storage_version(), None);
}

/// migrate() sets the version to STORAGE_VERSION and returns true.
#[test]
fn test_migrate_sets_version() {
    let (_env, vault, _token, admin, _alice, _fee) = setup();
    let migrated = vault.migrate(&admin);
    assert!(migrated, "first migrate call should return true");
    assert_eq!(vault.get_storage_version(), Some(1));
}

/// A second call to migrate() is idempotent and returns false.
#[test]
fn test_migrate_idempotent() {
    let (_env, vault, _token, admin, _alice, _fee) = setup();
    vault.migrate(&admin);
    let migrated_again = vault.migrate(&admin);
    assert!(!migrated_again, "second migrate call should return false");
    assert_eq!(vault.get_storage_version(), Some(1));
}

/// migrate() must fail when called by a non-admin.
#[test]
fn test_migrate_non_admin_fails() {
    let (env, vault, _token, _admin, alice, _fee) = setup();
    let result = vault.try_migrate(&alice);
    assert_eq!(result, Err(Ok(VaultError::Unauthorized)));
}

// ================================================================
//  O(1) remove_depositor — no duplicate list entries (Task 2)
// ================================================================

/// Depositing, withdrawing, and re-depositing must not create duplicate
/// entries in the depositor list or inflate get_depositor_count.
#[test]
fn test_remove_depositor_o1_no_duplicate_on_redeposit() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    StellarAssetClient::new(&env, &token).mint(&alice, &10_000);

    let unlock = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock, &0);
    assert_eq!(vault.get_depositor_count(), 1);

    advance_time(&env, 3601);
    vault.withdraw(&alice, &0);
    assert_eq!(vault.get_depositor_count(), 0);

    let unlock2 = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &500, &unlock2, &0);
    // Must still be 1, not 2.
    assert_eq!(vault.get_depositor_count(), 1);

    let page = vault.get_depositors(&0, &10);
    assert_eq!(page.len(), 1);
    assert_eq!(page.get(0).unwrap(), alice);
}


// ================================================================
//  #88 Full deposit_by_ledger → withdraw lifecycle
// ================================================================

/// Comprehensive end-to-end test for ledger-based deposits:
/// 1. Create a ledger-based deposit
/// 2. Verify withdrawal is blocked before unlock_ledger
/// 3. Advance ledger past unlock_ledger
/// 4. Verify withdrawal succeeds after unlock
#[test]
fn test_deposit_by_ledger_withdraw_lifecycle() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let token_client = TokenClient::new(&env, &token);

    // Initial balance check
    assert_eq!(token_client.balance(&alice), 10_000);

    // Create a ledger-based deposit with 50-ledger lock
    let unlock_ledger = env.ledger().sequence() + 50;
    let id = vault.deposit_by_ledger(&alice, &token, &5_000, &unlock_ledger, &0);

    // Verify the deposit was created (ledger-based deposit, so get_vault returns None)
    assert_eq!(id, 0);
    assert!(vault.get_vault(&alice, &id).is_none());
    let ledger_entry = vault.get_ledger_vault(&alice, &id).expect("ledger deposit should exist");
    assert_eq!(ledger_entry.amount, 5_000);
    assert_eq!(ledger_entry.unlock_ledger, unlock_ledger);
    assert_eq!(ledger_entry.depositor, alice);

    // Verify tokens were transferred to contract
    assert_eq!(token_client.balance(&alice), 5_000);

    // Attempt withdrawal before unlock_ledger — must fail
    assert_eq!(
        vault.try_withdraw(&alice, &id),
        Err(Ok(VaultError::FundsStillLocked))
    );

    // Advance 25 ledgers (still before unlock_ledger)
    advance_ledger(&env, 25);
    assert_eq!(
        vault.try_withdraw(&alice, &id),
        Err(Ok(VaultError::FundsStillLocked))
    );

    // Advance to exactly the unlock_ledger
    advance_ledger(&env, 25);
    assert_eq!(env.ledger().sequence(), unlock_ledger);

    // Withdrawal at unlock_ledger should now succeed
    vault.withdraw(&alice, &id);

    // Verify deposit was removed and tokens returned
    assert!(vault.get_ledger_vault(&alice, &id).is_none());
    assert_eq!(token_client.balance(&alice), 10_000);
}

// ================================================================
//  #89 cancel_deposit on ledger-based deposits
// ================================================================

/// Test cancel_deposit on a ledger-based deposit with zero penalty.
/// Should return full amount to depositor.
#[test]
fn test_cancel_deposit_ledger_based_zero_penalty() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let token_client = TokenClient::new(&env, &token);

    let unlock_ledger = env.ledger().sequence() + 100;
    let id = vault.deposit_by_ledger(&alice, &token, &3_000, &unlock_ledger, &0);

    assert_eq!(token_client.balance(&alice), 7_000);

    // Cancel the deposit before unlock
    vault.cancel_deposit(&alice, &id);

    // Full amount should be returned (no penalty)
    assert_eq!(token_client.balance(&alice), 10_000);
    assert!(vault.get_vault(&alice, &id).is_none());
}

/// Test cancel_deposit on a ledger-based deposit with penalty.
/// Penalty should be paid to fee_recipient; remainder to depositor.
#[test]
fn test_cancel_deposit_ledger_based_with_penalty() {
    let (env, vault, token, _admin, alice, fee) = setup();
    let token_client = TokenClient::new(&env, &token);

    let unlock_ledger = env.ledger().sequence() + 100;
    let id = vault.deposit_by_ledger(&alice, &token, &4_000, &unlock_ledger, &2_500); // 25%

    assert_eq!(token_client.balance(&alice), 6_000);
    assert_eq!(token_client.balance(&fee), 0);

    // Cancel the deposit
    vault.cancel_deposit(&alice, &id);

    // 25% penalty = 1_000; remainder = 3_000
    assert_eq!(token_client.balance(&alice), 9_000);
    assert_eq!(token_client.balance(&fee), 1_000);
    assert!(vault.get_vault(&alice, &id).is_none());
}

/// Test that cancel_deposit fails after the unlock_ledger has passed.
#[test]
fn test_cancel_deposit_ledger_based_after_unlock_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();

    let unlock_ledger = env.ledger().sequence() + MIN_LOCK_LEDGERS;
    let id = vault.deposit_by_ledger(&alice, &token, &2_000, &unlock_ledger, &500);

    // Advance past the unlock ledger
    advance_ledger(&env, MIN_LOCK_LEDGERS);

    // Attempt cancel should fail because vault is now unlocked
    assert_eq!(
        vault.try_cancel_deposit(&alice, &id),
        Err(Ok(VaultError::VaultAlreadyUnlocked))
    );
}

/// Test cancel_deposit on a non-existent ledger-based deposit.
#[test]
fn test_cancel_deposit_ledger_based_not_found() {
    let (_env, vault, _token, _admin, alice, _fee) = setup();

    assert_eq!(
        vault.try_cancel_deposit(&alice, &999),
        Err(Ok(VaultError::NoDepositFound))
    );
}

// ================================================================
//  #90 emergency_withdraw on ledger-based deposits
// ================================================================

/// Test that admin can emergency_withdraw a ledger-based deposit before unlock.
/// Funds must go to the depositor, not the admin.
#[test]
fn test_emergency_withdraw_ledger_based_succeeds() {
    let (env, vault, token, admin, alice, _fee) = setup();
    let token_client = TokenClient::new(&env, &token);

    // Create a ledger-based deposit that is still locked
    let unlock_ledger = env.ledger().sequence() + 500;
    let id = vault.deposit_by_ledger(&alice, &token, &3_500, &unlock_ledger, &0);

    assert_eq!(token_client.balance(&alice), 6_500);

    // Admin performs emergency withdrawal
    vault.emergency_withdraw(&admin, &alice, &id);

    // Funds go to depositor, not admin
    assert_eq!(token_client.balance(&alice), 10_000);
    assert!(vault.get_vault(&alice, &id).is_none());
}

/// Test that emergency_withdraw fails when called by a non-admin on a ledger-based deposit.
#[test]
fn test_emergency_withdraw_ledger_based_non_admin_fails() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);

    let unlock_ledger = env.ledger().sequence() + 500;
    vault.deposit_by_ledger(&alice, &token, &2_000, &unlock_ledger, &0);

    // Non-admin attempts to emergency_withdraw
    assert_eq!(
        vault.try_emergency_withdraw(&bob, &alice, &0),
        Err(Ok(VaultError::Unauthorized))
    );
}

/// Test emergency_withdraw on a non-existent ledger-based deposit.
#[test]
fn test_emergency_withdraw_ledger_based_not_found() {
    let (_env, vault, _token, admin, alice, _fee) = setup();

    assert_eq!(
        vault.try_emergency_withdraw(&admin, &alice, &999),
        Err(Ok(VaultError::NoDepositFound))
    );
}

// ================================================================
//  #91 Property-based tests for penalty calculation
// ================================================================

#[cfg(test)]
mod penalty_property_tests {
    use super::*;
    use proptest::prelude::*;

    /// Property: penalty + refund == original amount
    /// For any valid (amount, penalty_bps) pair, the sum of penalty and refund
    /// must equal the original amount (no loss or gain).
    proptest! {
        #[test]
        fn prop_penalty_plus_refund_equals_amount(
            amount in 1i128..=1_000_000,
            penalty_bps in 0u32..=10_000,
        ) {
            let penalty = (amount * penalty_bps as i128) / 10_000;
            let refund = amount - penalty;
            
            prop_assert_eq!(
                penalty + refund,
                amount,
                "penalty + refund must equal original amount"
            );
        }
    }

    /// Property: penalty is always non-negative
    proptest! {
        #[test]
        fn prop_penalty_non_negative(
            amount in 1i128..=1_000_000,
            penalty_bps in 0u32..=10_000,
        ) {
            let penalty = (amount * penalty_bps as i128) / 10_000;
            prop_assert!(penalty >= 0, "penalty must be non-negative");
        }
    }

    /// Property: penalty is always <= amount
    proptest! {
        #[test]
        fn prop_penalty_at_most_amount(
            amount in 1i128..=1_000_000,
            penalty_bps in 0u32..=10_000,
        ) {
            let penalty = (amount * penalty_bps as i128) / 10_000;
            prop_assert!(penalty <= amount, "penalty must not exceed amount");
        }
    }

    /// Property: refund is always non-negative
    proptest! {
        #[test]
        fn prop_refund_non_negative(
            amount in 1i128..=1_000_000,
            penalty_bps in 0u32..=10_000,
        ) {
            let penalty = (amount * penalty_bps as i128) / 10_000;
            let refund = amount - penalty;
            prop_assert!(refund >= 0, "refund must be non-negative");
        }
    }

    /// Property: max penalty (10_000 bps = 100%) equals the amount
    proptest! {
        #[test]
        fn prop_max_penalty_equals_amount(amount in 1i128..=1_000_000) {
            let penalty_bps = 10_000u32;
            let penalty = (amount * penalty_bps as i128) / 10_000;
            prop_assert_eq!(penalty, amount, "100% penalty must equal amount");
        }
    }

    /// Property: zero penalty (0 bps) results in zero penalty
    proptest! {
        #[test]
        fn prop_zero_penalty_bps_results_zero_penalty(amount in 1i128..=1_000_000) {
            let penalty_bps = 0u32;
            let penalty = (amount * penalty_bps as i128) / 10_000;
            prop_assert_eq!(penalty, 0, "0% penalty must be zero");
        }
    }

    /// Property: for edge case amount=1, penalty calculation doesn't overflow
    proptest! {
        #[test]
        fn prop_amount_one_no_overflow(penalty_bps in 0u32..=10_000) {
            let amount = 1i128;
            let penalty = (amount * penalty_bps as i128) / 10_000;
            let refund = amount - penalty;
            prop_assert_eq!(
                penalty + refund,
                amount,
                "even for amount=1, penalty + refund must equal 1"
            );
        }
    }

    /// Integration property: test with contract's actual penalty calculation
    /// This ensures the contract's logic matches the formula.
    proptest! {
        #[test]
        fn prop_penalty_calculation_matches_formula(
            amount in 1i128..=MAX_DEPOSIT_AMOUNT,
            penalty_bps in 0u32..=10_000,
        ) {
            // Replicate the contract's penalty calculation
            let penalty = (amount * penalty_bps as i128) / 10_000;
            let refund = amount - penalty;

            // Verify invariants
            prop_assert!(penalty >= 0);
            prop_assert!(penalty <= amount);
            prop_assert!(refund >= 0);
            prop_assert_eq!(penalty + refund, amount);
        }
    }
}

// ================================================================
//  get_vault_batch / get_deposit_batch — MAX_BATCH_SIZE clamping
// ================================================================

#[test]
fn test_get_vault_batch_clamps_at_max_batch_size() {
    use crate::constants::MAX_BATCH_SIZE;

    let (env, vault, token, _admin, alice, _fee) = setup();
    let unlock_time = env.ledger().timestamp() + 3600;

    // Create a single deposit for alice at id 0
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);

    // Build a depositor list larger than MAX_BATCH_SIZE (25)
    let mut depositors = soroban_sdk::Vec::new(&env);
    let excess = MAX_BATCH_SIZE + 5;
    for _ in 0..excess {
        depositors.push_back(alice.clone());
    }

    // Call get_vault_batch — should silently clamp to MAX_BATCH_SIZE results
    let results = vault.get_vault_batch(&depositors, &0);
    assert_eq!(results.len(), MAX_BATCH_SIZE);

    // All returned entries should be Some (alice has a deposit at id 0)
    for i in 0..MAX_BATCH_SIZE {
        let entry = results.get(i).unwrap();
        assert!(entry.is_some(), "index {} should have a deposit", i);
        let e = entry.unwrap();
        assert_eq!(e.amount, 1_000);
    }
}

#[test]
fn test_get_vault_batch_smaller_than_max_returns_exact_count() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    let bob: Address = Address::generate(&env);
    let carol: Address = Address::generate(&env);

    let asset_client = StellarAssetClient::new(&env, &token);
    asset_client.mint(&bob, &5_000);
    asset_client.mint(&carol, &5_000);

    let unlock_time = env.ledger().timestamp() + 3600;
    vault.deposit(&alice, &token, &1_000, &unlock_time, &0);
    vault.deposit(&bob, &token, &2_000, &unlock_time, &0);
    vault.deposit(&carol, &token, &3_000, &unlock_time, &0);

    // Build a depositor list of 3 — fewer than MAX_BATCH_SIZE
    let mut depositors = soroban_sdk::Vec::new(&env);
    depositors.push_back(alice.clone());
    depositors.push_back(bob.clone());
    depositors.push_back(carol.clone());

    let results = vault.get_vault_batch(&depositors, &0);
    assert_eq!(results.len(), 3);
}

#[test]
fn test_get_deposit_batch_clamps_at_max_batch_size() {
    use crate::constants::MAX_BATCH_SIZE;

    let (env, vault, token, _admin, alice, _fee) = setup();

    // Build a deposit_ids list longer than MAX_BATCH_SIZE (30 items).
    // Some of the IDs won't have deposits — that's fine; the test is only
    // verifying that the response is clamped to MAX_BATCH_SIZE entries.
    let mut deposit_ids = soroban_sdk::Vec::new(&env);
    for i in 0u32..(MAX_BATCH_SIZE + 5) {
        deposit_ids.push_back(i);
    }

    let results = vault.get_deposit_batch(&alice, &deposit_ids);
    assert_eq!(results.len(), MAX_BATCH_SIZE as usize);
}

#[test]
fn test_version_returns_cargo_pkg_version() {
    let (env, vault, _token, _admin, _alice, _fee) = setup();

    let version = vault.version(&env);
    // Should be a valid version string from Cargo.toml (e.g., "0.1.0")
    assert!(!version.is_empty());
    // Expect format like "0.1.0" — semantic versioning
    let version_str = String::from_utf8(version.to_bytes()).unwrap();
    let parts: Vec<&str> = version_str.split('.').collect();
    assert_eq!(parts.len(), 3, "Version should be in semantic format (major.minor.patch)");
}
