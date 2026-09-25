# Liquidation Protection - API Reference

## Quick Start

```rust
// 1. Enable protection on a deposit
vault.enable_liquidation_protection(
    &depositor,
    &deposit_id,
    &collateral_amount,
    &0,  // default liquidation threshold (1.5x)
    &0,  // default warning threshold (2.0x)
    &0,  // default grace period (7 days)
)?;

// 2. Check health at any time
let health = vault.get_health_ratio(&depositor, &deposit_id)?;
println!("Health: {}x, Status: {:?}", 
    health.ratio_bps as f64 / 10000.0, 
    health.status);

// 3. Add collateral if needed
let new_health = vault.add_collateral_for_deposit(
    &depositor,
    &deposit_id,
    &additional_amount
)?;

// 4. Clean up when done
vault.remove_liquidation_protection(&depositor, &depositor, &deposit_id)?;
```

## Contract Functions

### Enabling Protection

#### `enable_liquidation_protection()`
Activate liquidation protection for a collateralized deposit.

**Signature**
```rust
pub fn enable_liquidation_protection(
    env: Env,
    depositor: Address,           // Must sign
    deposit_id: u32,
    collateral_amount: i128,
    liquidation_threshold_bps: u32,  // 0 = default (15000)
    warning_threshold_bps: u32,      // 0 = default (20000)
    grace_period_secs: u64,          // 0 = default (604800)
) -> Result<(), VaultError>
```

**Parameters**
| Name | Type | Description |
|------|------|-------------|
| `depositor` | Address | Owner of deposit (must sign transaction) |
| `deposit_id` | u32 | ID of deposit to protect |
| `collateral_amount` | i128 | Amount of collateral backing deposit (must be > 0) |
| `liquidation_threshold_bps` | u32 | Liquidation trigger threshold. 0 = use default (15000 = 1.5x). Must be 10000-50000. |
| `warning_threshold_bps` | u32 | Warning trigger threshold. 0 = use default (20000 = 2.0x). Must be 10000-50000 and > liquidation threshold. |
| `grace_period_secs` | u64 | Grace period duration. 0 = use default (604800 = 7 days). Must be 3600-2592000. |

**Returns**
- `Ok(())` - Protection successfully enabled
- `Err(InvalidAmount)` - Collateral amount ≤ 0
- `Err(NoDepositFound)` - Deposit doesn't exist
- `Err(InvalidLiquidationThreshold)` - Threshold outside valid range
- `Err(InvalidGracePeriod)` - Grace period outside valid range

**Events Emitted**
- `LiquidationProtected` with collateral amount, thresholds, and grace period

**Example**
```rust
// Enable with defaults
vault.enable_liquidation_protection(
    &alice,
    &1,      // deposit_id
    &2500,   // 2.5x collateral backing 1000-unit deposit
    &0, &0, &0
)?;

// Enable with custom thresholds
vault.enable_liquidation_protection(
    &alice,
    &1,
    &2500,
    &12000,  // 1.2x liquidation (aggressive)
    &18000,  // 1.8x warning (early alert)
    &604800  // 7 day grace period
)?;
```

---

### Monitoring Health

#### `get_health_ratio()`
Calculate and return the current health ratio of a protected deposit.

**Signature**
```rust
pub fn get_health_ratio(
    env: Env,
    depositor: Address,
    deposit_id: u32,
) -> Result<HealthRatio, VaultError>
```

**Returns**
```rust
pub struct HealthRatio {
    pub ratio_bps: u32,                    // Health in basis points
    pub status: HealthStatus,              // Current health state
    pub collateral_amount: i128,           // Current collateral
    pub deposit_amount: i128,              // Current deposit amount
    pub grace_period_remaining_secs: u64,  // Seconds left (0 if no grace)
}
```

**HealthStatus Enum**
```rust
pub enum HealthStatus {
    Healthy,          // ratio_bps > warning_threshold
    Warning,          // liquidation_threshold < ratio_bps <= warning_threshold
    CriticalRisk,     // ratio_bps <= liquidation_threshold (no grace)
    GracePeriod,      // ratio_bps <= liquidation_threshold (grace active)
    Liquidatable,     // Grace period has expired
}
```

**Errors**
- `LiquidationProtectionNotEnabled` - No protection on this deposit
- `NoDepositFound` - Deposit doesn't exist

**Example**
```rust
let health = vault.get_health_ratio(&alice, &deposit_id)?;

match health.status {
    HealthStatus::Healthy => {
        println!("Deposit is safe: {:.2}x health", 
            health.ratio_bps as f64 / 10000.0);
    }
    HealthStatus::Warning => {
        println!("Warning: Health dropping ({:.2}x)", 
            health.ratio_bps as f64 / 10000.0);
        println!("Consider adding collateral");
    }
    HealthStatus::CriticalRisk => {
        println!("CRITICAL: Must add collateral immediately!");
        println!("Health is only {:.2}x", 
            health.ratio_bps as f64 / 10000.0);
    }
    HealthStatus::GracePeriod => {
        println!("Grace period active: {} seconds remaining",
            health.grace_period_remaining_secs);
        println!("Add collateral before expiration!");
    }
    HealthStatus::Liquidatable => {
        println!("Grace period expired - liquidation imminent!");
    }
}
```

