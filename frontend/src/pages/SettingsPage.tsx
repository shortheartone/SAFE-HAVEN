/**
 * Settings page for managing user settings and deposit templates
 */

import { useState, useRef } from 'react'
import toast from 'react-hot-toast'
import type {
  DepositTemplate,
  SettingsValidation,
  SettingsWithMergeInfo,
} from '../types'
import {
  exportSettingsAsJson,
  exportSettingsEncrypted,
  generateExportFilename,
  parseSettingsFromJson,
  parseSettingsFromEncrypted,
  validateSettings,
  mergeSettings,
} from '../lib/settingsExport'
import { isValidBase64 } from '../lib/encryption'
import { SettingsPreview } from '../components/SettingsPreview'

interface SettingsPageProps {}

// Demo data for testing
const DEFAULT_DEMO_TEMPLATES: DepositTemplate[] = [
  {
    tokenAddress: 'CAQHBVHGKYPIGWPVGKSSWC5HG34HPCMX55AQBVGDSSLCCJV6VNCEWAGF',
    lockDurationSeconds: 30 * 24 * 60 * 60, // 30 days
    penaltyBps: 500, // 5%
    label: 'Default: 30 days at 5% penalty',
  },
]

const DEFAULT_DEMO_TOKENS = [
  'CAQHBVHGKYPIGWPVGKSSWC5HG34HPCMX55AQBVGDSSLCCJV6VNCEWAGF',
]

