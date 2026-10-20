// ============================================================
//  MEV Hash TypeScript Implementation Tests
// ============================================================

import { computeMevHash, verifyMevHash, MevCommitInput } from './mev-hash'
import { describe, it, expect } from 'vitest'

describe('MEV Hash - TypeScript Implementation', () => {
  it('should compute hash with valid input', () => {
    const input: MevCommitInput = {
      token: 'GBUQWP3BOUZX34ULNQG23RQ6F4BWFIREPJ7KSCGUQXNROPPMRUK74WM',
      amount: BigInt(1_000_000),
      price: BigInt(5_000_000),
      nonce: 42,
    }

    const hash = computeMevHash(input)

    // Should return a 64-character hex string (32 bytes)
    expect(hash).toMatch(/^[0-9a-f]{64}$/)
  })

  it('should be deterministic', () => {
    const input: MevCommitInput = {
      token: 'GBUQWP3BOUZX34ULNQG23RQ6F4BWFIREPJ7KSCGUQXNROPPMRUK74WM',
      amount: BigInt(1_000_000),
      price: BigInt(5_000_000),
      nonce: 42,
    }

    const hash1 = computeMevHash(input)
    const hash2 = computeMevHash(input)

    expect(hash1).toBe(hash2)
  })

  it('should change hash when amount changes', () => {
    const input: MevCommitInput = {
      token: 'GBUQWP3BOUZX34ULNQG23RQ6F4BWFIREPJ7KSCGUQXNROPPMRUK74WM',
      amount: BigInt(1_000_000),
      price: BigInt(5_000_000),
      nonce: 42,
    }

    const hash1 = computeMevHash(input)

    const input2: MevCommitInput = {
      ...input,
      amount: BigInt(1_000_001),
    }

    const hash2 = computeMevHash(input2)

    expect(hash1).not.toBe(hash2)
  })

  it('should change hash when price changes', () => {
    const input: MevCommitInput = {
      token: 'GBUQWP3BOUZX34ULNQG23RQ6F4BWFIREPJ7KSCGUQXNROPPMRUK74WM',
      amount: BigInt(1_000_000),
      price: BigInt(5_000_000),
      nonce: 42,
    }

    const hash1 = computeMevHash(input)

    const input2: MevCommitInput = {
      ...input,
      price: BigInt(5_000_001),
    }

    const hash2 = computeMevHash(input2)

    expect(hash1).not.toBe(hash2)
  })

  it('should change hash when nonce changes', () => {
    const input: MevCommitInput = {
      token: 'GBUQWP3BOUZX34ULNQG23RQ6F4BWFIREPJ7KSCGUQXNROPPMRUK74WM',
      amount: BigInt(1_000_000),
      price: BigInt(5_000_000),
      nonce: 42,
    }

    const hash1 = computeMevHash(input)

    const input2: MevCommitInput = {
      ...input,
      nonce: 43,
    }

    const hash2 = computeMevHash(input2)

    expect(hash1).not.toBe(hash2)
  })

  it('should verify correct hash', () => {
    const input: MevCommitInput = {
      token: 'GBUQWP3BOUZX34ULNQG23RQ6F4BWFIREPJ7KSCGUQXNROPPMRUK74WM',
      amount: BigInt(1_000_000),
      price: BigInt(5_000_000),
      nonce: 42,
    }

    const hash = computeMevHash(input)
    const isValid = verifyMevHash(input, hash)

    expect(isValid).toBe(true)
  })

  it('should reject incorrect hash', () => {
    const input: MevCommitInput = {
      token: 'GBUQWP3BOUZX34ULNQG23RQ6F4BWFIREPJ7KSCGUQXNROPPMRUK74WM',
      amount: BigInt(1_000_000),
      price: BigInt(5_000_000),
      nonce: 42,
    }

    const wrongHash = 'a'.repeat(64)
    const isValid = verifyMevHash(input, wrongHash)

    expect(isValid).toBe(false)
  })

  it('should handle edge case: zero nonce', () => {
    const input: MevCommitInput = {
      token: 'GBUQWP3BOUZX34ULNQG23RQ6F4BWFIREPJ7KSCGUQXNROPPMRUK74WM',
      amount: BigInt(0),
      price: BigInt(0),
      nonce: 0,
    }

    const hash = computeMevHash(input)
    expect(hash).toMatch(/^[0-9a-f]{64}$/)
  })

  it('should handle edge case: large amounts', () => {
    const input: MevCommitInput = {
      token: 'GBUQWP3BOUZX34ULNQG23RQ6F4BWFIREPJ7KSCGUQXNROPPMRUK74WM',
      amount: BigInt('9223372036854775807'), // i64 max
      price: BigInt('18446744073709551615'), // u64 max
      nonce: 4294967295, // u32 max
    }

    const hash = computeMevHash(input)
    expect(hash).toMatch(/^[0-9a-f]{64}$/)
  })

  it('should handle different Stellar addresses', () => {
    const input1: MevCommitInput = {
      token: 'GBUQWP3BOUZX34ULNQG23RQ6F4BWFIREPJ7KSCGUQXNROPPMRUK74WM',
      amount: BigInt(1_000_000),
      price: BigInt(5_000_000),
      nonce: 42,
    }

    const input2: MevCommitInput = {
      token: 'GBBD47UZQ5SYWVVCDROIGMC5QCWGZIOFWXILGTBtardvcsfd3FJJSZOXM',
      amount: BigInt(1_000_000),
      price: BigInt(5_000_000),
      nonce: 42,
    }

    const hash1 = computeMevHash(input1)
    const hash2 = computeMevHash(input2)

    expect(hash1).not.toBe(hash2)
  })
})
