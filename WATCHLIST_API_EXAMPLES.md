# Watchlist API Examples

## Contract Functions

### Add to Watchlist

```rust
pub fn add_to_watchlist(
    env: Env,
    subscriber: Address,      // User adding to watchlist
    depositor: Address,       // Owner of deposit
    deposit_id: u32,          // Deposit to watch
) -> Result<(), VaultError>
```

**Returns:**
- `Ok(())` on success
- `Err(VaultError::WatchlistFull)` if watchlist has 100 entries
- `Err(VaultError::DepositAlreadyWatched)` if already watching this deposit
- `Err(VaultError::Unauthorized)` if subscriber doesn't authorize

**Example Usage:**
```rust
let subscriber = Address::generate(&env);
let depositor = alice;
let deposit_id = 0u32;

vault.add_to_watchlist(&subscriber, &depositor, &deposit_id)?;
// Event emitted: ("watch_add", subscriber) → (depositor, deposit_id)
```

---

### Remove from Watchlist

```rust
pub fn remove_from_watchlist(
    env: Env,
    subscriber: Address,      // User removing from watchlist
    depositor: Address,       // Owner of deposit
    deposit_id: u32,          // Deposit to unwatch
) -> Result<(), VaultError>
```

**Returns:**
- `Ok(())` on success
- `Err(VaultError::DepositNotWatched)` if not in watchlist
- `Err(VaultError::Unauthorized)` if subscriber doesn't authorize

**Example Usage:**
```rust
vault.remove_from_watchlist(&subscriber, &depositor, &deposit_id)?;
// Event emitted: ("watch_rmv", subscriber) → (depositor, deposit_id)
```

---

### Get Watchlist

```rust
pub fn get_watchlist(
    env: Env,
    subscriber: Address       // User's watchlist to retrieve
) -> Vec<WatchlistEntry>
```

**Returns:**
- `Vec<WatchlistEntry>` with 0 to 100 entries
- Each entry: `{ depositor: Address, deposit_id: u32 }`
- **No auth required** — this is a public read-only query

**Example Usage:**
```rust
let watchlist = vault.get_watchlist(&subscriber);

for entry in watchlist.iter() {
    let depositor = entry.depositor;
    let id = entry.deposit_id;
    
    // Query the actual deposit if needed
    if let Some(vault_entry) = vault.get_vault(&depositor, &id) {
        println!(
            "Watching: {} staked {} until {}",
            depositor, vault_entry.amount, vault_entry.unlock_time
        );
    }
}
```

---

## Event Structure

### "watch_add" Event

**Topics:** `(Symbol("watch_add"), subscriber)`
**Data:** `(depositor, deposit_id)`

**Indexed by:**
- `subscriber` — enables filtering by user

**Example Event:**
```
Topics: ["watch_add", "G...subscriber"]
Data:   ("G...depositor", 0)
```

---

### "watch_rmv" Event

**Topics:** `(Symbol("watch_rmv"), subscriber)`
**Data:** `(depositor, deposit_id)`

**Indexed by:**
- `subscriber` — enables filtering by user

**Example Event:**
```
Topics: ["watch_rmv", "G...subscriber"]
Data:   ("G...depositor", 0)
```

---

## Error Handling

### WatchlistFull (Code 15)

Watchlist has reached MAX_WATCHLIST_SIZE (100 entries).

**Cause:** User has already added 100 deposits to watchlist

**Recovery:**
```rust
match vault.try_add_to_watchlist(&subscriber, &depositor, &id) {
    Err(Ok(VaultError::WatchlistFull)) => {
        // Need to remove something first
        vault.remove_from_watchlist(&subscriber, &old_depositor, &old_id)?;
        // Then try again
        vault.add_to_watchlist(&subscriber, &depositor, &id)?;
    }
    Ok(()) => println!("Added to watchlist"),
    _ => panic!("Unexpected error"),
}
```

---

### DepositAlreadyWatched (Code 16)

