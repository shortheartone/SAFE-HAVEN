/**
 * Hook for querying insurance pool details
 * Integrates with the contract to fetch insurance-related data
 */

import { useEffect, useState } from 'react'
import type {
  InsurancePool,
  InsuranceCoverage,
  ClaimStatus,
  InsuranceTerms,
  Deposit,
} from '../types'

interface UseInsurancePoolReturn {
  pool: InsurancePool | null
  coverage: Record<number, InsuranceCoverage> // keyed by depositId
  claims: Record<number, ClaimStatus>         // keyed by depositId
  terms: InsuranceTerms | null
  loading: boolean
  error: string | null
  refresh: () => Promise<void>
}

/**
 * Mock insurance terms (placeholder for future contract integration)
 */
const MOCK_INSURANCE_TERMS: InsuranceTerms = {
  version: '1.0.0',
  effectiveDate: Math.floor(Date.now() / 1000),
  maxCoveragePercentage: 100,
  maxCoveragePerDeposit: BigInt('10000000000000'), // 10M stroops (~100k XLM)
  minDepositAmount: BigInt('1000000'), // 0.01 XLM
  maxLockDuration: 5 * 365 * 24 * 60 * 60, // 5 years
  claimProcessingTime: 7 * 24 * 60 * 60, // 7 days
  exclusions: [
    'Deposits with penalties > 50%',
    'Deposits locked for < 24 hours',
    'Deposits during contract maintenance',
  ],
}

/**
 * Calculate insurance coverage for a deposit
 * @param deposit The deposit to calculate coverage for
 * @param poolBalance The total insurance pool balance
 * @returns Insurance coverage information
 */
function calculateCoverage(deposit: Deposit, poolBalance: bigint): InsuranceCoverage {
  const amount = deposit.amount
  const maxCovered = MOCK_INSURANCE_TERMS.maxCoveragePerDeposit
  const coveredAmount = amount > maxCovered ? maxCovered : amount

  // Check eligibility
  const penaltyNotTooHigh = deposit.penaltyBps <= 5000 // 50%
  const minDurationMet = Math.floor(Date.now() / 1000) + 24 * 60 * 60 <= deposit.unlockTime
  const isEligible = penaltyNotTooHigh && minDurationMet

  return {
    depositId: deposit.depositId,
    amount: coveredAmount,
    coveragePercentage: isEligible ? 100 : 0,
    maxCoveredAmount: maxCovered,
    isEligible,
  }
}

/**
 * Generate mock insurance pool data
 * In production, this would query the contract
 */
function generateMockPoolData(deposits: Deposit[]): InsurancePool {
  const eligibleDeposits = deposits.filter((d) => d.penaltyBps <= 5000)
  const totalCoverage = eligibleDeposits.reduce((sum, d) => {
    const cov = calculateCoverage(d, BigInt(0))
    return sum + cov.amount
  }, BigInt(0))

  return {
    totalPoolBalance: BigInt('500000000000'), // 5M stroops (~50k XLM) - mock value
    totalDepositsInsured: eligibleDeposits.length,
    totalCoverageAmount: totalCoverage,
    coveragePercentage:
      deposits.length > 0
        ? Math.round((Number(totalCoverage) / Number(totalCoverage + BigInt('1000000000000'))) * 100)
        : 0,
    claimsApproved: 3,
    claimsPending: 1,
    lastUpdated: Math.floor(Date.now() / 1000),
  }
}

/**
 * Generate mock claim statuses
 * In production, this would query the contract
 */
function generateMockClaimData(deposits: Deposit[]): Record<number, ClaimStatus> {
  const claims: Record<number, ClaimStatus> = {}

  deposits.forEach((deposit, idx) => {
    // Mock: some deposits have claims
    const claimChance = idx % 5
    if (claimChance === 0 && idx > 0) {
      const statuses: Array<'pending' | 'approved' | 'rejected'> = [
        'pending',
        'approved',
        'rejected',
      ]
      const status = statuses[idx % 3]

      claims[deposit.depositId] = {
        depositId: deposit.depositId,
        status,
        claimAmount: deposit.amount,
        claimDate: Math.floor(Date.now() / 1000) - idx * 24 * 60 * 60,
        approvalDate:
          status === 'approved' || status === 'rejected'
            ? Math.floor(Date.now() / 1000) - (idx - 2) * 24 * 60 * 60
            : undefined,
        rejectionReason: status === 'rejected' ? 'Deposit not eligible for coverage' : undefined,
      }
    } else {
      claims[deposit.depositId] = {
        depositId: deposit.depositId,
        status: 'none',
      }
    }
  })

  return claims
}

export function useInsurancePool(deposits: Deposit[]): UseInsurancePoolReturn {
  const [pool, setPool] = useState<InsurancePool | null>(null)
  const [coverage, setCoverage] = useState<Record<number, InsuranceCoverage>>({})
  const [claims, setClaims] = useState<Record<number, ClaimStatus>>({})
  const [terms, setTerms] = useState<InsuranceTerms | null>(MOCK_INSURANCE_TERMS)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const refresh = async () => {
    try {
      setLoading(true)
      setError(null)

      // In production: fetch from contract
      // const poolData = await contract.call('get_insurance_pool')
      // const claimsData = await contract.call('get_claims', { depositor })

      // For now: use mock data
      const poolData = generateMockPoolData(deposits)
      const claimsData = generateMockClaimData(deposits)

      // Calculate coverage for each deposit
      const coverageMap: Record<number, InsuranceCoverage> = {}
      deposits.forEach((deposit) => {
        coverageMap[deposit.depositId] = calculateCoverage(deposit, poolData.totalPoolBalance)
      })

      setPool(poolData)
      setCoverage(coverageMap)
      setClaims(claimsData)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch insurance data')
    } finally {
      setLoading(false)
    }
  }

  // Fetch on mount and when deposits change
  useEffect(() => {
    if (deposits.length > 0) {
      refresh()
    }
  }, [deposits])

  return {
    pool,
    coverage,
    claims,
    terms,
    loading,
    error,
    refresh,
  }
}
