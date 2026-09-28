// ============================================================
//  Hook: load all deposits for the connected wallet
// ============================================================

import { useCallback, useEffect, useRef, useState } from 'react'
import { getDepositIds, getDepositBatch, getLedgerTime } from '../lib/stellar'
import type { Deposit } from '../types'

interface UseDepositsResult {
  deposits: Deposit[]
  loading: boolean
  error: string | null
  refresh: () => Promise<void>
  pollRemoveDeposit: (depositId: number, maxAttempts?: number) => Promise<void>
}

export function useDeposits(depositorAddress: string | null): UseDepositsResult {
  const [deposits, setDeposits] = useState<Deposit[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const abortRef = useRef<AbortController | null>(null)

  const refresh = useCallback(async () => {
    if (!depositorAddress) {
      setDeposits([])
      return
    }

    // Cancel any in-flight fetch
    abortRef.current?.abort()
    const ctrl = new AbortController()
    abortRef.current = ctrl

    setLoading(true)
    setError(null)

    try {
      const ids = await getDepositIds(depositorAddress)
      if (ctrl.signal.aborted) return

      // Get current ledger time for computing timeRemaining
      const now = await getLedgerTime()
      if (ctrl.signal.aborted) return

      // Fetch all vault entries in batches (max 25 per call)
      const batchSize = 25
      const allDeposits: Deposit[] = []

      for (let i = 0; i < ids.length; i += batchSize) {
        const batch = ids.slice(i, i + batchSize)
        const results = await getDepositBatch(depositorAddress, batch)
        if (ctrl.signal.aborted) return

        for (let j = 0; j < results.length; j++) {
          const { entry } = results[j]
          if (entry) {
            const timeRemaining = Math.max(0, entry.unlockTime - now)
            allDeposits.push({
              ...entry,
              depositId: batch[j],
              timeRemaining,
            })
          }
        }
      }

      if (ctrl.signal.aborted) return
      setDeposits(allDeposits)
    } catch (e) {
      if (ctrl.signal.aborted) return
      setError(e instanceof Error ? e.message : 'Failed to load deposits')
    } finally {
      if (!ctrl.signal.aborted) setLoading(false)
    }
  }, [depositorAddress])

  /**
   * Poll a single deposit until it is removed (getVault returns null),
   * then remove it from local state immediately.
   * Polls with exponential backoff: 500ms, 1s, 2s, 4s, 8s.
   */
  const pollRemoveDeposit = useCallback(async (depositId: number, maxAttempts = 5) => {
    if (!depositorAddress) return

    for (let attempt = 0; attempt < maxAttempts; attempt++) {
      // Exponential backoff: 500ms * 2^attempt
      const delayMs = 500 * Math.pow(2, attempt)
      await new Promise((r) => setTimeout(r, delayMs))

      try {
        const vault = await getVault(depositorAddress, depositId)
        if (vault === null) {
          // Deposit removed on-chain, remove from local state immediately
          setDeposits((prev) => prev.filter((d) => d.depositId !== depositId))
          return
        }
      } catch (e) {
        // Retry on network error
        console.error(`pollRemoveDeposit(${depositId}) attempt ${attempt + 1} failed:`, e)
      }
    }

    // If polling failed, fall back to full refresh
    console.warn(`pollRemoveDeposit(${depositId}) exhausted attempts, falling back to full refresh`)
    await refresh()
  }, [depositorAddress, refresh])

  // Auto-refresh on address change
  useEffect(() => {
    void refresh()
    return () => abortRef.current?.abort()
  }, [refresh])

  // Tick countdown every second
  useEffect(() => {
    const id = setInterval(() => {
      setDeposits((prev) =>
        prev.map((d) => ({
          ...d,
          timeRemaining: d.timeRemaining === null ? null : Math.max(0, d.timeRemaining - 1),
        })),
      )
    }, 1000)
    return () => clearInterval(id)
  }, [])

  return { deposits, loading, error, refresh, pollRemoveDeposit }
}
