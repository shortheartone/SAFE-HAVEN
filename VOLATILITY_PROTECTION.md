# SAFE-HAVEN Volatility Protection Mechanism

## Overview

The volatility protection mechanism allows users to set minimum value guarantees on deposits, protecting against price fluctuations during the lock period. This ensures price-sensitive depositors can commit their funds without catastrophic loss exposure.

## Key Features

### 1. Oracle Configuration (Admin Function)
- **`configure_oracle(admin, token, oracle)`** - Configure an oracle for a token
- **`remove_oracle(admin, token)`** - Remove oracle configuration
- Only the contract admin can configure/remove oracles
- Enables value verification on withdrawal

### 2. Deposit with Value Guarantee
All deposit functions now support `min_value_guarantee` parameter:

- **`deposit(depositor, token, amount, unlock_time, penalty_bps, min_value_guarantee)`**
- **`deposit_for(payer, depositor, token, amount, unlock_time, penalty_bps, min_value_guarantee)`**
- **`deposit_by_ledger(depositor, token, amount, unlock_ledger, penalty_bps, min_value_guarantee)`**

#### Requirements:
- If `min_value_guarantee > 0`, an oracle **must** be configured for the token
- If `min_value_guarantee == 0`, guarantee is disabled (no oracle needed)
- Violation: Returns `VaultError::OracleNotConfigured`

### 3. Value Verification During Withdrawal

When a user withdraws a deposit with `min_value_guarantee > 0`:

1. **Check Current Value** - Contract queries oracle price
2. **Calculate Shortfall** - If current_value < min_value_guarantee:
   - `shortfall = min_value_guarantee - current_value`
3. **Cover from Protection Fund** - Contract attempts to cover shortfall from fund
   - If protection fund balance < shortfall: Returns `InsufficientProtectionFund`
   - If sufficient: Withdraws shortfall from protection fund
4. **Return Guaranteed Value** - User receives the guaranteed minimum value
5. **Emit Event** - `ValueGuaranteeTriggered` event is published

#### Implementation Details:
- **`withdraw(depositor, deposit_id)`** - Handles value guarantee for timestamp-based deposits
- **`withdraw_to(depositor, deposit_id, recipient)`** - Same for alternate recipient
- Helper functions:
  - `handle_value_guarantee()` - Processes timestamp-based deposits
  - `handle_value_guarantee_ledger()` - Processes ledger-based deposits

### 4. Protection Fund Management

#### Storage:
- **`ProtectionFundBalance`** - Total reserve balance in storage

#### Functions:
- **`add_to_protection_fund(amount)`** - Add funds to protection reserve
- **`withdraw_from_protection_fund(amount)`** - Withdraw funds for shortfall coverage
- **`get_protection_fund_balance()`** - Query current balance

#### Fund Behavior:
- Fund starts at 0 (can be funded externally or via contract mechanisms)
- Only decreases when covering value shortfalls
- Publishes `ProtectionFundUpdated` event on changes

### 5. Query Functions (Read-only)

#### Oracle Queries:
- **`get_oracle(token)`** - Retrieve configured oracle for a token

#### Value Queries:
- **`get_vault_current_value(depositor, deposit_id)`** - Current value of timestamp deposit
- **`get_ledger_vault_current_value(depositor, deposit_id)`** - Current value of ledger deposit
- **`get_vault_min_guarantee(depositor, deposit_id)`** - Retrieve min guarantee for timestamp deposit
- **`get_ledger_vault_min_guarantee(depositor, deposit_id)`** - Retrieve min guarantee for ledger deposit

#### Protection Fund Query:
- **`get_protection_fund_balance()`** - Current reserve balance

## Error Codes

### New Error Types:

| Code | Name | Meaning |
|------|------|---------|
| 15 | `OracleNotConfigured` | Attempted to deposit with guarantee but no oracle configured |
| 16 | `InsufficientProtectionFund` | Protection fund cannot cover the value shortfall |
| 17 | `InvalidOracleData` | Oracle returned invalid/malformed price data |
| 18 | `ValueDropDetected` | Reserved for future use; currently emitted in guarantee triggers |

## Events

### New Events:

