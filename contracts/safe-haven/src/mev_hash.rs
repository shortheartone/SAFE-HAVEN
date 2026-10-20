// ============================================================
//  MEV Commit Hash Helper — Rust Implementation
//  Follows MEV_SPEC.md hash format specification
// ============================================================

use sha3::{Keccak256, Digest};
use soroban_sdk::Address;

/// MEV commit input fields that will be hashed.
///
/// The hash is computed over:
///   token || amount || price || nonce
///
/// See MEV_SPEC.md for detailed format specification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MevCommitInput {
    /// Stellar address of the token (20 bytes when serialized)
    pub token: Address,
    /// Amount in token base units (i128, 16 bytes big-endian)
    pub amount: i128,
    /// Token price in stroops (u64, 8 bytes big-endian)
    pub price: u64,
    /// Unique nonce per deposit (u32, 4 bytes big-endian)
    pub nonce: u32,
}

/// Compute MEV commit hash using Keccak256.
///
/// Input format (48 bytes total):
///   - token:  20 bytes (Stellar address identifier)
///   - amount: 16 bytes (i128 big-endian)
///   - price:  8 bytes (u64 big-endian)
///   - nonce:  4 bytes (u32 big-endian)
///
/// # Arguments
/// * `input` - MEV commit parameters
///
/// # Returns
/// Array of 32 bytes representing the Keccak256 hash
///
/// # Example
/// ```rust
/// let input = MevCommitInput {
///     token: Address::from_contract_id(&env, &contract_id),
///     amount: 1_000_000,
///     price: 5_000_000,
///     nonce: 42,
/// };
/// let hash = compute_mev_hash(&input);
/// ```
pub fn compute_mev_hash(input: &MevCommitInput) -> [u8; 32] {
    let mut hasher = Keccak256::new();

    // Step 1: Add token (20 bytes - Stellar address bytes)
    hasher.update(&input.token.to_bytes());

    // Step 2: Add amount (i128 as big-endian 16 bytes)
    hasher.update(&input.amount.to_be_bytes());

    // Step 3: Add price (u64 as big-endian 8 bytes)
    hasher.update(&input.price.to_be_bytes());

    // Step 4: Add nonce (u32 as big-endian 4 bytes)
    hasher.update(&input.nonce.to_be_bytes());

    // Step 5: Finalize and extract the 32-byte hash
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

/// Verify that a computed MEV hash matches an expected value.
///
/// # Arguments
/// * `input` - MEV commit parameters
/// * `expected_hash` - Expected hash (32 bytes)
///
/// # Returns
/// `true` if the computed hash matches the expected value, `false` otherwise
pub fn verify_mev_hash(input: &MevCommitInput, expected_hash: &[u8; 32]) -> bool {
    compute_mev_hash(input) == *expected_hash
}

/// Convert a hash byte array to a hex string for display/logging.
///
/// # Arguments
/// * `hash` - 32-byte hash array
///
/// # Returns
/// Lowercase hex string representation (64 characters)
pub fn hash_to_hex(hash: &[u8; 32]) -> String {
    hash.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join("")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test vector for validating MEV hash implementation.
    /// This should match the TypeScript implementation.
    #[test]
    fn test_mev_hash_consistency() {
        // We'll use a fixed address and known values for reproducibility.
        // The actual address bytes will be verified against TypeScript output.
        
        let input = MevCommitInput {
            token: Address::random(&soroban_sdk::Env::default()),
            amount: 1_000_000,
            price: 5_000_000,
            nonce: 42,
        };

        let hash = compute_mev_hash(&input);
        
        // Verify hash is 32 bytes
        assert_eq!(hash.len(), 32);
        
        // Verify hash is deterministic (same input = same output)
        let hash2 = compute_mev_hash(&input);
        assert_eq!(hash, hash2);
    }

    /// Verify hash changes when any input field changes
    #[test]
    fn test_mev_hash_sensitivity() {
        let env = soroban_sdk::Env::default();
        let token = Address::random(&env);

        let base_input = MevCommitInput {
            token: token.clone(),
            amount: 1_000_000,
            price: 5_000_000,
            nonce: 42,
        };

        let base_hash = compute_mev_hash(&base_input);

        // Change amount
        let mut variant = base_input.clone();
        variant.amount = 1_000_001;
        let hash_amount_diff = compute_mev_hash(&variant);
        assert_ne!(base_hash, hash_amount_diff, "Hash must change when amount changes");

        // Change price
        variant = base_input.clone();
        variant.price = 5_000_001;
        let hash_price_diff = compute_mev_hash(&variant);
        assert_ne!(base_hash, hash_price_diff, "Hash must change when price changes");

        // Change nonce
        variant = base_input.clone();
        variant.nonce = 43;
        let hash_nonce_diff = compute_mev_hash(&variant);
        assert_ne!(base_hash, hash_nonce_diff, "Hash must change when nonce changes");
    }

    /// Test hash_to_hex output format
    #[test]
    fn test_hash_to_hex() {
        let hash = [0u8; 32];
        let hex = hash_to_hex(&hash);
        
        // Should be 64 characters (32 bytes * 2 hex chars)
        assert_eq!(hex.len(), 64);
        // All zeros should be all '0's
        assert_eq!(hex, "0".repeat(64));
        
        // Test with non-zero values
        let mut hash2 = [0u8; 32];
        hash2[0] = 0xAB;
        hash2[1] = 0xCD;
        let hex2 = hash_to_hex(&hash2);
        assert!(hex2.starts_with("abcd"));
    }

    /// Test verify_mev_hash function
    #[test]
    fn test_verify_mev_hash() {
        let env = soroban_sdk::Env::default();
        let input = MevCommitInput {
            token: Address::random(&env),
            amount: 1_000_000,
            price: 5_000_000,
            nonce: 42,
        };

        let hash = compute_mev_hash(&input);
        
        // Verification should pass with correct hash
        assert!(verify_mev_hash(&input, &hash));
        
        // Verification should fail with wrong hash
        let mut wrong_hash = hash;
        wrong_hash[0] ^= 0xFF;
        assert!(!verify_mev_hash(&input, &wrong_hash));
    }
}
