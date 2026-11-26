# MEV Protection User and Implementer Guide

This guide describes the MEV commit-reveal and price-deviation features exposed by the SAFE-HAVEN contract. It documents the current contract behavior; this feature is a detection and accounting mechanism, not a guarantee that MEV is prevented or that recovered tokens are paid out.

## Workflow

1. **Create a deposit.** MEV commits are associated with an existing deposit. Both timestamp-based and ledger-based deposits are supported.
2. **Prepare the commitment hash off-chain.** Use the exact encoding below. Keep the nonce private until reveal and use a fresh, unpredictable nonce for each commitment.
3. **Commit.** Call `mev_commit(depositor, deposit_id, commit_hash)` with the depositor's authorization. The 32-byte hash and commit timestamp are stored; the status becomes `Committed`.
4. **Reveal within 30 minutes.** Call `mev_reveal(depositor, deposit_id, token, amount, price, nonce)` with the same values used to calculate the hash. The contract checks the deadline and hash, records a price sample, and runs its deviation check. A successful call ends with status `Revealed`.
5. **Review detections and pending recovery.** Query status, detections, and pending recovery as needed.
6. **Claim.** Call `claim_mev_recovery(depositor)` with the depositor's authorization. It returns the pending `i128` amount and clears the pending accounting balance.

The reveal deadline is the commit timestamp plus **1,800 seconds (30 minutes)**. A reveal exactly at the deadline is accepted; one after it fails with `RevealWindowExpired`. This window uses ledger timestamp, independently of a deposit's unlock time or ledger sequence.

## Commitment hash format

The contract computes **Keccak-256** over this byte sequence, in order:

```text
Soroban-serialized token Address
|| amount as signed i128 little-endian (16 bytes)
|| price as signed i128 little-endian (16 bytes)
|| nonce as u32 little-endian (4 bytes)
```

The resulting digest is exactly 32 bytes (`BytesN<32>`). The token part is the bytes returned by Soroban's `env.serialize_to_bytes(&token)`; it is **not** the human-readable contract address string. Do not substitute SHA-256, packed decimal strings, big-endian integers, or a guessed fixed-width raw-address encoding.

For example, for `amount = 1_000`, `price = 102`, and `nonce = 42`, the numeric suffix of the preimage is:

```text
amount:  e8 03 00 00 00 00 00 00 00 00 00 00 00 00 00 00
price:   66 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
nonce:   2a 00 00 00
```

Prefix those bytes with the token's Soroban-serialized bytes, then Keccak-256 the complete preimage. This example specifies the input format; the digest depends on the token's serialized bytes.

### Rust/Soroban hash example

This mirrors the contract's encoding and is suitable when the caller has a Soroban `Env` (for example, in a Rust integration or test harness):

```rust
use soroban_sdk::{Address, Bytes, BytesN, Env};

fn mev_commit_hash(
    env: &Env,
    token: &Address,
    amount: i128,
    price: i128,
    nonce: u32,
) -> BytesN<32> {
    let mut preimage = Bytes::new(env);
    let token_bytes = env.serialize_to_bytes(token).expect("serialize token address");
    for byte in token_bytes.iter() {
        preimage.push_back(byte);
    }
    for byte in amount.to_le_bytes() {
        preimage.push_back(byte);
    }
    for byte in price.to_le_bytes() {
        preimage.push_back(byte);
    }
    for byte in nonce.to_le_bytes() {
        preimage.push_back(byte);
    }
    soroban_sdk::crypto::keccak256(&preimage)
}
```

The contract's Soroban address serialization must be matched by off-chain implementations too. Confirm the generated 32-byte digest against a contract test vector before using a different language or serialization library.

### Commit/reveal call shape

The hash-generation step is client-side; these calls show the contract arguments (use the generated client or the Soroban CLI for your deployment):

```text
mev_commit(depositor, deposit_id, commit_hash)
mev_reveal(depositor, deposit_id, token, amount, price, nonce)
```

The same `token`, `amount`, `price`, and `nonce` must be supplied at reveal. `mev_commit` requires an existing deposit and depositor authorization. `mev_reveal` also requires depositor authorization.

## Attack detection and the 200 bps threshold

On reveal, the contract records a price sample and compares the revealed price to the available price history for that token. The calculation uses a time-weighted average with recent samples weighted more heavily over a one-hour decay period. With no usable history, there is no deviation to flag.

The deviation is calculated as:

```text
abs(current_price - TWAP) * 10,000 / TWAP
```

The configured threshold is **200 basis points (2%)**. Detection occurs only when the calculated deviation is **greater than** 200 bps:

