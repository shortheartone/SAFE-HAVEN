/**
 * Insurance Pool component - displays pool overview and statistics
 */

import { useState } from 'react'
import type { InsurancePool, InsuranceTerms } from '../types'
import { baseUnitsToAmount } from '../lib/format'
import { InsuranceTermsModal } from './InsuranceTermsModal'

interface InsurancePoolProps {
  pool: InsurancePool | null
  terms: InsuranceTerms | null
  loading: boolean
}

export function InsurancePool({ pool, terms, loading }: InsurancePoolProps) {
  const [showTerms, setShowTerms] = useState(false)

  if (!pool || !terms) {
    return null
  }

  const poolBalanceXlm = baseUnitsToAmount(pool.totalPoolBalance, 7)
  const totalCoverageXlm = baseUnitsToAmount(pool.totalCoverageAmount, 7)

  return (
    <>
      <div className="border border-slate-700/60 rounded-lg p-6 bg-slate-900/30 space-y-6">
        {/* Header */}
        <div className="flex items-start justify-between">
          <div>
            <h2 className="text-lg font-semibold text-slate-100">Insurance Protection</h2>
            <p className="text-sm text-slate-400 mt-1">Pool details and coverage status</p>
          </div>
          <button
            onClick={() => setShowTerms(true)}
            className="px-3 py-1 text-sm bg-slate-700 hover:bg-slate-600 text-slate-200 rounded transition-colors"
          >
            View Terms
          </button>
        </div>

        {/* Pool Balance */}
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div className="bg-slate-800/50 border border-slate-700/40 rounded p-4">
            <p className="text-xs text-slate-500 uppercase tracking-wider">Pool Balance</p>
            <p className="text-2xl font-bold text-slate-100 mt-2">{poolBalanceXlm}</p>
            <p className="text-xs text-slate-400 mt-1">XLM</p>
          </div>

          <div className="bg-slate-800/50 border border-slate-700/40 rounded p-4">
            <p className="text-xs text-slate-500 uppercase tracking-wider">Total Coverage</p>
            <p className="text-2xl font-bold text-stellar-300 mt-2">{totalCoverageXlm}</p>
            <p className="text-xs text-slate-400 mt-1">XLM Covered</p>
          </div>
        </div>

        {/* Statistics */}
        <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
          {/* Coverage % */}
          <div className="bg-slate-800/50 border border-slate-700/40 rounded p-3">
            <p className="text-xs text-slate-500">Coverage %</p>
            <p className="text-lg font-semibold text-stellar-400 mt-1">
              {pool.coveragePercentage}%
            </p>
          </div>

          {/* Insured Deposits */}
          <div className="bg-slate-800/50 border border-slate-700/40 rounded p-3">
            <p className="text-xs text-slate-500">Insured</p>
            <p className="text-lg font-semibold text-slate-100 mt-1">
              {pool.totalDepositsInsured}
            </p>
          </div>

          {/* Approved Claims */}
          <div className="bg-slate-800/50 border border-slate-700/40 rounded p-3">
            <p className="text-xs text-slate-500">Approved</p>
            <p className="text-lg font-semibold text-emerald-400 mt-1">
              {pool.claimsApproved}
            </p>
          </div>

          {/* Pending Claims */}
          <div className="bg-slate-800/50 border border-slate-700/40 rounded p-3">
            <p className="text-xs text-slate-500">Pending</p>
            <p className="text-lg font-semibold text-amber-400 mt-1">
              {pool.claimsPending}
            </p>
          </div>
        </div>

        {/* Status Indicators */}
        <div className="space-y-2">
          <div className="flex items-center gap-2 text-sm">
            <div className="w-2 h-2 rounded-full bg-emerald-500" />
            <span className="text-slate-300">
              {pool.totalDepositsInsured} deposit{pool.totalDepositsInsured !== 1 ? 's' : ''} are
              insured
            </span>
          </div>
          <div className="flex items-center gap-2 text-sm">
            <div className="w-2 h-2 rounded-full bg-stellar-500" />
            <span className="text-slate-300">
              Pool covers up to {pool.coveragePercentage}% of total locked value
            </span>
          </div>
          <div className="flex items-center gap-2 text-sm">
            <div className="w-2 h-2 rounded-full bg-amber-500" />
            <span className="text-slate-300">
              {pool.claimsPending} claim{pool.claimsPending !== 1 ? 's' : ''} pending review
            </span>
          </div>
        </div>

        {/* Info Box */}
        <div className="bg-slate-800/30 border border-slate-700/40 rounded p-4 text-sm text-slate-300 space-y-2">
          <p className="font-medium text-slate-200">ℹ️ How Insurance Works</p>
          <ul className="space-y-1 text-xs">
            <li className="flex gap-2">
              <span className="text-slate-500">•</span>
              <span>Qualifying deposits automatically receive coverage up to pool limits</span>
            </li>
            <li className="flex gap-2">
              <span className="text-slate-500">•</span>
              <span>Coverage percentage reflects pool's ability to cover losses</span>
            </li>
            <li className="flex gap-2">
              <span className="text-slate-500">•</span>
              <span>Claims are reviewed within 7 days of submission</span>
            </li>
          </ul>
        </div>
      </div>

      {/* Terms Modal */}
      {showTerms && (
        <InsuranceTermsModal terms={terms} onClose={() => setShowTerms(false)} />
      )}
    </>
  )
}