#### `OracleConfigured`
```
Topics: (symbol!("oracle_conf"), token)
Data: (admin, oracle)
```
Emitted when an oracle is configured for a token.

#### `ValueGuaranteeTriggered`
```
Topics: (symbol!("val_guarantee"), depositor, token)
Data: (shortfall_amount, deposit_id)
```
Emitted when a value guarantee shortfall is covered from the protection fund.

#### `ProtectionFundUpdated`
```
Topics: (symbol!("prot_fund_upd"),)
Data: (new_balance, change_amount, is_addition)
```
Emitted when the protection fund balance changes.

## Storage Schema Changes

### New VaultKey Variants:
```rust
Oracle(Address)              // Maps token -> oracle address
ProtectionFundBalance        // Total protection fund reserve (i128)
```

### Extended VaultEntry:
```rust
pub struct VaultEntry {
    pub token: Address,
    pub amount: i128,
    pub unlock_time: u64,
    pub depositor: Address,
    pub penalty_bps: u32,
    pub min_value_guarantee: i128,  // NEW: Min value guarantee (0 = disabled)
}
```

### Extended LedgerVaultEntry:
```rust
pub struct LedgerVaultEntry {
    pub token: Address,
    pub amount: i128,
    pub unlock_ledger: u32,
    pub depositor: Address,
    pub penalty_bps: u32,
    pub min_value_guarantee: i128,  // NEW: Min value guarantee (0 = disabled)
}
```

## Usage Example

```rust
// 1. Admin configures oracle for a token
let oracle = Address::generate(&env);
vault.configure_oracle(&admin, &token, &oracle);

// 2. User deposits with value guarantee
let unlock_time = env.ledger().timestamp() + 86400;
let min_guarantee = 900; // Guarantee at least 900 units
let deposit_id = vault.deposit(
    &alice,      // depositor
    &token,      // token
    &1000,       // amount
    &unlock_time,
    &0,          // no early exit penalty
    &min_guarantee
);

// 3. Time passes, price potentially changes
advance_time(&env, 86401);

// 4. User withdraws - guarantee is checked and honored if needed
vault.withdraw(&alice, &deposit_id);
// → User receives at least 900 units even if price dropped
// → Shortfall (if any) is covered from protection fund
```

## Design Principles

### 1. Optional & Backward Compatible
- Min value guarantee defaults to 0 (disabled)
- Existing contracts continue working without changes
- Oracle configuration is opt-in per token

### 2. Security
- Oracle configuration is admin-only
- Value shortfall coverage requires sufficient protection fund
- All value checks happen before token transfers (checks-effects-interactions pattern)

### 3. Efficiency
- Oracle storage is O(1) per token
- Protection fund balance is a single i128 value
- No enumeration overhead

### 4. Transparency
- All guarantee triggers emit events
- Fund balance queryable on-chain
- Oracle addresses publicly retrievable

## Limitations & Future Enhancements

### Current Implementation:
- **Simplified oracle integration** - `get_oracle_price()` is a placeholder
- **No real price conversion** - Current value assumes price hasn't changed
- **No liquidation mechanism** - Protection fund must be externally funded

### Future Enhancements:
1. **Real Oracle Integration** - Connect to Stellar price feeds or other oracle networks
2. **Multi-token Baskets** - Support composite value guarantees across multiple tokens
3. **Dynamic Guarantees** - Adjust guarantees based on market conditions
4. **Auto-funding** - Mechanism to automatically top-up protection fund
5. **Liquidation Auctions** - Allow arbitrage to replenish protection fund
6. **Insurance Pool** - Community-managed insurance for guarantee coverage

## Testing

Comprehensive test suite includes 27 tests covering:
- Oracle configuration (admin-only, removal, authorization)
- Protection fund operations
- Deposits with/without guarantees
- Value verification during withdrawal
- Query functions
- Ledger-based deposits
- Withdraw-to operations
- Edge cases (zero guarantee, multiple deposits, missing oracle)

Run tests with: `cargo test --features testutils`

## Migration Notes

No migration required for existing contracts. The new `min_value_guarantee` field defaults to 0, maintaining backward compatibility with existing deposits and vault entries.

For future schema changes, use the existing `migrate()` function with updated `STORAGE_VERSION`.
