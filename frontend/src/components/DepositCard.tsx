import { useState } from 'react'
import type { Deposit } from '../types'
import { stroopsToXlm, formatUnlockDate, formatCountdown, formatBps, shortAddr, explorerAddrUrl } from '../lib/format'
import { CONFIG } from '../config'

interface DepositCardProps {
  deposit: Deposit
  onWithdraw: (depositId: number) => void
  onCancel: (depositId: number) => void
  onRenew: (depositId: number, newUnlockTime: number, penaltyBps: number) => void
  txPending: boolean
}

function toLocalDateTimeValue(timestamp: number): string {
  const date = new Date(timestamp * 1000)
  date.setMinutes(date.getMinutes() - date.getTimezoneOffset())
  return date.toISOString().slice(0, 16)
}

export function DepositCard({ deposit, onWithdraw, onCancel, onRenew, txPending }: DepositCardProps) {
  const [showDetails, setShowDetails] = useState(false)
  const [showRenewal, setShowRenewal] = useState(false)
  const [newUnlockTime, setNewUnlockTime] = useState('')
  const [penaltyBps, setPenaltyBps] = useState(String(deposit.penaltyBps))
  const [renewalError, setRenewalError] = useState<string | null>(null)

  const isXlm   = deposit.token === CONFIG.NATIVE_TOKEN
  const isUnlocked = deposit.timeRemaining !== null && deposit.timeRemaining === 0
  const hasPenalty = deposit.penaltyBps > 0

  const penaltyAmount = isUnlocked
    ? 0n
    : (deposit.amount * BigInt(deposit.penaltyBps)) / 10_000n

  const refundAmount = deposit.amount - penaltyAmount

  function submitRenewal(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault()
    const unlockTime = Math.floor(new Date(newUnlockTime).getTime() / 1000)
    const penalty = Number(penaltyBps)
    const lockDuration = unlockTime - Math.floor(Date.now() / 1000)

    if (!Number.isFinite(unlockTime) || lockDuration < CONFIG.MIN_LOCK_DURATION_SECS) {
      setRenewalError(`Choose an unlock time at least ${CONFIG.MIN_LOCK_DURATION_SECS} seconds from now.`)
      return
    }
    if (lockDuration > CONFIG.MAX_LOCK_DURATION_SECS) {
      setRenewalError(`Maximum renewal term is ${CONFIG.MAX_LOCK_DURATION_SECS} seconds.`)
      return
    }
    if (!Number.isInteger(penalty) || penalty < 0 || penalty > CONFIG.MAX_PENALTY_BPS) {
      setRenewalError(`Penalty must be between 0 and ${CONFIG.MAX_PENALTY_BPS} basis points.`)
      return
    }

    setRenewalError(null)
    onRenew(deposit.depositId, unlockTime, penalty)
  }

  return (
    <div className="card p-5 hover:border-slate-600/80 transition-colors">
      {/* Top row */}
      <div className="flex items-start justify-between gap-3">
        <div className="flex items-center gap-3 min-w-0">
          {/* Token icon placeholder */}
          <div className="w-10 h-10 rounded-full bg-stellar-900/60 border border-stellar-700/40 flex items-center justify-center flex-shrink-0 text-stellar-400 font-bold text-sm">
            {isXlm ? 'XLM' : '?'}
          </div>
          <div className="min-w-0">
            <p className="font-semibold text-base">{stroopsToXlm(deposit.amount)} {isXlm ? 'XLM' : 'tokens'}</p>
            <p className="text-xs text-slate-400 truncate">Deposit #{deposit.depositId}</p>
          </div>
        </div>

        {/* Status badge */}
        {isUnlocked ? (
          <span className="badge-green flex-shrink-0">
            <span className="w-1.5 h-1.5 rounded-full bg-green-400" />
            Unlocked
          </span>
        ) : (
          <span className="badge-yellow flex-shrink-0 countdown-active">
            <span className="w-1.5 h-1.5 rounded-full bg-yellow-400" />
            {formatCountdown(deposit.timeRemaining)}
          </span>
        )}
      </div>

      {/* Unlock date */}
      <div className="mt-3 text-sm text-slate-400">
        {isUnlocked ? (
          <span className="text-green-400">Ready to withdraw</span>
        ) : (
          <>Unlocks <span className="text-slate-200">{formatUnlockDate(deposit.unlockTime)}</span></>
        )}
      </div>

      {/* Penalty info (if any) */}
      {hasPenalty && !isUnlocked && (
        <div className="mt-2 text-xs text-orange-400 bg-orange-900/20 rounded-lg px-3 py-2 border border-orange-800/30">
          Early exit penalty: {formatBps(deposit.penaltyBps)} — you'd receive ~{stroopsToXlm(refundAmount)} {isXlm ? 'XLM' : 'tokens'}
        </div>
      )}

      {/* Expandable details */}
      {showDetails && (
        <div className="mt-4 pt-4 border-t border-slate-700/60 grid grid-cols-2 gap-y-2 gap-x-4 text-xs">
          <span className="text-slate-500">Token</span>
          <a
            href={explorerAddrUrl(deposit.token)}
            target="_blank"
            rel="noopener noreferrer"
            className="text-stellar-400 hover:text-stellar-300 font-mono truncate"
          >
            {shortAddr(deposit.token)}
          </a>
          <span className="text-slate-500">Depositor</span>
          <a
            href={explorerAddrUrl(deposit.depositor)}
            target="_blank"
            rel="noopener noreferrer"
            className="text-stellar-400 hover:text-stellar-300 font-mono truncate"
          >
            {shortAddr(deposit.depositor)}
          </a>
          <span className="text-slate-500">Penalty BPS</span>
          <span className="text-slate-300">{deposit.penaltyBps} ({formatBps(deposit.penaltyBps)})</span>
          <span className="text-slate-500">Unlock time</span>
          <span className="text-slate-300 font-mono">{deposit.unlockTime}</span>
        </div>
      )}

      {showRenewal && isUnlocked && (
        <form onSubmit={submitRenewal} className="mt-4 p-3 rounded-lg border border-stellar-700/40 bg-slate-900/40 space-y-3">
          <label className="block text-xs text-slate-400">
            New unlock date and time
            <input
              className="input mt-1 w-full"
              type="datetime-local"
              min={toLocalDateTimeValue(Math.floor(Date.now() / 1000) + CONFIG.MIN_LOCK_DURATION_SECS)}
              value={newUnlockTime}
              onChange={(event) => setNewUnlockTime(event.target.value)}
              required
              disabled={txPending}
            />
          </label>
          <label className="block text-xs text-slate-400">
            Early-exit penalty (basis points)
            <input
              className="input mt-1 w-full"
              type="number"
              min="0"
              max={CONFIG.MAX_PENALTY_BPS}
              step="1"
              value={penaltyBps}
              onChange={(event) => setPenaltyBps(event.target.value)}
              required
              disabled={txPending}
            />
          </label>
          {renewalError && <p className="text-xs text-red-400">{renewalError}</p>}
          <div className="flex gap-2">
            <button className="btn-primary flex-1" type="submit" disabled={txPending}>
              Renew deposit
            </button>
            <button
              className="btn-secondary"
              type="button"
              onClick={() => { setShowRenewal(false); setRenewalError(null) }}
              disabled={txPending}
            >
              Close
            </button>
          </div>
        </form>
      )}

      {/* Actions */}
      <div className="mt-4 flex items-center gap-2 flex-wrap">
        {isUnlocked ? (
          <>
            <button
              className="btn-primary flex-1"
              onClick={() => onWithdraw(deposit.depositId)}
              disabled={txPending}
            >
              {txPending ? (
                <span className="w-3.5 h-3.5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
              ) : (
                <svg viewBox="0 0 20 20" fill="currentColor" className="w-4 h-4">
                  <path fillRule="evenodd" d="M3 17a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm3.293-7.707a1 1 0 011.414 0L9 10.586V3a1 1 0 112 0v7.586l1.293-1.293a1 1 0 111.414 1.414l-3 3a1 1 0 01-1.414 0l-3-3a1 1 0 010-1.414z" clipRule="evenodd" />
                </svg>
              )}
              Withdraw
            </button>
            <button
              className="btn-secondary flex-1"
              onClick={() => { setShowRenewal((shown) => !shown); setRenewalError(null) }}
              disabled={txPending}
            >
              {showRenewal ? 'Hide renewal' : 'Renew'}
            </button>
          </>
        ) : (
          <button
            className="btn-danger flex-1"
            onClick={() => onCancel(deposit.depositId)}
            disabled={txPending}
          >
            Cancel {hasPenalty ? `(${formatBps(deposit.penaltyBps)} penalty)` : ''}
          </button>
        )}

        <button
          className="btn-secondary text-xs px-3"
          onClick={() => setShowDetails((v) => !v)}
        >
          {showDetails ? 'Less' : 'Details'}
        </button>
      </div>
    </div>
  )
}