Deposit is already on this user's watchlist.

**Cause:** User already watching `(depositor, deposit_id)`

**Recovery:**
```rust
match vault.try_add_to_watchlist(&subscriber, &depositor, &id) {
    Err(Ok(VaultError::DepositAlreadyWatched)) => {
        println!("Already watching this deposit");
        // No action needed
    }
    Ok(()) => println!("Added to watchlist"),
    _ => panic!("Unexpected error"),
}
```

---

### DepositNotWatched (Code 17)

Deposit is not on this user's watchlist.

**Cause:** Trying to remove a deposit that was never added

**Recovery:**
```rust
match vault.try_remove_from_watchlist(&subscriber, &depositor, &id) {
    Err(Ok(VaultError::DepositNotWatched)) => {
        println!("Not watching this deposit");
        // Idempotent — already removed
    }
    Ok(()) => println!("Removed from watchlist"),
    _ => panic!("Unexpected error"),
}
```

---

## Test Examples

### Basic Add/Remove Flow

```rust
#[test]
fn test_watchlist_flow() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    StellarAssetClient::new(&env, &token).mint(&alice, &10_000);

    // Alice deposits 1000 units with 1 hour lock
    let unlock = env.ledger().timestamp() + 3600;
    let deposit_id = vault.deposit(&alice, &token, &1_000, &unlock, &0);

    // Bob adds it to his watchlist
    let bob = Address::generate(&env);
    vault.add_to_watchlist(&bob, &alice, &deposit_id);

    // Verify it's on his watchlist
    let watchlist = vault.get_watchlist(&bob);
    assert_eq!(watchlist.len(), 1);
    assert_eq!(watchlist.get(0).unwrap().depositor, alice);
    assert_eq!(watchlist.get(0).unwrap().deposit_id, deposit_id);

    // Bob removes it
    vault.remove_from_watchlist(&bob, &alice, &deposit_id);

    // Verify it's gone
    let watchlist = vault.get_watchlist(&bob);
    assert_eq!(watchlist.len(), 0);
}
```

---

### Size Limit Test

```rust
#[test]
fn test_watchlist_max_size() {
    let (env, vault, token, _admin, alice, _fee) = setup();
    StellarAssetClient::new(&env, &token).mint(&alice, &1_000_000);

    let bob = Address::generate(&env);
    
    // Add exactly 100 deposits (the maximum)
    for i in 0..100 {
        let unlock = env.ledger().timestamp() + 3600 + i as u64;
        let id = vault.deposit(&alice, &token, &1_000, &unlock, &0);
        vault.add_to_watchlist(&bob, &alice, &id);
    }

    // Verify watchlist is full
    assert_eq!(vault.get_watchlist(&bob).len(), 100);

    // Try to add one more — should fail
    let unlock = env.ledger().timestamp() + 3600 + 100;
    let id = vault.deposit(&alice, &token, &1_000, &unlock, &0);
    let result = vault.try_add_to_watchlist(&bob, &alice, &id);
    assert_eq!(result, Err(Ok(VaultError::WatchlistFull)));
}
```

---

### Multiple Depositors

```rust
#[test]
fn test_watch_multiple_depositors() {
    let (env, vault, token, _admin, _alice, _fee) = setup();
    
    let alice = Address::generate(&env);
    let charlie = Address::generate(&env);
    let bob = Address::generate(&env);

    StellarAssetClient::new(&env, &token).mint(&alice, &50_000);
    StellarAssetClient::new(&env, &token).mint(&charlie, &50_000);

    let unlock = env.ledger().timestamp() + 3600;
    
    // Alice and Charlie each deposit
    let alice_id = vault.deposit(&alice, &token, &1_000, &unlock, &0);
    let charlie_id = vault.deposit(&charlie, &token, &2_000, &unlock, &0);

    // Bob watches both
    vault.add_to_watchlist(&bob, &alice, &alice_id);
    vault.add_to_watchlist(&bob, &charlie, &charlie_id);

    // Verify both on watchlist
    let watchlist = vault.get_watchlist(&bob);
    assert_eq!(watchlist.len(), 2);
    
    assert_eq!(watchlist.get(0).unwrap().depositor, alice);
    assert_eq!(watchlist.get(1).unwrap().depositor, charlie);
}
```