export function SettingsPage({}: SettingsPageProps) {
  // Local state for templates
  const [templates, setTemplates] = useState<DepositTemplate[]>(DEFAULT_DEMO_TEMPLATES)
  const [frequentTokens, setFrequentTokens] = useState<string[]>(DEFAULT_DEMO_TOKENS)

  // Export state
  const [exportEncrypted, setExportEncrypted] = useState(false)
  const [exportPassword, setExportPassword] = useState('')
  const [showExportPassword, setShowExportPassword] = useState(false)

  // Import state
  const fileInputRef = useRef<HTMLInputElement>(null)
  const [importFile, setImportFile] = useState<File | null>(null)
  const [importPassword, setImportPassword] = useState('')
  const [showImportPassword, setShowImportPassword] = useState(false)
  const [importedSettings, setImportedSettings] = useState<SettingsWithMergeInfo | null>(null)
  const [importValidation, setImportValidation] = useState<SettingsValidation | null>(null)
  const [mergeAction, setMergeAction] = useState<'replace' | 'merge'>('merge')
  const [isProcessing, setIsProcessing] = useState(false)

  // ============================================================
  //  Export handlers
  // ============================================================

  async function handleExport() {
    try {
      let jsonContent = ''

      if (exportEncrypted) {
        if (!exportPassword) {
          toast.error('Enter a password for encryption')
          return
        }
        setIsProcessing(true)
        const encrypted = await exportSettingsEncrypted(templates, frequentTokens, exportPassword)
        jsonContent = encrypted
      } else {
        jsonContent = exportSettingsAsJson(templates, frequentTokens)
      }

      // Create blob and download
      const blob = new Blob([jsonContent], {
        type: 'application/octet-stream',
      })
      const url = URL.createObjectURL(blob)
      const link = document.createElement('a')
      link.href = url
      link.download = generateExportFilename(exportEncrypted)
      document.body.appendChild(link)
      link.click()
      document.body.removeChild(link)
      URL.revokeObjectURL(url)

      toast.success('Settings exported successfully')
      setExportPassword('')
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Export failed')
    } finally {
      setIsProcessing(false)
    }
  }

  // ============================================================
  //  Import handlers
  // ============================================================

  function handleFileSelect(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0]
    if (!file) return

    setImportFile(file)
    setImportedSettings(null)
    setImportValidation(null)
  }

  async function handleLoadFile() {
    if (!importFile) {
      toast.error('No file selected')
      return
    }

    try {
      setIsProcessing(true)
      const content = await importFile.text()

      let settings: SettingsWithMergeInfo

      // Try to detect if encrypted (base64 check)
      const isEncrypted = isValidBase64(content.trim())

      if (isEncrypted && importPassword) {
        settings = await parseSettingsFromEncrypted(content, importPassword)
      } else if (!isEncrypted) {
        settings = parseSettingsFromJson(content)
      } else {
        toast.error('File appears to be encrypted. Enter the password.')
        setIsProcessing(false)
        return
      }

      // Validate
      const validation = validateSettings(settings)
      setImportValidation(validation)

      if (!validation.valid) {
        toast.error(`Validation failed: ${validation.errors[0]}`)
        setIsProcessing(false)
        return
      }

      if (validation.warnings.length > 0) {
        validation.warnings.forEach((w) => toast.error(w))
      }

      setImportedSettings(settings)
      setImportPassword('')
      toast.success('Settings loaded successfully')
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Import failed')
    } finally {
      setIsProcessing(false)
    }
  }

  function handleApplySettings() {
    if (!importedSettings) return

    try {
      const merged = mergeSettings(
        { templates, frequentTokens },
        importedSettings,
        mergeAction
      )

      setTemplates(merged.templates)
      setFrequentTokens(merged.frequentTokens)

      // Reset import state
      setImportedSettings(null)
      setImportValidation(null)
      setImportFile(null)
      setImportPassword('')
      if (fileInputRef.current) {
        fileInputRef.current.value = ''
      }

      const action = mergeAction === 'replace' ? 'replaced' : 'merged'
      toast.success(`Settings ${action} successfully`)
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to apply settings')
    }
  }

  function handleCancelImport() {
    setImportedSettings(null)
    setImportValidation(null)
    setImportFile(null)
    setImportPassword('')
    if (fileInputRef.current) {
      fileInputRef.current.value = ''
    }
  }

  // ============================================================
  //  Render
  // ============================================================

  return (
    <div className="space-y-8 max-w-2xl">
      {/* Export section */}
      <section className="border border-slate-700/60 rounded-lg p-6 bg-slate-900/30">
        <h2 className="text-lg font-semibold text-slate-100 mb-4">Export Settings</h2>

        <div className="space-y-4">
          {/* Current state info */}
          <div className="bg-slate-800/50 border border-slate-700/40 rounded p-4">
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div>
                <p className="text-slate-500">Deposit Templates</p>
                <p className="text-slate-100 font-semibold">{templates.length}</p>
              </div>
              <div>
                <p className="text-slate-500">Frequent Tokens</p>
                <p className="text-slate-100 font-semibold">{frequentTokens.length}</p>
              </div>
            </div>
          </div>

          {/* Encryption toggle */}
          <div className="flex items-center gap-3">
            <input
              type="checkbox"
              id="export-encrypt"
              checked={exportEncrypted}
              onChange={(e) => {
                setExportEncrypted(e.target.checked)
                setExportPassword('')
              }}
              className="w-4 h-4 rounded border-slate-600 text-stellar-600 focus:ring-stellar-600"
            />
            <label htmlFor="export-encrypt" className="text-sm text-slate-300">
              Encrypt export with password
            </label>
          </div>

          {/* Password input (if encrypted) */}
          {exportEncrypted && (
            <div>
              <label className="block text-sm text-slate-300 mb-2">Encryption Password</label>
              <div className="relative">
                <input
                  type={showExportPassword ? 'text' : 'password'}
                  value={exportPassword}
                  onChange={(e) => setExportPassword(e.target.value)}
                  placeholder="Enter password for encryption"
                  className="w-full px-4 py-2 bg-slate-800 border border-slate-700 rounded text-slate-100 placeholder-slate-500 text-sm focus:outline-none focus:border-stellar-500"
                />
                <button
                  onClick={() => setShowExportPassword(!showExportPassword)}
                  className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-200 transition-colors"
                >
                  <svg className="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                    <path d="M10 12a2 2 0 100-4 2 2 0 000 4z" />
                    <path fillRule="evenodd" d="M.458 10C1.732 5.943 5.522 3 10 3s8.268 2.943 9.542 7c-1.274 4.057-5.064 7-9.542 7S1.732 14.057.458 10zM14 10a4 4 0 11-8 0 4 4 0 018 0z" clipRule="evenodd" />
                  </svg>
                </button>
              </div>
              <p className="text-xs text-slate-500 mt-1">
                You'll need this password to import encrypted settings
              </p>
            </div>
          )}

          {/* Export button */}
          <button
            onClick={handleExport}
            disabled={isProcessing}
            className="w-full px-4 py-2 bg-stellar-600 hover:bg-stellar-700 disabled:bg-slate-700 disabled:cursor-not-allowed text-white rounded-lg font-medium text-sm transition-colors flex items-center justify-center gap-2"
          >
            {isProcessing && <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />}
            {isProcessing ? 'Exporting...' : 'Export Settings'}
          </button>
        </div>
      </section>

      {/* Import section */}
      <section className="border border-slate-700/60 rounded-lg p-6 bg-slate-900/30">
        <h2 className="text-lg font-semibold text-slate-100 mb-4">Import Settings</h2>

        {!importedSettings ? (
          <div className="space-y-4">
            {/* File input */}
            <div>
              <label className="block text-sm text-slate-300 mb-2">Choose Settings File</label>
              <input
                ref={fileInputRef}
                type="file"
                onChange={handleFileSelect}
                accept=".json,.enc.json"
                className="w-full text-sm text-slate-400 file:mr-4 file:px-3 file:py-2 file:rounded file:border-0 file:bg-slate-700 file:text-slate-200 file:cursor-pointer hover:file:bg-slate-600"
              />
              {importFile && (
                <p className="text-xs text-slate-400 mt-2">
                  Selected: {importFile.name}
                </p>
              )}
            </div>

            {/* Password input (if encrypted) */}
            <div>
              <label className="block text-sm text-slate-300 mb-2">
                Password (if encrypted)
              </label>
              <div className="relative">
                <input
                  type={showImportPassword ? 'text' : 'password'}
                  value={importPassword}
                  onChange={(e) => setImportPassword(e.target.value)}
                  placeholder="Enter password if file is encrypted"
                  className="w-full px-4 py-2 bg-slate-800 border border-slate-700 rounded text-slate-100 placeholder-slate-500 text-sm focus:outline-none focus:border-stellar-500"
                />
                <button
                  onClick={() => setShowImportPassword(!showImportPassword)}
                  className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-200 transition-colors"
                >
                  <svg className="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                    <path d="M10 12a2 2 0 100-4 2 2 0 000 4z" />
                    <path fillRule="evenodd" d="M.458 10C1.732 5.943 5.522 3 10 3s8.268 2.943 9.542 7c-1.274 4.057-5.064 7-9.542 7S1.732 14.057.458 10zM14 10a4 4 0 11-8 0 4 4 0 018 0z" clipRule="evenodd" />
                  </svg>
                </button>
              </div>
            </div>

            {/* Load button */}
            <button
              onClick={handleLoadFile}
              disabled={!importFile || isProcessing}
              className="w-full px-4 py-2 bg-emerald-600 hover:bg-emerald-700 disabled:bg-slate-700 disabled:cursor-not-allowed text-white rounded-lg font-medium text-sm transition-colors flex items-center justify-center gap-2"
            >
              {isProcessing && <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />}
              {isProcessing ? 'Loading...' : 'Load Settings'}
            </button>
          </div>
        ) : (
          /* Preview and apply */
          <div className="space-y-4">
            {/* Merge action selection */}
            <div>
              <label className="block text-sm text-slate-300 mb-3 font-medium">
                How to apply settings?
              </label>
              <div className="flex gap-3">
                <label className="flex-1 flex items-center gap-2 p-3 border border-slate-700 rounded cursor-pointer hover:border-slate-600 hover:bg-slate-800/30" style={{
                  borderColor: mergeAction === 'replace' ? 'rgb(148, 163, 247)' : undefined,
                  backgroundColor: mergeAction === 'replace' ? 'rgba(148, 163, 247, 0.1)' : undefined
                }}>
                  <input
                    type="radio"
                    checked={mergeAction === 'replace'}
                    onChange={() => setMergeAction('replace')}
                    className="w-4 h-4"
                  />
                  <div>
                    <p className="text-sm font-medium text-slate-200">Replace All</p>
                    <p className="text-xs text-slate-400">Clear and replace with imported</p>
                  </div>
                </label>

                <label className="flex-1 flex items-center gap-2 p-3 border border-slate-700 rounded cursor-pointer hover:border-slate-600 hover:bg-slate-800/30" style={{
                  borderColor: mergeAction === 'merge' ? 'rgb(148, 163, 247)' : undefined,
                  backgroundColor: mergeAction === 'merge' ? 'rgba(148, 163, 247, 0.1)' : undefined
                }}>
                  <input
                    type="radio"
                    checked={mergeAction === 'merge'}
                    onChange={() => setMergeAction('merge')}
                    className="w-4 h-4"
                  />
                  <div>
                    <p className="text-sm font-medium text-slate-200">Merge</p>
                    <p className="text-xs text-slate-400">Combine with existing settings</p>
                  </div>
                </label>
              </div>
            </div>

            {/* Settings preview */}
            <SettingsPreview
              settings={importedSettings}
              mergeAction={mergeAction}
              currentTemplateCount={templates.length}
              currentTokenCount={frequentTokens.length}
            />

            {/* Validation messages */}
            {importValidation && importValidation.warnings.length > 0 && (
              <div className="bg-amber-900/20 border border-amber-600/40 rounded p-3 space-y-1">
                {importValidation.warnings.map((w, idx) => (
                  <p key={idx} className="text-xs text-amber-300">
                    ⚠ {w}
                  </p>
                ))}
              </div>
            )}

            {/* Action buttons */}
            <div className="flex gap-3">
              <button
                onClick={handleApplySettings}
                className="flex-1 px-4 py-2 bg-emerald-600 hover:bg-emerald-700 text-white rounded-lg font-medium text-sm transition-colors"
              >
                Apply Settings
              </button>
              <button
                onClick={handleCancelImport}
                className="flex-1 px-4 py-2 bg-slate-700 hover:bg-slate-600 text-slate-100 rounded-lg font-medium text-sm transition-colors"
              >
                Cancel
              </button>
            </div>
          </div>
        )}
      </section>

      {/* Info section */}
      <section className="border border-slate-700/60 rounded-lg p-6 bg-slate-900/30">
        <h3 className="text-sm font-semibold text-slate-100 mb-3">About Settings</h3>
        <ul className="space-y-2 text-sm text-slate-400">
          <li className="flex gap-2">
            <span className="text-slate-500">•</span>
            <span>Settings contain deposit templates and frequently used tokens</span>
          </li>
          <li className="flex gap-2">
            <span className="text-slate-500">•</span>
            <span>Encryption uses AES-256-GCM with password-based key derivation</span>
          </li>
          <li className="flex gap-2">
            <span className="text-slate-500">•</span>
            <span>Maximum 50 templates and 20 frequent tokens per export</span>
          </li>
          <li className="flex gap-2">
            <span className="text-slate-500">•</span>
            <span>Settings are stored locally in your browser, not in the cloud</span>
          </li>
        </ul>
      </section>
    </div>
  )
}
