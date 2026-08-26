/**
 * Tests for settings export/import functionality
 */

import {
  createExportData,
  exportSettingsAsJson,
  generateExportFilename,
  validateSettings,
  mergeSettings,
  parseSettingsFromJson,
  sanitizeSettings,
} from '../lib/settingsExport'
import { encryptData, decryptData } from '../lib/encryption'
import type { DepositTemplate, UserSettings } from '../types'

describe('Settings Export/Import', () => {
  // Sample test data
  const sampleTemplates: DepositTemplate[] = [
    {
      tokenAddress: 'CAQHBVHGKYPIGWPVGKSSWC5HG34HPCMX55AQBVGDSSLCCJV6VNCEWAGF',
      lockDurationSeconds: 30 * 24 * 60 * 60, // 30 days
      penaltyBps: 500, // 5%
      label: 'Template 1',
    },
    {
      tokenAddress: 'CBLTMQKPHH4RIWPGVVBJ5PZLX5GXFHSXQE5WOYXF5JLAJSAQBJHQ4J',
      lockDurationSeconds: 90 * 24 * 60 * 60, // 90 days
      penaltyBps: 1000, // 10%
    },
  ]

  const sampleTokens = [
    'CAQHBVHGKYPIGWPVGKSSWC5HG34HPCMX55AQBVGDSSLCCJV6VNCEWAGF',
    'CBLTMQKPHH4RIWPGVVBJ5PZLX5GXFHSXQE5WOYXF5JLAJSAQBJHQ4J',
  ]

  describe('createExportData', () => {
    it('should create export data with current timestamp', () => {
      const data = createExportData(sampleTemplates, sampleTokens)

      expect(data.version).toBe('1.0.0')
      expect(data.depositTemplates).toEqual(sampleTemplates)
      expect(data.frequentTokens).toEqual(sampleTokens)
      expect(typeof data.exportedAt).toBe('number')
      expect(data.exportedAt).toBeGreaterThan(0)
    })

    it('should limit templates to MAX_TEMPLATES', () => {
      const manyTemplates = Array(60).fill(sampleTemplates[0])
      const data = createExportData(manyTemplates, sampleTokens)

      expect(data.depositTemplates.length).toBe(50)
    })

    it('should limit frequent tokens to MAX_FREQUENT_TOKENS', () => {
      const manyTokens = Array(30).fill(sampleTokens[0])
      const data = createExportData(sampleTemplates, manyTokens)

      expect(data.frequentTokens.length).toBe(20)
    })
  })

  describe('exportSettingsAsJson', () => {
    it('should export settings as valid JSON string', () => {
      const json = exportSettingsAsJson(sampleTemplates, sampleTokens)

      expect(typeof json).toBe('string')
      const parsed = JSON.parse(json)
      expect(parsed.version).toBe('1.0.0')
      expect(Array.isArray(parsed.depositTemplates)).toBe(true)
      expect(Array.isArray(parsed.frequentTokens)).toBe(true)
    })
  })

  describe('generateExportFilename', () => {
    it('should generate unencrypted filename', () => {
      const filename = generateExportFilename(false)

      expect(filename).toContain('safe-haven-settings-')
      expect(filename).toContain('.json')
      expect(filename).not.toContain('enc')
    })

    it('should generate encrypted filename', () => {
      const filename = generateExportFilename(true)

      expect(filename).toContain('safe-haven-settings-')
      expect(filename).toContain('.enc.json')
    })
  })

  describe('validateSettings', () => {
    it('should validate correct settings', () => {
      const settings: UserSettings = {
        version: '1.0.0',
        exportedAt: Math.floor(Date.now() / 1000),
        depositTemplates: sampleTemplates,
        frequentTokens: sampleTokens,
      }

      const result = validateSettings(settings)

      expect(result.valid).toBe(true)
      expect(result.errors).toHaveLength(0)
    })

    it('should reject non-object input', () => {
      const result = validateSettings(null)

      expect(result.valid).toBe(false)
      expect(result.errors.length).toBeGreaterThan(0)
    })

    it('should reject missing depositTemplates', () => {
      const invalid = {
        version: '1.0.0',
        exportedAt: Math.floor(Date.now() / 1000),
        frequentTokens: sampleTokens,
      }

      const result = validateSettings(invalid)

      expect(result.valid).toBe(false)
      expect(result.errors.some((e) => e.includes('depositTemplates'))).toBe(true)
    })

    it('should reject invalid penalty basis points', () => {
      const settings = {
        version: '1.0.0',
        exportedAt: Math.floor(Date.now() / 1000),
        depositTemplates: [
          {
            tokenAddress: 'TEST',
            lockDurationSeconds: 100,
            penaltyBps: 15000, // > 10000
          },
        ],
        frequentTokens: [],
      }

      const result = validateSettings(settings)

      expect(result.valid).toBe(false)
      expect(result.errors.some((e) => e.includes('penaltyBps'))).toBe(true)
    })

    it('should warn on version mismatch', () => {
      const settings = {
        version: '2.0.0',
        exportedAt: Math.floor(Date.now() / 1000),
        depositTemplates: [],
        frequentTokens: [],
      }

      const result = validateSettings(settings)

      expect(result.valid).toBe(true)
      expect(result.warnings.some((w) => w.includes('version'))).toBe(true)
    })
  })

  describe('parseSettingsFromJson', () => {
    it('should parse valid JSON settings', () => {
      const settings: UserSettings = {
        version: '1.0.0',
        exportedAt: Math.floor(Date.now() / 1000),
        depositTemplates: sampleTemplates,
        frequentTokens: sampleTokens,
      }

      const json = JSON.stringify(settings)
      const parsed = parseSettingsFromJson(json)

      expect(parsed.depositTemplates).toEqual(sampleTemplates)
      expect(parsed.frequentTokens).toEqual(sampleTokens)
    })

    it('should throw on invalid JSON', () => {
      expect(() => parseSettingsFromJson('not valid json')).toThrow()
    })

    it('should throw on invalid settings structure', () => {
      const json = JSON.stringify({ invalid: 'structure' })

      expect(() => parseSettingsFromJson(json)).toThrow()
    })
  })

  describe('mergeSettings', () => {
    const existing = {
      templates: sampleTemplates,
      frequentTokens: sampleTokens,
    }

    const imported: UserSettings = {
      version: '1.0.0',
      exportedAt: Math.floor(Date.now() / 1000),
      depositTemplates: [
        {
          tokenAddress: 'CNEW',
          lockDurationSeconds: 7 * 24 * 60 * 60,
          penaltyBps: 200,
          label: 'New template',
        },
      ],
      frequentTokens: ['CNEW'],
    }

    it('should replace all settings in replace mode', () => {
      const result = mergeSettings(existing, imported, 'replace')

      expect(result.templates).toEqual(imported.depositTemplates)
      expect(result.frequentTokens).toEqual(imported.frequentTokens)
    })

    it('should merge settings in merge mode', () => {
      const result = mergeSettings(existing, imported, 'merge')

      expect(result.templates.length).toBe(3) // 2 existing + 1 new
      expect(result.frequentTokens.length).toBe(3) // 2 existing + 1 new
    })

    it('should deduplicate templates in merge mode', () => {
      const duplicateImported: UserSettings = {
        version: '1.0.0',
        exportedAt: Math.floor(Date.now() / 1000),
        depositTemplates: [sampleTemplates[0]], // Duplicate of existing
        frequentTokens: [],
      }

      const result = mergeSettings(existing, duplicateImported, 'merge')

      expect(result.templates.length).toBe(2) // No new template added
    })

    it('should deduplicate tokens in merge mode', () => {
      const duplicateImported: UserSettings = {
        version: '1.0.0',
        exportedAt: Math.floor(Date.now() / 1000),
        depositTemplates: [],
        frequentTokens: [sampleTokens[0]], // Duplicate of existing
      }

      const result = mergeSettings(existing, duplicateImported, 'merge')

      expect(result.frequentTokens.length).toBe(2) // No new token added
    })

    it('should respect capacity limits in merge mode', () => {
      const manyTemplates = Array(30).fill({
        tokenAddress: 'CTEST',
        lockDurationSeconds: 100,
        penaltyBps: 100,
      })

      const largeImported: UserSettings = {
        version: '1.0.0',
        exportedAt: Math.floor(Date.now() / 1000),
        depositTemplates: manyTemplates,
        frequentTokens: [],
      }

      const result = mergeSettings(existing, largeImported, 'merge')

      expect(result.templates.length).toBeLessThanOrEqual(50)
    })
  })

  describe('sanitizeSettings', () => {
    it('should sanitize settings with defaults', () => {
      const incomplete: Partial<UserSettings> = {
        depositTemplates: sampleTemplates,
      }

      const sanitized = sanitizeSettings(incomplete as UserSettings)

      expect(sanitized.version).toBe('1.0.0')
      expect(typeof sanitized.exportedAt).toBe('number')
      expect(sanitized.depositTemplates).toEqual(sampleTemplates)
    })

    it('should limit templates and tokens', () => {
      const excessive: UserSettings = {
        version: '1.0.0',
        exportedAt: Math.floor(Date.now() / 1000),
        depositTemplates: Array(100).fill(sampleTemplates[0]),
        frequentTokens: Array(50).fill(sampleTokens[0]),
      }

      const sanitized = sanitizeSettings(excessive)

      expect(sanitized.depositTemplates.length).toBe(50)
      expect(sanitized.frequentTokens.length).toBe(20)
    })
  })

  describe('Encryption (integration)', () => {
    it('should encrypt and decrypt data correctly', async () => {
      const testData = 'sensitive settings data'
      const password = 'test-password-123'

      const encrypted = await encryptData(testData, password)
      const decrypted = await decryptData(encrypted, password)

      expect(decrypted).toBe(testData)
    })

    it('should fail with wrong password', async () => {
      const testData = 'sensitive settings data'
      const password = 'correct-password'
      const wrongPassword = 'wrong-password'

      const encrypted = await encryptData(testData, password)

      await expect(decryptData(encrypted, wrongPassword)).rejects.toThrow()
    })

    it('should handle encrypted export with full workflow', async () => {
      const settings: UserSettings = {
        version: '1.0.0',
        exportedAt: Math.floor(Date.now() / 1000),
        depositTemplates: sampleTemplates,
        frequentTokens: sampleTokens,
      }

      const jsonData = JSON.stringify(settings, null, 2)
      const password = 'test-encryption-password'

      // Encrypt
      const encrypted = await encryptData(jsonData, password)
      expect(typeof encrypted).toBe('string')
      expect(encrypted).not.toBe(jsonData)

      // Decrypt
      const decrypted = await decryptData(encrypted, password)
      const parsed = JSON.parse(decrypted)

      expect(parsed).toEqual(settings)
    })
  })

  describe('Full workflow', () => {
    it('should export, validate, and re-import settings', () => {
      // Export
      const exported = exportSettingsAsJson(sampleTemplates, sampleTokens)

      // Parse and validate
      const parsed = parseSettingsFromJson(exported)
      const validation = validateSettings(parsed)

      expect(validation.valid).toBe(true)

      // Verify content
      expect(parsed.depositTemplates).toEqual(sampleTemplates)
      expect(parsed.frequentTokens).toEqual(sampleTokens)
    })

    it('should merge exported settings correctly', () => {
      const exported = exportSettingsAsJson(sampleTemplates, sampleTokens)
      const parsed = parseSettingsFromJson(exported)

      const existing = {
        templates: [
          {
            tokenAddress: 'CEXISTING',
            lockDurationSeconds: 60,
            penaltyBps: 100,
          },
        ],
        frequentTokens: ['CEXISTING'],
      }

      const result = mergeSettings(existing, parsed, 'merge')

      expect(result.templates.length).toBeGreaterThan(1)
      expect(result.frequentTokens.length).toBeGreaterThan(1)
    })
  })
})
