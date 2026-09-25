# SAFE-HAVEN Watchlist Feature

## Quick Start

The watchlist feature is now fully implemented in SAFE-HAVEN. Users can:
- **Add deposits** to their watchlist to monitor (own or others')
- **Remove deposits** from their watchlist
- **Query watchlist** to view all monitored deposits
- **Receive events** when watchlist changes

## Key Files

### Implementation
- `contracts/safe-haven/src/contract.rs` — Public API (3 functions)
- `contracts/safe-haven/src/storage.rs` — Storage layer (4 functions)
- `contracts/safe-haven/src/events.rs` — Event emission (2 functions)
- `contracts/safe-haven/src/types.rs` — WatchlistEntry struct
- `contracts/safe-haven/src/constants.rs` — MAX_WATCHLIST_SIZE = 100
- `contracts/safe-haven/src/errors.rs` — Error codes 15-17
- `contracts/safe-haven/src/test.rs` — 14 comprehensive tests

### Documentation
- **WATCHLIST_IMPLEMENTATION.md** — Full architecture & API reference
- **WATCHLIST_CHANGES_SUMMARY.md** — Detailed implementation details
- **WATCHLIST_API_EXAMPLES.md** — Code examples & integration patterns

## API Reference

### Add to Watchlist
```rust
pub fn add_to_watchlist(
    env: Env,
    subscriber: Address,      // User managing watchlist
    depositor: Address,       // Owner of deposit
    deposit_id: u32          // Deposit to watch
) -> Result<(), VaultError>
```
**Returns:** `Ok(())`, `WatchlistFull`, `DepositAlreadyWatched`, or `Unauthorized`

### Remove from Watchlist
```rust
pub fn remove_from_watchlist(
    env: Env,
    subscriber: Address,      // User managing watchlist
    depositor: Address,       // Owner of deposit
    deposit_id: u32          // Deposit to unwatch
) -> Result<(), VaultError>
```
**Returns:** `Ok(())`, `DepositNotWatched`, or `Unauthorized`

### Get Watchlist
```rust
pub fn get_watchlist(
    env: Env,
    subscriber: Address       // User's watchlist
) -> Vec<WatchlistEntry>
```
**Returns:** Vector of `{depositor: Address, deposit_id: u32}` (no auth required)

## Constants

```rust
MAX_WATCHLIST_SIZE: u32 = 100  // Max deposits per user's watchlist
```

## Error Codes

| Code | Name | Meaning |
|------|------|---------|
| 15 | `WatchlistFull` | Watchlist has 100 entries (limit reached) |
| 16 | `DepositAlreadyWatched` | Deposit already in watchlist |
| 17 | `DepositNotWatched` | Deposit not in watchlist |

## Events

**"watch_add"** — Emitted when deposit added to watchlist
- Topics: `(Symbol("watch_add"), subscriber)`
- Data: `(depositor, deposit_id)`

**"watch_rmv"** — Emitted when deposit removed from watchlist
- Topics: `(Symbol("watch_rmv"), subscriber)`
- Data: `(depositor, deposit_id)`

## Data Structures

```rust
#[contracttype]
pub struct WatchlistEntry {
    pub depositor: Address,    // Owner of watched deposit
    pub deposit_id: u32,       // Deposit ID
}
```

## Security Properties

✓ **Auth-First** — Requires subscriber authentication before add/remove
✓ **Size Limited** — MAX_WATCHLIST_SIZE = 100 prevents DOS
✓ **Per-User** — Each user has isolated watchlist
✓ **Duplicate Prevention** — Cannot add same deposit twice
✓ **Public Reads** — get_watchlist requires no auth (public data)

## Test Coverage

14 comprehensive tests covering:
- Basic add/remove functionality
- Duplicate detection
- Size limit enforcement (100 max)
- Multiple deposits per user
- Multiple depositors per subscriber
- Per-user isolation
- Event emission
- Authentication enforcement
- Edge cases (empty, full, invalid removes)

**Location:** `contracts/safe-haven/src/test.rs` (lines 1984-2272)

## Usage Example

```rust
// Add to watchlist
vault.add_to_watchlist(&subscriber, &depositor, &deposit_id)?;

// View watchlist
let watchlist = vault.get_watchlist(&subscriber);
for entry in watchlist.iter() {
    println!("Watching deposit {} by {}", entry.deposit_id, entry.depositor);
}

// Remove from watchlist
vault.remove_from_watchlist(&subscriber, &depositor, &deposit_id)?;
```

## Performance

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| add_to_watchlist | O(n) | n ≤ 100; includes duplicate check |
| remove_from_watchlist | O(n) | n ≤ 100; includes validation |
| get_watchlist | O(1) | Single storage read |

## Storage

- **Key:** `VaultKey::Watchlist(subscriber)`
- **Value:** `Vec<WatchlistEntry>`
- **Max per user:** 100 entries (~3.2 KB)
- **TTL:** Extended to BUMP_TARGET (covers max lock duration)

## Deployment

Before deploying:
1. Run tests: `make test` (ensures 14 watchlist tests pass)
2. Build: `make build`
3. Optimize: `make optimize`
4. Verify size: `make check-wasm-size`

## Integration

### For Frontend
```javascript
// Add to watchlist
await vault.add_to_watchlist(userAddress, depositorAddress, depositId);

// Get watchlist
const watchlist = await vault.get_watchlist(userAddress);

// Remove from watchlist
await vault.remove_from_watchlist(userAddress, depositorAddress, depositId);
```

### For Off-Chain Indexers
Listen to events and maintain index of `(subscriber, depositor, deposit_id)` tuples for notifications.

## Documentation Files

1. **WATCHLIST_IMPLEMENTATION.md**
   - Complete architecture overview
   - API reference with detailed signatures
   - Storage schema and TTL management
   - Security analysis
   - Performance characteristics

2. **WATCHLIST_CHANGES_SUMMARY.md**
   - Implementation details per file
   - Code snippets
   - Complexity analysis
   - Integration points
   - Deployment notes

3. **WATCHLIST_API_EXAMPLES.md**
   - Function signatures with examples
   - Error handling patterns
   - Test examples
   - Frontend integration example
   - Off-chain indexer integration

## Acceptance Criteria

✅ **Users can add deposits to their watchlist**
- Function: `add_to_watchlist(subscriber, depositor, deposit_id)`

✅ **Watchlist limited to MAX_WATCHLIST_SIZE (100) entries**
- Constant: `MAX_WATCHLIST_SIZE = 100`
- Enforced with `WatchlistFull` error

✅ **Events include watchlist subscriber information**
- Events: "watch_add" and "watch_rmv" with subscriber in topics

✅ **get_watchlist() returns user's monitored deposits**
- Function: `get_watchlist(subscriber) -> Vec<WatchlistEntry>`

✅ **Users can remove deposits from watchlist**
- Function: `remove_from_watchlist(subscriber, depositor, deposit_id)`

✅ **Tests verify watchlist management and size limits**
- 14 comprehensive test cases covering all scenarios

## Next Steps (Out of Scope)

Future enhancements could include:
- Off-chain notification delivery
- Watchlist sharing between users
- Conditional alerts based on deposit events
- Watchlist pagination
- Deposit categorization/tagging

---

**Status:** ✅ Complete and Ready for Deployment

For detailed information, see:
- `WATCHLIST_IMPLEMENTATION.md` for architecture
- `WATCHLIST_CHANGES_SUMMARY.md` for implementation details
- `WATCHLIST_API_EXAMPLES.md` for usage examples