---

### Adding Collateral

#### `add_collateral_for_deposit()`
Increase collateral on a deposit to improve health ratio and prevent liquidation.

**Signature**
```rust
pub fn add_collateral_for_deposit(
    env: Env,
    depositor: Address,        // Must sign
    deposit_id: u32,
    additional_collateral_amount: i128,
) -> Result<u32, VaultError>
```

**Parameters**
| Name | Type | Description |
|------|------|-------------|
| `depositor` | Address | Owner of deposit (must sign) |
| `deposit_id` | u32 | ID of deposit |
| `additional_collateral_amount` | i128 | Amount to add (must be > 0) |

**Returns**
- `Ok(new_health_bps)` - New health ratio in basis points
- `Err(InvalidAmount)` - Amount ≤ 0
- `Err(LiquidationProtectionNotEnabled)` - No protection on deposit
- `Err(NoDepositFound)` - Deposit doesn't exist

**Behavior**
1. Adds `additional_collateral_amount` to current collateral
2. Recalculates health ratio
3. If new health > warning threshold, resets grace period
4. Emits `CollateralAdded` event

**Events Emitted**
- `CollateralAdded` with amount added, new total, new health

**Example**
```rust
// Current health is 1.2x (critical)
let new_health_bps = vault.add_collateral_for_deposit(
    &alice,
    &deposit_id,
    &1000  // Add 1000 more units
)?;

// New health calculated and stored
// If new health > warning threshold, grace period is cleared
assert_eq!(new_health_bps, 22_000); // 2.2x - back to healthy
```

---

### Status Queries

#### `has_liquidation_protection()`
Check if a deposit has liquidation protection enabled.

**Signature**
```rust
pub fn has_liquidation_protection(
    env: Env,
    depositor: Address,
    deposit_id: u32,
) -> bool
```

**Returns**
- `true` if protection is enabled
- `false` if protection not enabled or deposit doesn't exist

---

#### `get_liquidation_protection()`
Retrieve the full liquidation protection configuration.

**Signature**
```rust
pub fn get_liquidation_protection(
    env: Env,
    depositor: Address,
    deposit_id: u32,
) -> Option<LiquidationProtection>
```

**Returns**
```rust
pub struct LiquidationProtection {
    pub collateral_amount: i128,
    pub liquidation_threshold_bps: u32,
    pub warning_threshold_bps: u32,
    pub grace_period_secs: u64,
    pub grace_period_start: u64,              // 0 if not in grace
    pub warning_emitted: bool,
    pub last_grace_period_reset: u64,
}
```

---

#### `is_in_grace_period()`
Check if a deposit is currently in active grace period.

**Signature**
```rust
pub fn is_in_grace_period(
    env: Env,
    depositor: Address,
    deposit_id: u32,
) -> Result<bool, VaultError>
```

**Returns**
- `Ok(true)` if in active grace period
- `Ok(false)` if grace period not started or has expired
- `Err(LiquidationProtectionNotEnabled)` if no protection

---

#### `grace_period_remaining()`
Get the number of seconds remaining in the active grace period.

**Signature**
```rust
pub fn grace_period_remaining(
    env: Env,
    depositor: Address,
    deposit_id: u32,
) -> Result<u64, VaultError>
```

**Returns**
- `Ok(seconds)` - Seconds remaining (0 if no active grace)
- `Err(LiquidationProtectionNotEnabled)` if no protection

**Example**
```rust
let remaining = vault.grace_period_remaining(&alice, &deposit_id)?;
if remaining > 0 {
    let days = remaining / 86400;
    println!("{} days remaining to add collateral", days);
}
```

---

### Cleanup

#### `remove_liquidation_protection()`
Remove liquidation protection from a deposit.

**Signature**
```rust
pub fn remove_liquidation_protection(
    env: Env,
    depositor_or_admin: Address,  // Must sign - must be depositor or admin
    depositor: Address,
    deposit_id: u32,
) -> Result<(), VaultError>
```

**Authorization**
- Only depositor or admin can call this
- Must sign the transaction

**Returns**
- `Ok(())` - Protection successfully removed
- `Err(Unauthorized)` - Caller is not depositor or admin
- `Err(NoDepositFound)` - Deposit doesn't exist

**Example**
```rust
// When withdrawing deposit
vault.withdraw(&alice, &deposit_id)?;
vault.remove_liquidation_protection(&alice, &alice, &deposit_id)?;
```

---

## Events

### `LiquidationProtected`
Emitted when protection is first enabled on a deposit.

```
Topics: ["liq_protected", depositor]
Data: (deposit_id, collateral_amount, liquidation_threshold_bps, warning_threshold_bps, grace_period_secs)
```

### `LiquidationWarning`
Emitted when health ratio drops below warning threshold.

```
Topics: ["liq_warning", depositor]
Data: (deposit_id, health_ratio_bps, warning_threshold_bps, collateral_amount, deposit_amount)
```

