# MEV Protection — Quick Start

1. Create a deposit and note its `deposit_id`.
2. Calculate the 32-byte Keccak-256 commitment using the exact Soroban address serialization and little-endian integer format in the [MEV user and implementer guide](./docs/MEV_PROTECTION_GUIDE.md#commitment-hash-format).
3. Call `mev_commit(depositor, deposit_id, commit_hash)`.
4. Within 30 minutes, call `mev_reveal(depositor, deposit_id, token, amount, price, nonce)` with the committed values.
5. Query status and pending recovery; call `claim_mev_recovery(depositor)` if recovery is pending.

Detection uses a strict threshold of **more than 200 bps (2%)** from the available time-weighted price history. A claim returns and clears the pending accounting amount; **it does not transfer tokens**. This feature is not a guarantee against MEV.

For the exact byte encoding, sample implementation, API details, errors, and best practices, see the [MEV Protection User and Implementer Guide](./docs/MEV_PROTECTION_GUIDE.md).
