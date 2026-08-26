/**
 * Claim Status Tracker - displays claim information and status
 */

import type { ClaimStatus, Deposit } from '../types'
import { baseUnitsToAmount } from '../lib/format'

interface ClaimStatusTrackerProps {
  claims: Record<number, ClaimStatus>
  deposits: Deposit[]
  compact?: boolean
}

export function ClaimStatusTracker({ claims, deposits, compact = false }: ClaimStatusTrackerProps) {
  // Filter deposits with claims
  const claimingDeposits = deposits.filter((d) => {
    const claim = claims[d.depositId]
    return claim && claim.status !== 'none'
  })

  if (claimingDeposits.length === 0) {
    return null
  }

  if (compact) {
    // Show summary badge
    const approved = claimingDeposits.filter((d) => claims[d.depositId].status === 'approved')
      .length
    const pending = claimingDeposits.filter((d) => claims[d.depositId].status === 'pending').length

    return (
      <div className="flex items-center gap-2">
        {approved > 0 && (
          <span className="inline-flex items-center gap-1 px-2 py-1 bg-emerald-500/20 border border-emerald-500/40 rounded text-xs font-medium text-emerald-300">
            <svg className="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
              <path
                fillRule="evenodd"
                d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                clipRule="evenodd"
              />
            </svg>
            {approved} Approved
          </span>
        )}
        {pending > 0 && (
          <span className="inline-flex items-center gap-1 px-2 py-1 bg-amber-500/20 border border-amber-500/40 rounded text-xs font-medium text-amber-300">
            <svg className="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
              <path d="M10 18a8 8 0 100-16 8 8 0 000 16zm1-11a1 1 0 11-2 0 1 1 0 012 0m0 4a1 1 0 11-2 0 1 1 0 012 0z" />
            </svg>
            {pending} Pending
          </span>
        )}
      </div>
    )
  }

  // Full view with list
  return (
    <div className="border border-slate-700/60 rounded-lg p-6 bg-slate-900/30 space-y-4">
      <div>
        <h3 className="text-lg font-semibold text-slate-100">Claim Status</h3>
        <p className="text-sm text-slate-400 mt-1">
          {claimingDeposits.length} claim{claimingDeposits.length !== 1 ? 's' : ''} being tracked
        </p>
      </div>

      {/* Claims List */}
      <div className="space-y-3">
        {claimingDeposits.map((deposit) => {
          const claim = claims[deposit.depositId]
          if (!claim) return null

          const statusColors = {
            pending: 'border-amber-500/40 bg-amber-500/10',
            approved: 'border-emerald-500/40 bg-emerald-500/10',
            rejected: 'border-red-500/40 bg-red-500/10',
            paid: 'border-blue-500/40 bg-blue-500/10',
          }

          const statusTextColors = {
            pending: 'text-amber-400',
            approved: 'text-emerald-400',
            rejected: 'text-red-400',
            paid: 'text-blue-400',
          }

          const statusIcons = {
            pending: '⏳',
            approved: '✓',
            rejected: '✗',
            paid: '✓✓',
          }

          return (
            <div
              key={deposit.depositId}
              className={`border rounded-lg p-4 ${statusColors[claim.status as keyof typeof statusColors]}`}
            >
              <div className="flex items-start justify-between gap-4 mb-3">
                <div>
                  <p className="font-medium text-slate-100">Deposit #{deposit.depositId}</p>
                  <p className="text-sm text-slate-400 mt-0.5">
                    {baseUnitsToAmount(deposit.amount, 7)} XLM
                  </p>
                </div>
                <div className="text-right">
                  <p
                    className={`font-semibold text-sm ${statusTextColors[claim.status as keyof typeof statusTextColors]}`}
                  >
                    {statusIcons[claim.status as keyof typeof statusIcons]}{' '}
                    {claim.status.charAt(0).toUpperCase() + claim.status.slice(1)}
                  </p>
                </div>
              </div>

              {/* Claim Details */}
              <div className="space-y-2 text-sm">
                {claim.claimDate && (
                  <div className="flex justify-between">
                    <span className="text-slate-500">Claim Filed</span>
                    <span className="text-slate-200">
                      {new Date(claim.claimDate * 1000).toLocaleDateString()}
                    </span>
                  </div>
                )}

                {claim.claimAmount && (
                  <div className="flex justify-between">
                    <span className="text-slate-500">Claim Amount</span>
                    <span className="text-slate-200 font-medium">
                      {baseUnitsToAmount(claim.claimAmount, 7)} XLM
                    </span>
                  </div>
                )}

                {claim.approvalDate && (
                  <div className="flex justify-between">
                    <span className="text-slate-500">
                      {claim.status === 'rejected' ? 'Reviewed' : 'Approved'}
                    </span>
                    <span className="text-slate-200">
                      {new Date(claim.approvalDate * 1000).toLocaleDateString()}
                    </span>
                  </div>
                )}

                {claim.rejectionReason && (
                  <div className="mt-2 p-2 bg-red-500/20 border border-red-500/30 rounded text-red-300 text-xs">
                    <p className="font-medium mb-1">Rejection Reason</p>
                    <p>{claim.rejectionReason}</p>
                  </div>
                )}
              </div>

              {/* Status Timeline */}
              <div className="mt-3 flex gap-1">
                <div
                  className={`flex-1 h-1 rounded-full ${claim.status !== 'none' ? 'bg-amber-500' : 'bg-slate-700'}`}
                />
                <div
                  className={`flex-1 h-1 rounded-full ${
                    claim.status === 'approved' || claim.status === 'rejected' || claim.status === 'paid'
                      ? claim.status === 'paid'
                        ? 'bg-blue-500'
                        : 'bg-emerald-500'
                      : 'bg-slate-700'
                  }`}
                />
                <div
                  className={`flex-1 h-1 rounded-full ${claim.status === 'paid' ? 'bg-blue-500' : 'bg-slate-700'}`}
                />
              </div>
            </div>
          )
        })}
      </div>

      {/* Info */}
      <div className="bg-slate-800/30 border border-slate-700/40 rounded p-3 text-xs text-slate-300 space-y-1">
        <p className="font-medium text-slate-200">Claim Status Information</p>
        <ul className="space-y-1 text-slate-400">
          <li>• Pending: Claim submitted and under review</li>
          <li>• Approved: Claim verified and approved for payment</li>
          <li>• Rejected: Claim denied due to policy exclusions</li>
          <li>• Paid: Claim payment has been processed</li>
        </ul>
      </div>
    </div>
  )
}