### `GracePeriodStarted`
Emitted when grace period begins (health falls to critical with grace).

```
Topics: ["grace_period_start", depositor]
Data: (deposit_id, grace_period_expires_at, health_ratio_bps)
```

### `CollateralAdded`
Emitted when collateral is added to a protected deposit.

```
Topics: ["collateral_added", depositor]
Data: (deposit_id, additional_collateral, new_collateral_total, new_health_ratio_bps)
```

### `LiquidationExecuted`
Emitted when liquidation is executed on a deposit.

```
Topics: ["liquidation_exec", depositor]
Data: (deposit_id, collateral_seized, liquidation_fee)
```

---

## Error Codes

| Code | Name | Cause |
|------|------|-------|
| 27 | `InsufficientCollateral` | Collateral not enough to meet health requirement |
| 28 | `NotCollateralized` | Deposit has no collateral |
| 29 | `LiquidationProtectionNotEnabled` | Protection not enabled on this deposit |
| 30 | `NoGracePeriodActive` | Grace period not currently active |
| 31 | `GracePeriodNotExpired` | Grace period still active, liquidation can't proceed |
| 32 | `InvalidLiquidationThreshold` | Threshold not in range 1.0x - 5.0x |
| 33 | `InvalidGracePeriod` | Grace period not in range 1 hr - 30 days |

---

## Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `DEFAULT_LIQUIDATION_THRESHOLD_BPS` | 15,000 | 1.5x default liquidation threshold |
| `DEFAULT_WARNING_THRESHOLD_BPS` | 20,000 | 2.0x default warning threshold |
| `DEFAULT_GRACE_PERIOD_SECS` | 604,800 | 7 days default grace period |
| `MIN_GRACE_PERIOD_SECS` | 3,600 | 1 hour minimum grace period |
| `MAX_GRACE_PERIOD_SECS` | 2,592,000 | 30 days maximum grace period |

---

## Threshold Guidelines

### Recommended Configurations

**Conservative (Low Risk)**
- Liquidation Threshold: 2.0x (20,000 bps)
- Warning Threshold: 2.5x (25,000 bps)
- Grace Period: 14 days (1,209,600 secs)

**Standard (Balanced)**
- Liquidation Threshold: 1.5x (15,000 bps)
- Warning Threshold: 2.0x (20,000 bps)
- Grace Period: 7 days (604,800 secs)

**Aggressive (High Risk)**
- Liquidation Threshold: 1.2x (12,000 bps)
- Warning Threshold: 1.5x (15,000 bps)
- Grace Period: 3 days (259,200 secs)

---

## Example Workflows

### Scenario 1: Protecting a Fixed Deposit

```rust
// Deposit 1000 tokens, locked for 30 days
let deposit_id = vault.deposit(
    &alice,
    &token,
    &1000,
    &(now + 2_592_000),  // 30 days
    &0
)?;

// Protect with 2.0x collateral
vault.enable_liquidation_protection(
    &alice,
    &deposit_id,
    &2000,  // 2.0x backing
    &0, &0, &0  // defaults
)?;

// Monitor periodically
let health = vault.get_health_ratio(&alice, &deposit_id)?;
println!("Health: {:?}", health.status);
```

### Scenario 2: Recovering from Critical Health

```rust
// Check current state
let health = vault.get_health_ratio(&alice, &deposit_id)?;
// status = CriticalRisk, ratio_bps = 12000

// Grace period automatically triggered
let in_grace = vault.is_in_grace_period(&alice, &deposit_id)?;  // true
let remaining = vault.grace_period_remaining(&alice, &deposit_id)?;
// remaining = 604800 (7 days)

// Add collateral before expiration
let new_health = vault.add_collateral_for_deposit(
    &alice,
    &deposit_id,
    &3000  // Add 3000 more units
)?;
// new_health = 50000 (5.0x) - well above warning

// Grace period automatically reset
let in_grace = vault.is_in_grace_period(&alice, &deposit_id)?;  // false
```

### Scenario 3: Dynamic Collateral Management

```rust
// Get current health
let health = vault.get_health_ratio(&alice, &deposit_id)?;
match health.status {
    HealthStatus::Healthy => {
        println!("No action needed");
    }
    HealthStatus::Warning => {
        // Preemptively add 10% more collateral
        vault.add_collateral_for_deposit(&alice, &deposit_id, 
            &(health.collateral_amount / 10))?;
    }
    HealthStatus::CriticalRisk | HealthStatus::GracePeriod => {
        // Emergency: double the collateral
        vault.add_collateral_for_deposit(&alice, &deposit_id,
            &health.collateral_amount)?;
    }
    HealthStatus::Liquidatable => {
        eprintln!("Too late - liquidation will proceed");
    }
}
```

---

## Deployment Checklist

- [ ] Rust compiled successfully
- [ ] Unit tests passing
- [ ] Integration tests passing
- [ ] No warnings from clippy
- [ ] Documentation reviewed
- [ ] Security audit completed
- [ ] Testnet deployment verified
- [ ] Mainnet deployment ready

