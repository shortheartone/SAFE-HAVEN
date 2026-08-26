/**
 * Settings preview component for import preview
 */

import type { UserSettings } from '../types'
import { formatBps, formatDuration } from '../lib/format'

interface SettingsPreviewProps {
  settings: UserSettings
  mergeAction: 'replace' | 'merge'
  currentTemplateCount: number
  currentTokenCount: number
}

export function SettingsPreview({
  settings,
  mergeAction,
  currentTemplateCount,
  currentTokenCount,
}: SettingsPreviewProps) {
  const exportDate = new Date(settings.exportedAt * 1000).toLocaleString()

  const templateCount = mergeAction === 'replace' ? 0 : currentTemplateCount
  const tokenCount = mergeAction === 'replace' ? 0 : currentTokenCount

  const newTemplateCount =
    templateCount + settings.depositTemplates.length >
    50
      ? 50 - templateCount
      : settings.depositTemplates.length

  const newTokenCount =
    tokenCount + settings.frequentTokens.length > 20
      ? 20 - tokenCount
      : settings.frequentTokens.length

  return (
    <div className="border border-slate-700/60 rounded-lg p-6 space-y-6 bg-slate-900/30">
      {/* Header info */}
      <div>
        <h3 className="text-sm font-medium text-slate-300 mb-4">Export Information</h3>
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 text-sm">
          <div>
            <p className="text-slate-500">Version</p>
            <p className="text-slate-100 font-mono">{settings.version}</p>
          </div>
          <div>
            <p className="text-slate-500">Exported</p>
            <p className="text-slate-100">{exportDate}</p>
          </div>
        </div>
      </div>

      {/* Merge mode info */}
      <div className="bg-slate-800/50 border border-slate-700/40 rounded p-4">
        <p className="text-sm text-slate-300 mb-2">
          <span className="font-medium">Mode:</span>{' '}
          <span
            className={
              mergeAction === 'replace'
                ? 'text-amber-400'
                : 'text-emerald-400'
            }
          >
            {mergeAction === 'replace'
              ? 'Replace all settings'
              : 'Merge with existing'}
          </span>
        </p>
        {mergeAction === 'merge' && (
          <p className="text-xs text-slate-400">
            Imported settings will be combined with your existing settings.
            Duplicates will be removed.
          </p>
        )}
        {mergeAction === 'replace' && (
          <p className="text-xs text-slate-400">
            All current settings will be replaced.
          </p>
        )}
      </div>

      {/* Deposit templates preview */}
      <div>
        <div className="flex items-center justify-between mb-4">
          <h4 className="text-sm font-medium text-slate-300">Deposit Templates</h4>
          <span className="inline-flex items-center gap-2">
            <span className="text-xs px-2 py-1 bg-slate-800 rounded text-slate-300">
              +{newTemplateCount}
            </span>
            {mergeAction === 'merge' && (
              <span className="text-xs text-slate-500">
                ({currentTemplateCount} existing)
              </span>
            )}
          </span>
        </div>

        {settings.depositTemplates.length === 0 ? (
          <p className="text-xs text-slate-500 italic">No templates to import</p>
        ) : (
          <div className="space-y-2 max-h-64 overflow-y-auto">
            {settings.depositTemplates.slice(0, 5).map((template, idx) => (
              <div
                key={idx}
                className="bg-slate-800/30 border border-slate-700/40 rounded p-3 text-xs"
              >
                <div className="flex items-start justify-between gap-2">
                  <div className="flex-1 space-y-1">
                    {template.label && (
                      <p className="font-medium text-slate-200">{template.label}</p>
                    )}
                    <p className="text-slate-400 font-mono truncate">
                      {template.tokenAddress}
                    </p>
                  </div>
                </div>
                <div className="grid grid-cols-2 gap-2 mt-2">
                  <div>
                    <p className="text-slate-500">Lock:</p>
                    <p className="text-slate-300">
                      {formatDuration(template.lockDurationSeconds)}
                    </p>
                  </div>
                  <div>
                    <p className="text-slate-500">Penalty:</p>
                    <p className="text-slate-300">
                      {formatBps(template.penaltyBps)}
                    </p>
                  </div>
                </div>
              </div>
            ))}
            {settings.depositTemplates.length > 5 && (
              <p className="text-xs text-slate-500 italic pt-2">
                +{settings.depositTemplates.length - 5} more templates
              </p>
            )}
          </div>
        )}
      </div>

      {/* Frequent tokens preview */}
      <div>
        <div className="flex items-center justify-between mb-4">
          <h4 className="text-sm font-medium text-slate-300">Frequent Tokens</h4>
          <span className="inline-flex items-center gap-2">
            <span className="text-xs px-2 py-1 bg-slate-800 rounded text-slate-300">
              +{newTokenCount}
            </span>
            {mergeAction === 'merge' && (
              <span className="text-xs text-slate-500">
                ({currentTokenCount} existing)
              </span>
            )}
          </span>
        </div>

        {settings.frequentTokens.length === 0 ? (
          <p className="text-xs text-slate-500 italic">No tokens to import</p>
        ) : (
          <div className="space-y-1 max-h-40 overflow-y-auto">
            {settings.frequentTokens.slice(0, 8).map((token, idx) => (
              <div key={idx} className="text-xs font-mono text-slate-300 truncate">
                {token}
              </div>
            ))}
            {settings.frequentTokens.length > 8 && (
              <p className="text-xs text-slate-500 italic pt-2">
                +{settings.frequentTokens.length - 8} more tokens
              </p>
            )}
          </div>
        )}
      </div>

      {/* Capacity warnings */}
      {mergeAction === 'merge' && newTemplateCount < settings.depositTemplates.length && (
        <div className="bg-amber-900/20 border border-amber-600/40 rounded p-3 text-xs text-amber-300">
          Only {newTemplateCount} of {settings.depositTemplates.length} templates can be imported
          (capacity: 50 total). Remaining will be skipped.
        </div>
      )}
      {mergeAction === 'merge' && newTokenCount < settings.frequentTokens.length && (
        <div className="bg-amber-900/20 border border-amber-600/40 rounded p-3 text-xs text-amber-300">
          Only {newTokenCount} of {settings.frequentTokens.length} tokens can be imported (capacity:
          20 total). Remaining will be skipped.
        </div>
      )}
    </div>
  )
}
