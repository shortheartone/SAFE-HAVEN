/**
 * Settings export/import utilities
 * Handles serialization, validation, and merging of user settings
 */

import type {
  UserSettings,
  DepositTemplate,
  SettingsValidation,
  SettingsWithMergeInfo,
} from '../types'
import { encryptData, decryptData } from './encryption'

const CURRENT_VERSION = '1.0.0'
const MAX_TEMPLATES = 50
const MAX_FREQUENT_TOKENS = 20

/**
 * Creates export data from current settings
 */
export function createExportData(
  templates: DepositTemplate[],
  frequentTokens: string[]
): UserSettings {
  return {
    version: CURRENT_VERSION,
    exportedAt: Math.floor(Date.now() / 1000),
    depositTemplates: templates.slice(0, MAX_TEMPLATES),
    frequentTokens: frequentTokens.slice(0, MAX_FREQUENT_TOKENS),
  }
}

/**
 * Exports settings as JSON string
 */
export function exportSettingsAsJson(
  templates: DepositTemplate[],
  frequentTokens: string[]
): string {
  const data = createExportData(templates, frequentTokens)
  return JSON.stringify(data, null, 2)
}

/**
 * Exports settings as encrypted JSON (password-protected)
 */
export async function exportSettingsEncrypted(
  templates: DepositTemplate[],
  frequentTokens: string[],
  password: string
): Promise<string> {
  const jsonData = exportSettingsAsJson(templates, frequentTokens)
  return encryptData(jsonData, password)
}

/**
 * Generates a filename for export
 */
export function generateExportFilename(encrypted: boolean = false): string {
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-').split('T')[0]
  const extension = encrypted ? 'enc.json' : 'json'
  return `safe-haven-settings-${timestamp}.${extension}`
}

/**
 * Validates imported settings data
 */
export function validateSettings(data: unknown): SettingsValidation {
  const errors: string[] = []
  const warnings: string[] = []

  // Basic structure check
  if (!data || typeof data !== 'object') {
    return {
      valid: false,
      errors: ['Settings must be a valid JSON object'],
      warnings: [],
    }
  }

  const settings = data as Record<string, unknown>

  // Version check
  if (!settings.version || typeof settings.version !== 'string') {
    errors.push('Missing or invalid version field')
  } else if (settings.version !== CURRENT_VERSION) {
    warnings.push(
      `Settings version ${settings.version} may not be fully compatible with v${CURRENT_VERSION}`
    )
  }

  // Timestamp check
  if (!settings.exportedAt || typeof settings.exportedAt !== 'number') {
    warnings.push('Missing export timestamp')
  }

  // Templates validation
  if (!Array.isArray(settings.depositTemplates)) {
    errors.push('depositTemplates must be an array')
  } else {
    if (settings.depositTemplates.length === 0) {
      warnings.push('No deposit templates to import')
    }
    settings.depositTemplates.forEach((template, idx) => {
      const t = template as Record<string, unknown>
      if (typeof t.tokenAddress !== 'string') {
        errors.push(`Template ${idx}: tokenAddress must be a string`)
      }
      if (typeof t.lockDurationSeconds !== 'number' || t.lockDurationSeconds < 0) {
        errors.push(`Template ${idx}: lockDurationSeconds must be a positive number`)
      }
      if (typeof t.penaltyBps !== 'number' || t.penaltyBps < 0 || t.penaltyBps > 10000) {
        errors.push(`Template ${idx}: penaltyBps must be 0-10000`)
      }
    })
  }

  // Frequent tokens validation
  if (!Array.isArray(settings.frequentTokens)) {
    errors.push('frequentTokens must be an array')
  } else {
    if (settings.frequentTokens.length === 0) {
      warnings.push('No frequent tokens to import')
    }
    settings.frequentTokens.forEach((token, idx) => {
      if (typeof token !== 'string') {
        errors.push(`frequentTokens[${idx}] must be a string`)
      }
    })
  }

  return {
    valid: errors.length === 0,
    errors,
    warnings,
  }
}

/**
 * Parses imported settings from JSON string (unencrypted)
 */
export function parseSettingsFromJson(jsonString: string): SettingsWithMergeInfo {
  let data: unknown
  try {
    data = JSON.parse(jsonString)
  } catch (e) {
    throw new Error(`Invalid JSON: ${e instanceof Error ? e.message : 'Unknown error'}`)
  }

  const validation = validateSettings(data)
  if (!validation.valid) {
    throw new Error(`Invalid settings: ${validation.errors.join('; ')}`)
  }

  return data as SettingsWithMergeInfo
}

/**
 * Parses imported settings from encrypted data (password-protected)
 */
export async function parseSettingsFromEncrypted(
  encryptedData: string,
  password: string
): Promise<SettingsWithMergeInfo> {
  const decrypted = await decryptData(encryptedData, password)
  return parseSettingsFromJson(decrypted)
}

/**
 * Merges imported settings with existing settings
 * mode='replace': replaces everything
 * mode='merge': appends new templates and tokens (dedupes)
 */
export function mergeSettings(
  existing: { templates: DepositTemplate[]; frequentTokens: string[] },
  imported: UserSettings,
  mode: 'replace' | 'merge'
): { templates: DepositTemplate[]; frequentTokens: string[] } {
  if (mode === 'replace') {
    return {
      templates: [...imported.depositTemplates],
      frequentTokens: [...imported.frequentTokens],
    }
  }

  // Merge mode: deduplicate and combine
  const existingTemplateKeys = new Set(
    existing.templates.map((t) => `${t.tokenAddress}:${t.lockDurationSeconds}:${t.penaltyBps}`)
  )
  const newTemplates = [
    ...existing.templates,
    ...imported.depositTemplates.filter(
      (t) => !existingTemplateKeys.has(`${t.tokenAddress}:${t.lockDurationSeconds}:${t.penaltyBps}`)
    ),
  ].slice(0, MAX_TEMPLATES)

  const existingTokenSet = new Set(existing.frequentTokens)
  const newTokens = [
    ...existing.frequentTokens,
    ...imported.frequentTokens.filter((t) => !existingTokenSet.has(t)),
  ].slice(0, MAX_FREQUENT_TOKENS)

  return {
    templates: newTemplates,
    frequentTokens: newTokens,
  }
}

/**
 * Sanitizes settings to ensure they're safe
 */
export function sanitizeSettings(settings: UserSettings): UserSettings {
  return {
    version: settings.version || CURRENT_VERSION,
    exportedAt: settings.exportedAt || Math.floor(Date.now() / 1000),
    depositTemplates: (settings.depositTemplates || []).slice(0, MAX_TEMPLATES),
    frequentTokens: (settings.frequentTokens || []).slice(0, MAX_FREQUENT_TOKENS),
  }
}
