/**
 * Encryption utilities for secure settings export/import
 * Uses Web Crypto API for AES-GCM encryption with password-based key derivation
 */

const ALGORITHM = {
  name: 'AES-GCM',
  length: 256,
}

const PBKDF2 = {
  name: 'PBKDF2',
  hash: 'SHA-256',
  iterations: 100_000,
  length: 256,
}

const IV_LENGTH = 16 // 128 bits for GCM
const SALT_LENGTH = 16 // 128 bits

/**
 * Derives a cryptographic key from a password using PBKDF2
 */
async function deriveKey(password: string, salt: Uint8Array): Promise<CryptoKey> {
  const baseKey = await crypto.subtle.importKey(
    'raw',
    new TextEncoder().encode(password),
    PBKDF2,
    false,
    ['deriveBits']
  )

  const derivedBits = await crypto.subtle.deriveBits(
    {
      ...PBKDF2,
      salt,
    },
    baseKey,
    PBKDF2.length
  )

  return crypto.subtle.importKey(
    'raw',
    derivedBits,
    ALGORITHM,
    false,
    ['encrypt', 'decrypt']
  )
}

/**
 * Encrypts data with a password
 * Returns base64-encoded result containing salt, IV, and ciphertext
 */
export async function encryptData(data: string, password: string): Promise<string> {
  const salt = crypto.getRandomValues(new Uint8Array(SALT_LENGTH))
  const iv = crypto.getRandomValues(new Uint8Array(IV_LENGTH))

  const key = await deriveKey(password, salt)

  const encrypted = await crypto.subtle.encrypt(
    {
      ...ALGORITHM,
      iv,
    },
    key,
    new TextEncoder().encode(data)
  )

  // Combine salt + IV + ciphertext
  const combined = new Uint8Array(salt.length + iv.length + encrypted.byteLength)
  combined.set(salt, 0)
  combined.set(iv, salt.length)
  combined.set(new Uint8Array(encrypted), salt.length + iv.length)

  // Return as base64
  return btoa(String.fromCharCode(...combined))
}

/**
 * Decrypts data with a password
 * Expects base64-encoded input from encryptData
 */
export async function decryptData(encryptedBase64: string, password: string): Promise<string> {
  let combined: Uint8Array
  try {
    // Decode from base64
    combined = Uint8Array.from(atob(encryptedBase64), (c) => c.charCodeAt(0))
  } catch (e) {
    throw new Error('Invalid encrypted data format: decoding failed')
  }

  if (combined.length < SALT_LENGTH + IV_LENGTH) {
    throw new Error('Invalid encrypted data: insufficient length')
  }

  const salt = combined.slice(0, SALT_LENGTH)
  const iv = combined.slice(SALT_LENGTH, SALT_LENGTH + IV_LENGTH)
  const ciphertext = combined.slice(SALT_LENGTH + IV_LENGTH)

  const key = await deriveKey(password, salt)

  try {
    const decrypted = await crypto.subtle.decrypt(
      {
        ...ALGORITHM,
        iv,
      },
      key,
      ciphertext
    )

    return new TextDecoder().decode(decrypted)
  } catch (e) {
    throw new Error('Failed to decrypt: invalid password or corrupted data')
  }
}

/**
 * Check if a string is valid base64 (rough check)
 */
export function isValidBase64(str: string): boolean {
  try {
    return btoa(atob(str)) === str
  } catch {
    return false
  }
}
