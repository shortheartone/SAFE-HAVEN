// ============================================================
//  Shared TypeScript types — mirrors the Soroban contract types
// ============================================================

/** Mirrors the Rust VaultEntry struct */
export interface VaultEntry {
  token: string       // Stellar address
  amount: bigint      // in token base units (stroops for XLM)
  unlockTime: number  // Unix timestamp (seconds)
  depositor: string   // Stellar address
  penaltyBps: number  // 0–10_000 (basis points)
}

/** A deposit with its ID attached */
export interface Deposit extends VaultEntry {
  depositId: number
  /** Seconds remaining until unlock (0 if unlocked, null during initial load) */
  timeRemaining: number | null
  /**
   * Whether the chain has confirmed this deposit is truly unlocked.
   *
   * - `true`  for all freshly-loaded deposits (timeRemaining was computed from
   *           a live getLedgerTime() call, so the value is chain-authoritative).
   * - `true`  after a getTimeRemaining() call confirms the value is 0.
   * - `false` while the local ticker has just ticked down to 0 but a chain
   *           re-verification is still in-flight. The Withdraw button must be
   *           hidden while this is false to prevent premature FundsStillLocked
   *           errors due to clock drift or tab-throttling.
   */
  unlockVerified: boolean
}

/** Result of wallet connection */
export interface WalletInfo {
  address: string
  displayAddress: string
  networkMismatch?: boolean  // True if wallet network differs from app network
  walletNetwork?: string     // Network passphrase from Freighter (for display)
}

/** Tab pages */
export type PageTab = 'dashboard' | 'deposit' | 'withdraw' | 'admin' | 'settings'

/** Loading states for async operations */
export type TxStatus = 'idle' | 'signing' | 'submitting' | 'confirming' | 'success' | 'error'

/** Contract call result wrapper */
export interface ContractResult<T> {
  success: boolean
  data?: T
  error?: string
  txHash?: string
}

/** Discriminated union for wallet signing results */
export type SigningResult = 
  | { signed: true; xdr: string }           // Successfully signed
  | { signed: false; rejected: true }       // User rejected the signing request
  | { signed: false; rejected: false; error: string } // Signing failed with error

/** Deposit template for settings */
export interface DepositTemplate {
  tokenAddress: string      // Stellar token address
  lockDurationSeconds: number // Lock duration in seconds
  penaltyBps: number        // Penalty in basis points
  label?: string            // Optional friendly name
}

/** User settings including templates and preferences */
export interface UserSettings {
  version: string           // Settings format version (e.g., "1.0.0")
  exportedAt: number        // Unix timestamp when exported
  depositTemplates: DepositTemplate[]
  frequentTokens: string[]  // List of frequently used token addresses
}

/** Settings with merge metadata */
export interface SettingsWithMergeInfo extends UserSettings {
  mergeAction?: 'replace' | 'merge'  // How to apply on import
}

/** Validation result for imported settings */
export interface SettingsValidation {
  valid: boolean
  errors: string[]
  warnings: string[]
}

/** Insurance coverage for a specific deposit */
export interface InsuranceCoverage {
  depositId: number
  amount: bigint              // Covered amount in base units
  coveragePercentage: number  // 0-100, e.g., 100 means 100% covered
  maxCoveredAmount: bigint    // Maximum amount that can be covered
  isEligible: boolean         // Whether deposit qualifies for insurance
}

/** Insurance pool details */
export interface InsurancePool {
  totalPoolBalance: bigint    // Total balance in insurance pool
  totalDepositsInsured: number // Count of insured deposits
  totalCoverageAmount: bigint  // Total coverage provided
  coveragePercentage: number   // Pool coverage % of total locked value
  claimsApproved: number       // Number of approved claims
  claimsPending: number        // Number of pending claims
  lastUpdated: number          // Unix timestamp
}

/** Claim status for a deposit */
export interface ClaimStatus {
  depositId: number
  status: 'none' | 'pending' | 'approved' | 'rejected' | 'paid'
  claimAmount?: bigint
  claimDate?: number           // Unix timestamp
  approvalDate?: number        // Unix timestamp when claim was approved
  rejectionReason?: string
}

/** Insurance terms and conditions */
export interface InsuranceTerms {
  version: string
  effectiveDate: number
  maxCoveragePercentage: number  // e.g., 100 (means 100%)
  maxCoveragePerDeposit: bigint
  minDepositAmount: bigint
  maxLockDuration: number        // seconds
  claimProcessingTime: number    // seconds (approx)
  exclusions: string[]
}
