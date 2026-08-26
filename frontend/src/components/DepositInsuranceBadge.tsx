/**
 * Insurance badge component - displays coverage for individual deposits
 */

import type { InsuranceCoverage, ClaimStatus } from '../types'
import { baseUnitsToAmount } from '../lib/format'

interface DepositInsuranceBadgeProps {
  coverage: InsuranceCoverage | null
  claimStatus: ClaimStatus | null
  compact?: boolean
}

export function DepositInsuranceBadge({
  coverage,
  claimStatus,
  compact = false,
}: DepositInsuranceBadgeProps) {
  if (!coverage) {
    return null
  }

  const coveredAmount = baseUnitsToAmount(coverage.amount, 7)
  const maxAmount = baseUnitsToAmount(coverage.maxCoveredAmount, 7)

  const isEligible = coverage.isEligible
  const isCovered = isEligible && coverage.coveragePercentage > 0
  const hasClaim = claimStatus && claimStatus.status !== 'none'

  if (compact) {
    return (
      <div className="flex items-center gap-2">
        {isCovered && (
          <span className="inline-flex items-center gap-1 px-2 py-1 bg-emerald-500/20 border border-emerald-500/40 rounded text-xs font-medium text-emerald-300">
            <svg className="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
              <path fillRule="evenodd" d="M6.267 3.455a3.066 3.066 0 001.745-.723 3.066 3.066 0 013.976 0 3.066 3.066 0 001.745.723 3.066 3.066 0 012.812 3.062v6.756a3.066 3.066 0 01-3.062 3.062H7.248a3.066 3.066 0 01-3.062-3.062V6.517a3.066 3.066 0 012.812-3.062zM9 11a1 1 0 11-2 0 1 1 0 012 0z" clipRule="evenodd" />
            </svg>
            Insured
          </span>
        )}

        {!isEligible && (
          <span className="inline-flex items-center gap-1 px-2 py-1 bg-slate-500/20 border border-slate-500/40 rounded text-xs font-medium text-slate-300">
            <svg className="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
              <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" />
            </svg>
            Not Eligible
          </span>
        )}

        {hasClaim && claimStatus && (
          <span
            className={`inline-flex items-center gap-1 px-2 py-1 rounded text-xs font-medium ${
              claimStatus.status === 'approved'
                ? 'bg-emerald-500/20 border border-emerald-500/40 text-emerald-300'
                : claimStatus.status === 'pending'
                  ? 'bg-amber-500/20 border border-amber-500/40 text-amber-300'
                  : 'bg-red-500/20 border border-red-500/40 text-red-300'
            }`}
          >
            {claimStatus.status === 'approved' && '✓'} {claimStatus.status === 'pending' && '⏳'}
            {claimStatus.status === 'rejected' && '✗'}
            Claim {claimStatus.status}
          </span>
        )}
      </div>
    )
  }

  // Full view
  return (
    <div className="border border-slate-700/60 rounded-lg p-4 bg-slate-900/30 space-y-3">
      {/* Header */}
      <div className="flex items-start justify-between">
        <h3 className="font-medium text-slate-200">Insurance Coverage</h3>
        {isCovered && (
          <span className="inline-flex items-center gap-1 px-2 py-1 bg-emerald-500/20 border border-emerald-500/40 rounded text-xs font-medium text-emerald-300">
            ✓ Protected
          </span>
        )}
        {!isEligible && (
          <span className="inline-flex items-center gap-1 px-2 py-1 bg-slate-500/20 border border-slate-500/40 rounded text-xs font-medium text-slate-300">
            Not Eligible
          </span>
        )}
      </div>

      {/* Coverage Details */}
      {isCovered && (
        <>
          <div className="space-y-2">
            <div className="flex items-center justify-between text-sm">
              <span className="text-slate-400">Coverage Amount</span>
              <span className="font-medium text-slate-100">{coveredAmount} XLM</span>
            </div>
            <div className="flex items-center justify-between text-sm">
              <span className="text-slate-400">Maximum Coverage</span>
              <span className="font-medium text-slate-100">{maxAmount} XLM</span>
            </div>
            <div className="flex items-center justify-between text-sm">
              <span className="text-slate-400">Coverage Percentage</span>
              <span className="font-medium text-stellar-400">{coverage.coveragePercentage}%</span>
            </div>
          </div>

          {/* Coverage Bar */}
          <div className="space-y-1">
            <div className="w-full h-2 bg-slate-800 rounded-full overflow-hidden">
              <div
                className="h-full bg-gradient-to-r from-stellar-500 to-emerald-500 rounded-full"
                style={{
                  width: `${Math.min((Number(coverage.amount) / Number(coverage.maxCoveredAmount)) * 100, 100)}%`,
                }}
              />
            </div>
            <p className="text-xs text-slate-400">
              {coverage.coveragePercentage}% of deposit amount covered
            </p>
          </div>
        </>
      )}

      {!isEligible && (
        <div className="bg-slate-800/30 border border-slate-700/40 rounded p-3 text-xs text-slate-300">
          <p className="font-medium text-slate-200 mb-1">Not eligible for coverage</p>
          <p>
            This deposit does not meet insurance requirements. Check terms for eligibility
            criteria.
          </p>
        </div>
      )}

      {/* Claim Status */}
      {hasClaim && claimStatus && (
        <div
          className={`border rounded p-3 text-sm space-y-2 ${
            claimStatus.status === 'approved'
              ? 'border-emerald-500/40 bg-emerald-500/10'
              : claimStatus.status === 'pending'
                ? 'border-amber-500/40 bg-amber-500/10'
                : 'border-red-500/40 bg-red-500/10'
          }`}
        >
          <p className="font-medium text-slate-200">
            Claim Status:{' '}
            <span
              className={
                claimStatus.status === 'approved'
                  ? 'text-emerald-400'
                  : claimStatus.status === 'pending'
                    ? 'text-amber-400'
                    : 'text-red-400'
              }
            >
              {claimStatus.status.charAt(0).toUpperCase() + claimStatus.status.slice(1)}
            </span>
          </p>

          {claimStatus.claimAmount && (
            <p className="text-slate-300">
              Claim Amount: {baseUnitsToAmount(claimStatus.claimAmount, 7)} XLM
            </p>
          )}

          {claimStatus.rejectionReason && (
            <p className="text-red-300 text-xs">Reason: {claimStatus.rejectionReason}</p>
          )}
        </div>
      )}

      {/* Info */}
      <p className="text-xs text-slate-500 italic">
        Coverage is automatic for eligible deposits. Claims can be filed through the insurance
        portal.
      </p>
    </div>
  )
}
