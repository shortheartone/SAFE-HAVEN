/**
 * Insurance Terms Modal - displays terms and conditions
 */

import type { InsuranceTerms } from '../types'
import { baseUnitsToAmount, formatDuration } from '../lib/format'

interface InsuranceTermsModalProps {
  terms: InsuranceTerms
  onClose: () => void
}

export function InsuranceTermsModal({ terms, onClose }: InsuranceTermsModalProps) {
  const maxCoverageXlm = baseUnitsToAmount(terms.maxCoveragePerDeposit, 7)
  const minDepositXlm = baseUnitsToAmount(terms.minDepositAmount, 7)
  const claimProcessingHours = Math.round(terms.claimProcessingTime / 3600)

  return (
    <>
      {/* Backdrop */}
      <div
        className="fixed inset-0 bg-black/50 z-40 transition-opacity"
        onClick={onClose}
        role="button"
        tabIndex={0}
      />

      {/* Modal */}
      <div className="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 w-full max-w-2xl max-h-[90vh] overflow-y-auto z-50 rounded-lg bg-slate-900 border border-slate-700/60 shadow-2xl">
        {/* Header */}
        <div className="sticky top-0 bg-slate-900 border-b border-slate-700/60 px-6 py-4 flex items-center justify-between">
          <div>
            <h2 className="text-xl font-semibold text-slate-100">Insurance Terms & Conditions</h2>
            <p className="text-sm text-slate-400 mt-1">Version {terms.version}</p>
          </div>
          <button
            onClick={onClose}
            className="text-slate-400 hover:text-slate-200 transition-colors"
          >
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {/* Content */}
        <div className="px-6 py-6 space-y-8">
          {/* Overview */}
          <section>
            <h3 className="text-lg font-semibold text-slate-100 mb-3">Overview</h3>
            <p className="text-slate-300 text-sm leading-relaxed">
              The SAFE-HAVEN Insurance Pool provides protection for eligible time-locked deposits on
              the Stellar network. This insurance covers losses due to smart contract failures or
              other covered events, up to the pool's available balance and per-deposit limits.
            </p>
          </section>

          {/* Coverage Details */}
          <section>
            <h3 className="text-lg font-semibold text-slate-100 mb-4">Coverage Details</h3>
            <div className="space-y-3">
              <div className="border border-slate-700/60 rounded p-4 bg-slate-800/30">
                <p className="text-sm text-slate-400 mb-1">Maximum Coverage Per Deposit</p>
                <p className="text-2xl font-bold text-stellar-400">{maxCoverageXlm} XLM</p>
                <p className="text-xs text-slate-500 mt-2">
                  Coverage is capped at {terms.maxCoveragePercentage}% of deposit value or {maxCoverageXlm} XLM, whichever is lower
                </p>
              </div>

              <div className="border border-slate-700/60 rounded p-4 bg-slate-800/30">
                <p className="text-sm text-slate-400 mb-1">Minimum Deposit Amount</p>
                <p className="text-2xl font-bold text-slate-100">{minDepositXlm} XLM</p>
                <p className="text-xs text-slate-500 mt-2">
                  Deposits below this amount do not qualify for insurance protection
                </p>
              </div>

              <div className="border border-slate-700/60 rounded p-4 bg-slate-800/30">
                <p className="text-sm text-slate-400 mb-1">Maximum Lock Duration</p>
                <p className="text-2xl font-bold text-slate-100">{formatDuration(terms.maxLockDuration)}</p>
                <p className="text-xs text-slate-500 mt-2">
                  Coverage applies to deposits locked for up to {formatDuration(terms.maxLockDuration)}
                </p>
              </div>
            </div>
          </section>

          {/* Eligibility */}
          <section>
            <h3 className="text-lg font-semibold text-slate-100 mb-3">Eligibility Criteria</h3>
            <div className="space-y-2">
              <div className="flex gap-3 text-sm">
                <div className="w-6 h-6 flex items-center justify-center rounded-full bg-emerald-500/20 flex-shrink-0 text-emerald-400 font-bold">
                  ✓
                </div>
                <div>
                  <p className="text-slate-200">Deposit amount ≥ {minDepositXlm} XLM</p>
                  <p className="text-xs text-slate-400">Minimum deposit threshold must be met</p>
                </div>
              </div>

              <div className="flex gap-3 text-sm">
                <div className="w-6 h-6 flex items-center justify-center rounded-full bg-emerald-500/20 flex-shrink-0 text-emerald-400 font-bold">
                  ✓
                </div>
                <div>
                  <p className="text-slate-200">Early exit penalty ≤ 50%</p>
                  <p className="text-xs text-slate-400">High penalty deposits are excluded</p>
                </div>
              </div>

              <div className="flex gap-3 text-sm">
                <div className="w-6 h-6 flex items-center justify-center rounded-full bg-emerald-500/20 flex-shrink-0 text-emerald-400 font-bold">
                  ✓
                </div>
                <div>
                  <p className="text-slate-200">Lock duration ≥ 24 hours</p>
                  <p className="text-xs text-slate-400">Deposits must be locked for at least one day</p>
                </div>
              </div>

              <div className="flex gap-3 text-sm">
                <div className="w-6 h-6 flex items-center justify-center rounded-full bg-emerald-500/20 flex-shrink-0 text-emerald-400 font-bold">
                  ✓
                </div>
                <div>
                  <p className="text-slate-200">Insurance pool has available balance</p>
                  <p className="text-xs text-slate-400">Pool must have sufficient funds for coverage</p>
                </div>
              </div>
            </div>
          </section>

          {/* Claim Process */}
          <section>
            <h3 className="text-lg font-semibold text-slate-100 mb-3">Claim Process</h3>
            <ol className="space-y-3">
              {[
                { step: 1, title: 'Submit Claim', desc: 'File a claim through the claims portal with supporting documentation' },
                { step: 2, title: 'Initial Review', desc: 'Claims team reviews the claim against policy terms' },
                { step: 3, title: 'Investigation', desc: 'Further investigation if needed (typically 2-5 business days)' },
                { step: 4, title: 'Decision', desc: `Claim decision within ${claimProcessingHours} hours` },
                { step: 5, title: 'Payment', desc: 'Approved claims paid within 2-3 business days' },
              ].map(({ step, title, desc }) => (
                <li key={step} className="flex gap-3">
                  <div className="w-8 h-8 flex items-center justify-center rounded-full bg-stellar-500/20 text-stellar-400 font-semibold flex-shrink-0">
                    {step}
                  </div>
                  <div className="pt-0.5">
                    <p className="font-medium text-slate-200">{title}</p>
                    <p className="text-xs text-slate-400">{desc}</p>
                  </div>
                </li>
              ))}
            </ol>
          </section>

          {/* Exclusions */}
          <section>
            <h3 className="text-lg font-semibold text-slate-100 mb-3">Exclusions</h3>
            <div className="space-y-2">
              {terms.exclusions.map((exclusion, idx) => (
                <div key={idx} className="flex gap-2 text-sm">
                  <span className="text-red-400 font-bold">✕</span>
                  <span className="text-slate-300">{exclusion}</span>
                </div>
              ))}
            </div>
          </section>

          {/* Important Notes */}
          <section>
            <h3 className="text-lg font-semibold text-slate-100 mb-3">Important Notes</h3>
            <div className="bg-amber-500/10 border border-amber-500/40 rounded p-4 text-sm text-amber-200 space-y-2">
              <p>
                • Insurance coverage is provided on an "as-is" basis without warranties of any kind
              </p>
              <p>
                • Coverage is limited to the available balance in the insurance pool at the time of
                claim approval
              </p>
              <p>
                • The insurance pool may be depleted if multiple claims exceed the available balance
              </p>
              <p>
                • Claims must be filed within 90 days of a covered loss event to be eligible for
                payment
              </p>
              <p>
                • Insurance coverage excludes normal contract operations, user error, and regulatory
                changes
              </p>
            </div>
          </section>

          {/* Last Updated */}
          <div className="text-xs text-slate-500 border-t border-slate-700/60 pt-4">
            Terms version {terms.version} • Effective{' '}
            {new Date(terms.effectiveDate * 1000).toLocaleDateString()}
          </div>
        </div>

        {/* Footer */}
        <div className="sticky bottom-0 bg-slate-900 border-t border-slate-700/60 px-6 py-4 flex gap-3">
          <button
            onClick={onClose}
            className="flex-1 px-4 py-2 bg-slate-700 hover:bg-slate-600 text-slate-100 rounded-lg font-medium text-sm transition-colors"
          >
            Close
          </button>
          <button
            onClick={onClose}
            className="flex-1 px-4 py-2 bg-stellar-600 hover:bg-stellar-700 text-white rounded-lg font-medium text-sm transition-colors"
          >
            I Understand
          </button>
        </div>
      </div>
    </>
  )
}