| Current price | TWAP | Deviation | Result |
|---:|---:|---:|---|
| 102 | 100 | 200 bps (2%) | No detection: threshold is strict `>` |
| 103 | 100 | 300 bps (3%) | Detection |
| 97 | 100 | 300 bps (3%) | Detection |

This is a price-deviation heuristic, not proof that a sandwich attack occurred. A detection records the deviation and adds `abs(current_price - TWAP)` to the internal MEV pool and the depositor's pending amount. That value is an `i128` price difference; it is not independently verified as token-denominated proceeds.

Although `MEVStatus` includes `AttackDetected`, the current reveal flow sets the status back to `Revealed` after the detection check returns. Therefore, a detected attack may not remain visible through `get_mev_status_query`; use detection records and pending recovery to check for detections.

Price history is populated by successful reveals, not by an external oracle supplied to these calls. Insufficient or unrepresentative samples can therefore cause missed detections or false positives. Do not describe the threshold as a guarantee of front-running prevention.

## Recovery claim process

1. Check `get_mev_pending(depositor)` for the pending amount.
2. Optionally inspect `get_mev_detections(depositor, deposit_id, offset, limit)`; `limit` is capped at 50.
3. Submit `claim_mev_recovery(depositor)` with depositor authorization.
4. The call returns the pending `i128`, subtracts it from the MEV pool, and resets the pending amount to zero. If there is no positive pending amount, it fails with `NoRewardsToClaim`.

**Important:** `claim_mev_recovery` does not transfer tokens. It only returns and clears the contract's recovery accounting. Do not present a successful claim as an on-chain token payout.

`finalize_mev_redistribution(admin)` is admin-authorized monitoring: it returns the current pool total (or zero when empty). It does not distribute funds.

## Common use cases

### Protecting a newly created deposit

```text
deposit(...) -> deposit_id
hash = keccak256(SorobanSerialize(token) || LE_i128(amount) || LE_i128(price) || LE_u32(nonce))
mev_commit(depositor, deposit_id, hash)
# Reveal no later than 30 minutes after the commit:
mev_reveal(depositor, deposit_id, token, amount, price, nonce)
get_mev_status_query(depositor, deposit_id)
```

Keep the nonce with the intended reveal values in the client until the reveal succeeds. Never use a different token, amount, price, or nonce at reveal.

### Checking and claiming recovery

```text
pending = get_mev_pending(depositor)
if pending > 0:
    amount = claim_mev_recovery(depositor)
    # amount is returned from the contract; no token transfer is performed.
```

### Recovering from a failed reveal

- `CommitNotFound`: verify the depositor and deposit ID, then submit a commitment.
- `CommitMismatch`: recompute using the exact token address serialization and little-endian values; compare all four reveal fields with the committed values.
- `RevealWindowExpired`: the commitment is past its 30-minute deadline. Submit a new commitment and reveal within its window.
- `InvalidPriceData`: the revealed price must be positive.
- `NoRewardsToClaim`: query pending recovery; there is no positive amount to claim.

## Configuration and queries

| Setting or function | Current behavior |
|---|---|
| `MEV_REVEAL_WINDOW_SECS` | `1_800` seconds (30 minutes) |
| `MEV_PRICE_DEVIATION_THRESHOLD_BPS` | `200` bps (2%); detection uses strict `>` |
| `get_mev_status_query(depositor, deposit_id)` | Returns an `MEVStatus`; successful reveals finish as `Revealed`, including when the detection check records an attack |
| `get_mev_pending(depositor)` | Pending recovery accounting for the depositor |
| `get_mev_pool_total()` | Current internal MEV pool total |
| `get_mev_detections(depositor, deposit_id, offset, limit)` | Detection records; maximum page size is 50 |

These constants are compiled into the contract; they are not per-user runtime settings. Changing them requires a code change and deployment of a contract with the new values.

## Best practices and limitations

- Generate a fresh, unpredictable nonce for every commitment; do not reveal it before committing.
- Hash exact values in contract units and verify the hash with a test vector before production integration.
- Reveal promptly rather than relying on the full deadline.
- Treat detection and recovery as monitoring/accounting signals, not guaranteed attack prevention or token reimbursement.
- Monitor detection records and pending amount as well as status; status is `Revealed` after a successful reveal even if an attack was recorded.
- Avoid claiming that commits are permanently immutable: a later `mev_commit` for the same depositor and deposit replaces the stored commitment.

For the complete contract signatures and current implementation, see [the API reference](./API_COMPREHENSIVE.md#mev-protection) and [the contract source](../contracts/safe-haven/src/contract.rs).
