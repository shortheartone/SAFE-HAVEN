/**
 * Tests for encryption utilities
 */

import { encryptData, decryptData, isValidBase64 } from '../lib/encryption'

describe('Encryption', () => {
  describe('encryptData', () => {
    it('should encrypt data with a password', async () => {
      const data = 'sensitive information'
      const password = 'my-secret-password'

      const encrypted = await encryptData(data, password)

      expect(typeof encrypted).toBe('string')
      expect(encrypted).not.toBe(data)
      expect(encrypted.length).toBeGreaterThan(0)
    })

    it('should produce different ciphertexts for the same input (due to random salt/IV)', async () => {
      const data = 'same data'
      const password = 'same password'

      const encrypted1 = await encryptData(data, password)
      const encrypted2 = await encryptData(data, password)

      expect(encrypted1).not.toBe(encrypted2)
    })

    it('should handle empty strings', async () => {
      const encrypted = await encryptData('', 'password')

      expect(typeof encrypted).toBe('string')
    })

    it('should handle long data', async () => {
      const data = 'x'.repeat(100000)
      const password = 'password'

      const encrypted = await encryptData(data, password)

      expect(typeof encrypted).toBe('string')
    })

    it('should handle special characters', async () => {
      const data = 'data with special chars: éàü 你好 🔐'
      const password = 'пароль 密碼'

      const encrypted = await encryptData(data, password)

      expect(typeof encrypted).toBe('string')
    })
  })

  describe('decryptData', () => {
    it('should decrypt data with correct password', async () => {
      const originalData = 'secret message'
      const password = 'correct-password'

      const encrypted = await encryptData(originalData, password)
      const decrypted = await decryptData(encrypted, password)

      expect(decrypted).toBe(originalData)
    })

    it('should fail with wrong password', async () => {
      const data = 'secret'
      const password = 'correct'
      const wrongPassword = 'wrong'

      const encrypted = await encryptData(data, password)

      await expect(decryptData(encrypted, wrongPassword)).rejects.toThrow(
        /Failed to decrypt|invalid password/i
      )
    })

    it('should fail with invalid base64', async () => {
      const invalidData = 'not-valid-base64!!!'

      await expect(decryptData(invalidData, 'password')).rejects.toThrow()
    })

    it('should fail with corrupted data', async () => {
      const data = 'original'
      const password = 'password'

      const encrypted = await encryptData(data, password)
      const corrupted = btoa(atob(encrypted).slice(0, -5)) // Remove last 5 bytes

      await expect(decryptData(corrupted, password)).rejects.toThrow()
    })

    it('should handle empty encrypted message', async () => {
      const encrypted = await encryptData('', 'password')
      const decrypted = await decryptData(encrypted, 'password')

      expect(decrypted).toBe('')
    })
  })

  describe('isValidBase64', () => {
    it('should identify valid base64', () => {
      expect(isValidBase64('SGVsbG8gV29ybGQ=')).toBe(true)
      expect(isValidBase64('YWJjMTIz')).toBe(true)
      expect(isValidBase64(btoa('test data'))).toBe(true)
    })

    it('should reject invalid base64', () => {
      expect(isValidBase64('not valid base64!!!')).toBe(false)
      expect(isValidBase64('!!!invalid!!!')).toBe(false)
      expect(isValidBase64('')).toBe(true) // Empty string is valid base64
    })
  })

  describe('Round-trip encryption', () => {
    const testCases = [
      { data: 'simple text', password: 'password' },
      { data: '', password: 'password' },
      { data: 'text with\nnewlines\nand\ttabs', password: 'password' },
      { data: JSON.stringify({ key: 'value', number: 42 }), password: 'password' },
      { data: 'a'.repeat(10000), password: 'very-long-password-' + 'x'.repeat(100) },
      { data: 'emoji: 🔐🔑🛡️', password: '密码🔒' },
    ]

    testCases.forEach(({ data, password }, idx) => {
      it(`should round-trip test case ${idx}`, async () => {
        const encrypted = await encryptData(data, password)
        const decrypted = await decryptData(encrypted, password)

        expect(decrypted).toBe(data)
      })
    })
  })

  describe('Security properties', () => {
    it('should use different salts for each encryption', async () => {
      const data = 'test'
      const password = 'password'

      const encrypted1 = await encryptData(data, password)
      const encrypted2 = await encryptData(data, password)

      // Extract salt (first 16 bytes) from base64
      const salt1 = atob(encrypted1).slice(0, 16)
      const salt2 = atob(encrypted2).slice(0, 16)

      expect(salt1).not.toBe(salt2)
    })

    it('should use different IVs for each encryption', async () => {
      const data = 'test'
      const password = 'password'

      const encrypted1 = await encryptData(data, password)
      const encrypted2 = await encryptData(data, password)

      // Extract IV (bytes 16-32) from base64
      const iv1 = atob(encrypted1).slice(16, 32)
      const iv2 = atob(encrypted2).slice(16, 32)

      expect(iv1).not.toBe(iv2)
    })
  })
})
