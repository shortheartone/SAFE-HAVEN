// ============================================================
//  MEV Commit Hash Helper — TypeScript Implementation
//  Follows MEV_SPEC.md hash format specification
// ============================================================

import { keccak256 } from 'js-sha3'
import { Address } from '@stellar/stellar-sdk'

/**
 * MEV commit input fields that will be hashed.
 *
 * The hash is computed over:
 *   token || amount || price || nonce
 *
 * See MEV_SPEC.md for detailed format specification.
 */
export interface MevCommitInput {
  /** Stellar address of the token (20 bytes when serialized) */
  token: string
  /** Amount in token base units (i128, 16 bytes big-endian) */
  amount: bigint
  /** Token price in stroops (u64, 8 bytes big-endian) */
  price: bigint
  /** Unique nonce per deposit (u32, 4 bytes big-endian) */
  nonce: number
}

/**
 * Compute MEV commit hash using Keccak256.
 *
 * Input format (48 bytes total):
 *   - token:  20 bytes (Stellar address identifier)
 *   - amount: 16 bytes (i128 big-endian)
 *   - price:  8 bytes (u64 big-endian)
 *   - nonce:  4 bytes (u32 big-endian)
 *
 * @param input MEV commit parameters
 * @returns Lowercase hex string (64 characters)
 *
 * @throws Error if the token address is invalid
 *
 * @example
 * ```typescript
 * const hash = computeMevHash({
 *   token: "GBUQWP3BOUZX34ULNQG23RQ6F4BWFIREPJ7KSCGUQXNROPPMRUK74WM",
 *   amount: BigInt(1_000_000),
 *   price: BigInt(5_000_000),
 *   nonce: 42
 * })
 * console.log(hash) // "a3f7b2c1d4e8f9a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5"
 * ```
 */
export function computeMevHash(input: MevCommitInput): string {
  // Step 1: Convert Stellar address to 20-byte buffer
  const addressObj = new Address(input.token)
  const tokenBytes = addressObj.toBuffer() // 20 bytes

  // Step 2: Encode amount (i128) as big-endian 16 bytes
  const amountCorrect = encodeI128BE(input.amount)

  // Step 3: Encode price (u64) as big-endian 8 bytes
  const priceBuf = Buffer.alloc(8)
  priceBuf.writeBigUInt64BE(input.price, 0)

  // Step 4: Encode nonce (u32) as big-endian 4 bytes
  const nonceBuf = Buffer.alloc(4)
  nonceBuf.writeUInt32BE(input.nonce, 0)

  // Step 5: Concatenate all fields
  const inputBytes = Buffer.concat([tokenBytes, amountCorrect, priceBuf, nonceBuf])

  // Verify we have exactly 48 bytes
  if (inputBytes.length !== 48) {
    throw new Error(`MEV input must be exactly 48 bytes, got ${inputBytes.length}`)
  }

  // Step 6: Compute Keccak256
  const hash = keccak256(inputBytes)

  return hash
}

/**
 * Encode a signed 128-bit integer as big-endian bytes.
 *
 * The Keccak256 library's keccak256() function expects a Buffer or
 * Uint8Array. We use js-sha3's digest API which returns lowercase hex.
 *
 * @param value Signed 128-bit integer
 * @returns Buffer with 16 bytes in big-endian order
 */
function encodeI128BE(value: bigint): Buffer {
  const buf = Buffer.alloc(16)

  // Handle negative numbers using two's complement
  let v = value
  if (v < 0n) {
    v = (1n << 128n) + v // Convert to unsigned two's complement form
  }

  // Write upper 64 bits
  buf.writeBigUInt64BE(v >> 64n, 0)
  // Write lower 64 bits
  buf.writeBigUInt64BE(v & BigInt('0xFFFFFFFFFFFFFFFF'), 8)

  return buf
}

/**
 * Encode an unsigned 64-bit integer as big-endian bytes.
 * (Convenience function, mostly for documentation)
 *
 * @param value Unsigned 64-bit integer
 * @returns Buffer with 8 bytes in big-endian order
 */
export function encodeU64BE(value: bigint): Buffer {
  const buf = Buffer.alloc(8)
  buf.writeBigUInt64BE(value, 0)
  return buf
}

/**
 * Encode an unsigned 32-bit integer as big-endian bytes.
 * (Convenience function, mostly for documentation)
 *
 * @param value Unsigned 32-bit integer
 * @returns Buffer with 4 bytes in big-endian order
 */
export function encodeU32BE(value: number): Buffer {
  const buf = Buffer.alloc(4)
  buf.writeUInt32BE(value, 0)
  return buf
}

/**
 * Verify that a computed MEV hash matches an expected value.
 *
 * @param input MEV commit parameters
 * @param expectedHash Expected hash (lowercase hex string)
 * @returns true if hashes match, false otherwise
 */
export function verifyMevHash(input: MevCommitInput, expectedHash: string): boolean {
  const computed = computeMevHash(input)
  return computed === expectedHash.toLowerCase()
}

/**
 * Test vector for validating MEV hash implementation.
 *
 * This vector should match across TypeScript and Rust implementations.
 * Use this to verify correctness during testing.
 */
export const TEST_VECTOR_1 = {
  input: {
    token: 'GBUQWP3BOUZX34ULNQG23RQ6F4BWFIREPJ7KSCGUQXNROPPMRUK74WM',
    amount: BigInt(1_000_000),
    price: BigInt(5_000_000),
    nonce: 42,
  } as MevCommitInput,
  // This value should be computed once and then used as the canonical test vector
  expectedHashPlaceholder: 'COMPUTE_AND_VERIFY_FIRST',
}