---

## Integration Pattern: Off-Chain Indexer

### Listen to Watchlist Events

```javascript
// Pseudo-code for off-chain event listener
const contractAddress = "...";

// Subscribe to all watchlist events
soroban.events()
    .where(event => event.topic[0] === "watch_add" || event.topic[0] === "watch_rmv")
    .subscribe(event => {
        const subscriber = event.topic[1];
        const depositor = event.data[0];
        const depositId = event.data[1];
        const isAdd = event.topic[0] === "watch_add";
        
        if (isAdd) {
            database.watchlist.insert({
                subscriber, depositor, depositId
            });
        } else {
            database.watchlist.delete({
                subscriber, depositor, depositId
            });
        }
    });

// Query watchlist for a user
async function getUserWatchlist(subscriber) {
    return database.watchlist
        .where(row => row.subscriber === subscriber)
        .select(row => ({
            depositor: row.depositor,
            depositId: row.depositId
        }));
}
```

---

## Integration Pattern: Frontend

### React Component Example

```typescript
// Pseudo-code for React component
import { useCallback, useEffect, useState } from 'react';

function WatchlistManager({ vault, user }) {
    const [watchlist, setWatchlist] = useState([]);
    const [loading, setLoading] = useState(false);

    // Fetch watchlist on mount and after changes
    useEffect(() => {
        const loadWatchlist = async () => {
            setLoading(true);
            try {
                const items = await vault.get_watchlist(user);
                setWatchlist(items);
            } finally {
                setLoading(false);
            }
        };
        loadWatchlist();
    }, [vault, user]);

    const addToWatchlist = useCallback(
        async (depositor, depositId) => {
            setLoading(true);
            try {
                await vault.add_to_watchlist(user, depositor, depositId);
                await loadWatchlist(); // Refresh
            } catch (error) {
                if (error.includes("WatchlistFull")) {
                    alert("Watchlist is full (max 100 deposits)");
                } else if (error.includes("DepositAlreadyWatched")) {
                    alert("Already watching this deposit");
                } else {
                    alert("Error adding to watchlist: " + error);
                }
            } finally {
                setLoading(false);
            }
        },
        [vault, user]
    );

    const removeFromWatchlist = useCallback(
        async (depositor, depositId) => {
            setLoading(true);
            try {
                await vault.remove_from_watchlist(user, depositor, depositId);
                await loadWatchlist(); // Refresh
            } catch (error) {
                alert("Error removing from watchlist: " + error);
            } finally {
                setLoading(false);
            }
        },
        [vault, user]
    );

    return (
        <div>
            <h2>My Watchlist ({watchlist.length}/100)</h2>
            {watchlist.map((entry) => (
                <div key={`${entry.depositor}-${entry.deposit_id}`}>
                    <span>Deposit {entry.deposit_id} by {entry.depositor}</span>
                    <button
                        onClick={() =>
                            removeFromWatchlist(entry.depositor, entry.deposit_id)
                        }
                        disabled={loading}
                    >
                        Remove
                    </button>
                </div>
            ))}
        </div>
    );
}
```

---

## Constants

```rust
MAX_WATCHLIST_SIZE: u32 = 100  // Maximum deposits per user's watchlist
```

---

## Error Codes Reference

| Code | Name | Meaning |
|------|------|---------|
| 15 | `WatchlistFull` | Watchlist at MAX_WATCHLIST_SIZE (100) |
| 16 | `DepositAlreadyWatched` | Deposit already in watchlist |
| 17 | `DepositNotWatched` | Deposit not in watchlist |
