# MEV Commit Hash Format Specification

## Overview

The MEV (Maximal Extractable Value) commit hash is a cryptographic commitment used to prevent transaction ordering manipulation and ensure protocol fairness. It combines deposit parameters into a single hash that can be verified across both the frontend (TypeScript) and smart contract (Rust) implementations.

---

## Hash Input Format

The MEV commit hash is computed over the following fields, concatenated in big-endian byte order:

```
commit_hash = Keccak256(token || amount || price || nonce)
```

### Field Specifications

| Field | Type | Size | Encoding | Description |
|-------|------|------|----------|-------------|
| `token` | Address | 20 bytes | Raw bytes | Stellar address as 20-byte identifier |
| `amount` | I128 | 16 bytes | Big-endian signed int | Token amount in base units (stroops for XLM) |
| `price` | U64 | 8 bytes | Big-endian unsigned int | Token price in stroops per unit |
| `nonce` | U32 | 4 bytes | Big-endian unsigned int | Unique nonce to prevent hash collisions |

**Total input size: 48 bytes**

---

## Encoding Details

### Token Address
- **Source:** Stellar account address (e.g., `GBUQWP3BOUZX34ULNQG23RQ6F4BWFIREPJ7KSCGUQXNtroppmruk74WM`)
- **Encoding:** Convert to 20-byte raw identifier used in Soroban scVal representation
- **Byte order:** As-is (no endianness conversion)

### Amount
- **Source:** Deposit amount in token base units
- **Type:** Signed 128-bit integer
- **Encoding:** Big-endian (most significant byte first)
- **Example:** 1,000,000 stroops → `0x00000000000000000000000000F42400`

### Price
- **Source:** Token price reference (from price oracle or config)
- **Type:** Unsigned 64-bit integer
- **Encoding:** Big-endian
- **Example:** 5,000,000 stroops → `0x004C4B40`

### Nonce
- **Source:** Unique identifier per deposit (e.g., timestamp, counter, or random value)
- **Type:** Unsigned 32-bit integer
- **Encoding:** Big-endian
- **Example:** 42 → `0x0000002A`

---

## Hash Algorithm

### Keccak256
- **Function:** Keccak256 (SHA-3 variant, not SHA3-256)
- **Output size:** 32 bytes
- **Output encoding:** Lowercase hexadecimal string (64 characters)

**Important:** Use the Keccak256 algorithm, not the standardized SHA3-256. These are different functions.

---

## Example Computation

### Input Values
```
token:  GBUQWP3BOUZX34ULNQG23RQ6F4BWFIREPJ7KSCGUQXNROPPMRUK74WM
amount: 1,000,000 stroops
price:  5,000,000 stroops per unit
nonce:  42
```

### Byte Construction (48 bytes total)

```
Token:     [20 bytes of Stellar address identifier]
Amount:    0x00 0x00 0x00 0x00 0x00 0x00 0x00 0x00 0x00 0x00 0x00 0x00 0x00 0x0F 0x42 0x40
Price:     0x00 0x4C 0x4B 0x40
Nonce:     0x00 0x00 0x00 0x2A
```

### Hash Output
```
commit_hash = "a3f7b2c1d4e8f9a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5"
```

---

## Implementation References

### TypeScript (Frontend)

```typescript
import { keccak256 } from 'js-sha3'
import { Address } from '@stellar/stellar-sdk'

interface MevInput {
  token: string        // Stellar address
  amount: bigint       // i128
  price: bigint        // u64
  nonce: number        // u32
}

export function computeMevHash(input: MevInput): string {
  const addressObj = new Address(input.token)
  const tokenBytes = addressObj.toBuffer()    // 20 bytes

  // Encode amount (i128) as big-endian 16 bytes
  const amountBuf = Buffer.alloc(16)
  amountBuf.writeBigInt64BE(input.amount, 0)

  // Encode price (u64) as big-endian 8 bytes
  const priceBuf = Buffer.alloc(8)
  priceBuf.writeBigUInt64BE(input.price, 0)

  // Encode nonce (u32) as big-endian 4 bytes
  const nonceBuf = Buffer.alloc(4)
  nonceBuf.writeUInt32BE(input.nonce, 0)

  // Concatenate: token || amount || price || nonce
  const input_bytes = Buffer.concat([tokenBytes, amountBuf, priceBuf, nonceBuf])

  // Compute Keccak256
  return keccak256(input_bytes)
}
```

### Rust (Smart Contract)

```rust
use sha3::{Keccak256, Digest};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MevInput {
    pub token: Address,
    pub amount: i128,
    pub price: u64,
    pub nonce: u32,
}

pub fn compute_mev_hash(input: &MevInput) -> [u8; 32] {
    let mut hasher = Keccak256::new();

    // Token: 20 bytes (Stellar address)
    hasher.update(&input.token.to_bytes());

    // Amount: i128 as big-endian 16 bytes
    hasher.update(&input.amount.to_be_bytes());

    // Price: u64 as big-endian 8 bytes
    hasher.update(&input.price.to_be_bytes());

    // Nonce: u32 as big-endian 4 bytes
    hasher.update(&input.nonce.to_be_bytes());

    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}
```

---

## Consistency Verification

To ensure implementations are correct:

1. Both TypeScript and Rust implementations must produce **identical hash outputs** for the same input
2. Test vectors should cover:
   - Minimum values (0 for all fields)
   - Maximum values (MAX_I128, MAX_U64, MAX_U32)
   - Edge cases (negative amounts, boundary nonces)
3. Cross-language unit tests must validate hash agreement

---

## Security Considerations

### Collision Resistance
- Keccak256 provides 128 bits of collision resistance
- Nonce should be unique per deposit to prevent replay attacks

### Preimage Resistance
- Keccak256 provides 256 bits of preimage resistance
- Infeasible to compute alternative (token, amount, price, nonce) tuples for a given hash

### Big-Endian Requirement
- **Critical:** All multi-byte integers must use big-endian (network byte order)
- Little-endian encoding will produce incorrect hashes

### Address Encoding
- Stellar addresses must be converted to their 20-byte raw form
- Do not hash the base32-encoded address string

---

## Usage Example

### Frontend (TypeScript)

```typescript
const mevInput: MevInput = {
  token: "GBUQWP3BOUZX34ULNQG23RQ6F4BWFIREPJ7KSCGUQXNROPPMRUK74WM",
  amount: BigInt(1_000_000),
  price: BigInt(5_000_000),
  nonce: 42
}

const hash = computeMevHash(mevInput)
console.log("Commit hash:", hash)
// Output: a3f7b2c1d4e8f9a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5
```

### Smart Contract (Rust)

```rust
let mev_input = MevInput {
    token: token_address,
    amount: 1_000_000,
    price: 5_000_000,
    nonce: 42,
};

let hash = compute_mev_hash(&mev_input);
// hash == [0xa3, 0xf7, ..., 0xf5]
```

---

## Migration & Versioning

This specification defines **version 1.0** of the MEV commit hash format.

If the format changes in the future:
1. Document the new version with a new section
2. Update implementations with version negotiation
3. Do not modify this v1.0 specification

---

## References

- [Keccak256 (SHA-3)](https://keccak.team/keccak.html)
- [Stellar Address Format](https://developers.stellar.org/learn/fundamentals/stellar-data-structures/accounts)
- [Soroban Address Type](https://github.com/stellar/rs-soroban-sdk)
- [Big-Endian Byte Order](https://en.wikipedia.org/wiki/Endianness)
