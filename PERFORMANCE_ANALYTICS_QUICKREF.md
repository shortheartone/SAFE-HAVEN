# Performance Analytics Quick Reference

## Query Performance Metrics

```rust
// Get comprehensive performance summary for a depositor
let benchmark = BenchmarkIndex::ContractDefault;
let result = vault.get_performance_metrics(&depositor_address, &benchmark)?;

// Access the results
let summary = result;
println!("Total deposits: {}", summary.deposit_count);
println!("Portfolio value: {} stroops", summary.total_current_value);
println!("Total gain: {} stroops", summary.total_absolute_gain);
println!("Average return: {:.2}%", summary.weighted_avg_return_bps as f64 / 100.0);
```

## Available Benchmarks

```rust
BenchmarkIndex::ContractDefault    // 5% per annum (matches contract's default rate)
BenchmarkIndex::StellarInflation   // 1% per annum (Stellar network inflation)
BenchmarkIndex::MoneyMarket        // 2% per annum (money market rates)
BenchmarkIndex::SAndP500           // 10% per annum (S&P 500 average)
BenchmarkIndex::Custom(300)        // Custom rate (300 bps = 3%)
```

## Key Metrics

### Per-Deposit Metrics (DepositPerformance)
- `principal` — Original deposit amount
- `current_value` — Current value including accrued interest
- `absolute_gain` — Total gain in token units
- `return_bps` — Return percentage in basis points (0-10000 = 0-100%)
- `time_weighted_return_bps` — Annualized return accounting for duration
- `compound_interest` — Interest earned through compounding
- `is_unlocked` — Whether deposit is past unlock time
- `time_remaining_secs` — Seconds until unlock (0 if unlocked)
- `benchmark_return_bps` — Expected return from benchmark
- `outperformance_bps` — Over/underperformance vs benchmark
- `return_on_gas` — Return per unit of estimated gas cost

### Aggregated Metrics (DepositorPerformanceSummary)
- `deposit_count` — Number of active deposits
- `total_principal` — Sum of all principal amounts
- `total_current_value` — Sum of all current values
- `total_absolute_gain` — Total portfolio gain
- `weighted_avg_return_bps` — Weighted average return rate
- `avg_time_weighted_return_bps` — Average annualized return
- `total_compound_interest` — Total compound interest earned
- `unlocked_deposit_count` — Number of unlocked deposits
- `total_unlocked_value` — Value available from unlocked deposits
- `avg_benchmark_return_bps` — Average benchmark rate
- `portfolio_outperformance_bps` — Portfolio vs benchmark performance

## Calculations

### Return Percentage
```
return_bps = (gain / principal) × 10,000
```
Example: 5% gain = 500 basis points

### Time-Weighted Return (Annualized)
```
twr_bps = (return_bps / time_held_seconds) × 31,536,000
```
Allows comparing deposits held for different durations

### Outperformance
```
outperformance_bps = deposit_return_bps - benchmark_return_bps
```
Positive = outperforming, negative = underperforming

### Return on Gas
```
rog = total_gain / estimated_gas_cost
```
Higher values = more efficient deposits

### Benchmark Return (Pro-rata)
```
benchmark_return_bps = benchmark_annual_rate_bps × (time_held / seconds_per_year)
```

## Examples

### Example 1: Single Deposit Performance
```rust
let summary = vault.get_performance_metrics(&alice, &BenchmarkIndex::ContractDefault)?;
assert_eq!(summary.deposit_count, 1);
assert_eq!(summary.total_principal, 1_000_000);
assert_eq!(summary.total_current_value, 1_050_000); // 5% gain
assert_eq!(summary.weighted_avg_return_bps, 500);    // 5% = 500 bps
```

### Example 2: Portfolio with Mixed Deposits
```rust
// Alice has 2 deposits
let summary = vault.get_performance_metrics(&alice, &BenchmarkIndex::SAndP500)?;
println!("Deposits: locked={}, unlocked={}", 
    summary.locked_deposit_count,
    summary.unlocked_deposit_count);
println!("Total portfolio value: {}", summary.total_current_value);
println!("vs S&P 500: {} bps", summary.portfolio_outperformance_bps);
```

### Example 3: Gas Efficiency Analysis
```rust
let summary = vault.get_performance_metrics(&alice, &BenchmarkIndex::ContractDefault)?;
let efficiency = summary.avg_return_on_gas; // gain per stroops spent
if efficiency > 100 {
    println!("High efficiency - good ROG");
} else if efficiency < 10 {
    println!("Low efficiency - consider larger deposits");
}
```

## Basis Points Quick Reference

| Percentage | Basis Points |
|-----------|-------------|
| 0.01% | 1 bps |
| 0.1% | 10 bps |
| 1% | 100 bps |
| 2% | 200 bps |
| 5% | 500 bps |
| 10% | 1,000 bps |

## Constants

- Annual interest rate: 5% (500 basis points)
- Seconds per year: 31,536,000
- Basis points per percent: 10,000
- Estimated deposit gas: 100,000 stroops
- Estimated withdrawal gas: 150,000 stroops

## Performance Considerations

- Function is read-only, safe to call repeatedly
- No state mutations
- Gas cost scales with number of active deposits (~50K instructions per 10 deposits)
- Time-weighted returns are annualized approximations
- Gas costs are estimates; actual may vary

## Troubleshooting

**Issue: Empty result for depositor with deposits**
- Check depositor address matches
- Verify deposits haven't been withdrawn
- Ensure deposit IDs are actively tracked

**Issue: Benchmark return seems wrong**
- Benchmark return is pro-rata based on time held
- Use time_weighted_return_bps for annualized comparison
- Custom benchmarks must be provided in basis points (0-10000)

**Issue: Return on gas very high**
- Verify large deposit amounts were used
- Check gas cost estimation assumptions
- Consider multiple smaller deposits for lower ROG
