# SAFE-HAVEN Frequently Asked Questions (FAQ)

Welcome to the SAFE-HAVEN FAQ. This document contains answers to the most common questions about setting up, using, troubleshooting, and administering SAFE-HAVEN.

**Quick Links:**
- [Setup & Installation](#setup--installation)
- [Usage & Basic Operations](#usage--basic-operations)
- [Troubleshooting](#troubleshooting)
- [Advanced Topics](#advanced-topics)
- [Community Contributions](#community-contributions)

---

## Setup & Installation

### Q1: What are the prerequisites for running SAFE-HAVEN?

**A:** SAFE-HAVEN requires:

1. **For Smart Contract Development:**
   - Rust 1.81+ (installed via [rustup](https://rustup.rs/))
   - Soroban CLI (latest version)
   - WASM target: `rustup target add wasm32-unknown-unknown`
   - Optional dev tools: `cargo-watch`, `cargo-audit`, `cargo-deny`, `jq`

   Install all at once:
   ```bash
   make install-tools
   ```

2. **For Frontend:**
   - Node.js 20+
   - npm or yarn
   - Freighter wallet browser extension
   - A deployed SAFE-HAVEN contract

3. **General:**
   - A Stellar account (funded with XLM for fees)
   - Network access to Soroban RPC and Horizon APIs

**Reference:** [Quick Start in README](./README.md#quick-start)

---

### Q2: How do I set up a local development environment?

**A:** Use the convenience Makefile target:

```bash
# Install tools, build contract, deploy locally, and start frontend
make dev
```

This runs:
1. `make build` — compiles Rust to WASM
2. `make test` — runs unit tests
3. `make deploy-local` (via Docker) — deploys to a local Soroban network
4. Starts the frontend dev server at `http://localhost:5173`

For manual steps:

```bash
# 1. Build the contract
make build

# 2. Run tests
make test

# 3. Deploy to a local network (requires Docker + docker-compose)
docker-compose up -d
soroban contract deploy --network local ...

# 4. Start frontend
cd frontend
npm install
cp .env.example .env
npm run dev
```

**Reference:** [README Quick Start](./README.md#quick-start)

---

### Q3: How do I deploy to testnet or mainnet?

**A:** 

**For Testnet:**
```bash
export SOROBAN_SECRET_KEY=S...  # Your funded testnet account secret
make deploy-testnet
```

**For Mainnet:**
```bash
export SOROBAN_SECRET_KEY=S...  # Your funded mainnet account secret
make deploy-mainnet
```

**Important:** 
- The deployer account must be funded with XLM to pay transaction fees.
- Mainnet has no faucet; you must acquire XLM beforehand.
- Store deployment artifacts (contract ID, WASM hash) in version control.
- Always simulate before submitting: `soroban contract invoke --network testnet ... --simulate`

**Reference:** [README Deployment](./README.md#quick-start), [DISASTER_RECOVERY.md](./DISASTER_RECOVERY.md)

---

### Q4: How do I configure the frontend for my contract?

**A:**

1. Copy the environment template:
   ```bash
   cd frontend
   cp .env.example .env
   ```

2. Edit `.env` with your values:
   ```env
   # Required
   VITE_CONTRACT_ID=C...                                  # Your deployed contract ID
   VITE_NETWORK_PASSPHRASE=Test SDF Network ; September 2015  # Testnet/mainnet
   VITE_RPC_URL=https://soroban-testnet.stellar.org      # Soroban RPC endpoint
   VITE_HORIZON_URL=https://horizon-testnet.stellar.org  # Horizon endpoint

   # Optional
   VITE_RAMP_API_KEY=rampnetwork          # For fiat on-ramp (staging)
   VITE_RAMP_ENVIRONMENT=staging           # staging or production
   VITE_SIMULATION_ACCOUNT=G...            # For read-only queries (defaults to contract ID)
   ```

3. Start the dev server:
   ```bash
   npm run dev
   ```

**Network configs:**

| Network | Passphrase | RPC URL | Horizon URL | Explorer |
|---------|-----------|---------|-------------|----------|
| Testnet | `Test SDF Network ; September 2015` | `https://soroban-testnet.stellar.org` | `https://horizon-testnet.stellar.org` | `https://stellar.expert/explorer/testnet` |
| Mainnet | `Public Global Stellar Network ; September 2015` | `https://soroban.stellar.org` | `https://horizon.stellar.org` | `https://stellar.expert/explorer/public` |

**Reference:** [Frontend README](./frontend/README.md#environment-variables)

---

## Usage & Basic Operations

### Q5: What is a deposit and how do I create one?

**A:** A **deposit** locks tokens in the contract until a future time. You specify:

- **Token**: Any SEP-41 token contract (e.g., USDC, native XLM wrapped)
- **Amount**: Number of tokens to lock (1 to 10^15 units)
- **Unlock Time**: Wall-clock timestamp (seconds since Unix epoch) when you can withdraw
- **Penalty (optional)**: Early-exit fee in basis points (0–10000 bps = 0–100%)

**Via Frontend:**
1. Connect your Freighter wallet
2. Go to **Deposit** tab
3. Select a token
4. Enter amount and unlock date
5. Set penalty (if desired)
6. Confirm and sign transaction

**Via CLI (using Stellar SDK):**
```bash
soroban contract invoke \
  --id C... \
  --source-account S... \
  --network testnet \
  -- \
  deposit \
  --depositor G... \
  --token C... \
  --amount 1000000000 \
  --unlock_time 1735689600 \
  --penalty_bps 500
```

The contract returns a **deposit ID** (u32) unique per depositor. You use this ID to withdraw later.

**Reference:** [README Contract API](./README.md#core-functions), [Frontend Deposit Page](./frontend/src/pages/DepositPage.tsx)

---

### Q6: How do I withdraw my tokens?

**A:**

**Normal withdrawal** (after unlock time):
1. Go to **Withdraw** tab
2. Select the deposit
3. Click "Withdraw"
4. Sign transaction

Your tokens are returned to your wallet.

**Early exit (before unlock time):**
1. Go to **Withdraw** tab
2. Select the deposit
3. Click "Cancel Early"
4. Confirm the penalty fee
5. Sign transaction

You receive: `amount × (1 - penalty_bps / 10000)`, e.g., if penalty is 20% (2000 bps):
- Locked amount: 1000
- Your refund: 1000 × (1 - 0.20) = **800**
- Penalty: **200** (30% to fee recipient, 70% to staker rewards pool)

**Via CLI:**
```bash
# Normal withdrawal
soroban contract invoke ... -- withdraw --depositor G... --deposit_id 1

# Early exit
soroban contract invoke ... -- cancel_deposit --depositor G... --deposit_id 1
```

**Reference:** [README Core Functions](./README.md#core-functions)

---

### Q7: What are staker rewards and how do I claim them?

**A:** 

When users cancel deposits early, the penalty is split:
- **30%** → Fee recipient
- **70%** → Staker rewards pool (accumulates)

**How to earn rewards:**

1. **Register as a staker** with a stake amount:
   ```bash
   soroban contract invoke ... -- register_staker --staker G... --amount 1000000000
   ```

2. Your proportional share = `your_stake / total_staked`

3. **Claim accumulated rewards:**
   ```bash
   soroban contract invoke ... -- claim_staker_rewards --staker G...
   ```

**Example:**
- Total staked: 10,000 tokens
- Your stake: 1,000 tokens (10%)
- Rewards pool: 700 tokens
- Your claim: 1000 / 10000 × 700 = **70 tokens**

**Reference:** [README Staker Registry](./README.md#staker-registry-functions)

---

### Q8: How do I use ledger-based deposits instead of timestamp-based ones?

**A:** 

**Timestamp-based deposits** (default) unlock at a specific wall-clock time:
```bash
soroban contract invoke ... -- deposit --unlock_time 1735689600 ...
```

**Ledger-based deposits** unlock when the blockchain reaches a specific ledger number. Use when you want to express locks in terms of on-chain block progression:

```bash
soroban contract invoke ... -- deposit_by_ledger --unlock_ledger 50000000 ...
```

The contract will release your funds when `current_ledger >= unlock_ledger`.

**Estimating wall-clock time:**
Stellar produces ~1 ledger per 5 seconds on average, so:
```
estimated_seconds = (unlock_ledger - current_ledger) × 5
```

**⚠️ Important caveats:**
- This estimate is **not exact** — actual ledger times vary ±1–2 seconds
- Use for UI display and rough scheduling only
- **Do not** rely on this for precise timing or critical deadlines
- The actual unlock check is exact: `current_ledger >= unlock_ledger`

**Current limitations:**
- No frontend UI support (CLI/SDK only)
- No maximum lock duration enforced (unlimited future ledgers accepted)
- `get_vault()` doesn't find ledger-based deposits; use `get_ledger_vault()` instead
- Not included in paginated `get_deposits_page()` results

**Reference:** [README Ledger-Based Deposits](./README.md#ledger-based-deposit-time-estimation-precision--confidence), [ADR-001](./docs/adr/ADR-001-dual-deposit-types.md)

---

### Q9: Can I transfer a deposit to someone else?

**A:** 

**Via deposit_for:** Deposit on behalf of someone else (you pay, they benefit):

```bash
soroban contract invoke ... -- deposit_for \
  --payer G... \       # Your wallet (signs the transaction)
  --depositor G2... \  # Beneficiary (receives tokens on withdraw)
  --token C... \
  --amount 1000 \
  --unlock_time 1735689600 \
  --penalty_bps 500
```

This is useful for:
- **Vesting**: Lock team tokens until a future date
- **Escrow**: Ensure payment only upon timelock expiry
- **Gift**: Lock funds for a friend

**Direct transfer:** The contract does not support transferring a deposit by ID. To move funds to a new deposit:
1. Withdraw the existing deposit
2. Create a new deposit in the beneficiary's name

**Reference:** [README deposit_for](./README.md#core-functions)

---

## Troubleshooting

### Q10: I'm getting "FundsStillLocked" error. What does it mean?

**A:** This error occurs when you try to withdraw before the unlock time/ledger has passed.

**Causes:**
- The current time is before the `unlock_time` you set
- For ledger-based deposits, the current ledger is before `unlock_ledger`

**Solutions:**
1. **Check current time:**
   ```bash
   soroban contract invoke ... -- get_time
   # Returns current ledger timestamp (seconds since Unix epoch)
   ```

2. **Check time remaining:**
   ```bash
   soroban contract invoke ... -- time_remaining --depositor G... --deposit_id 1
   # Returns seconds until unlocked
   ```

3. **If you need funds now, cancel early:**
   ```bash
   soroban contract invoke ... -- cancel_deposit --depositor G... --deposit_id 1
   # You'll receive amount minus penalty
   ```

**Reference:** [README Error Codes](./README.md#error-codes) (code 4)

---

### Q11: I set a penalty but it's not working. What's wrong?

**A:** 

**Common issues:**

1. **No fee_recipient configured:**
   - Penalties > 0 require a `fee_recipient` address
   - If not set, deposits with penalties are rejected: `MissingFeeRecipient`
   - Admin must run: `soroban contract invoke ... -- set_fee_recipient --fee_recipient G...`

2. **Penalty is 0 bps:**
   - A penalty of 0 means no early-exit fee
   - You'll get 100% of your funds back (minus gas fees)

3. **Penalty applied incorrectly:**
   - Formula: `refund = amount × (1 - penalty_bps / 10000)`
   - Example: 1000 tokens, 2000 bps (20%) → refund = 800
   - Check the transaction simulation result

**Verify penalty split:**
- 30% → fee_recipient
- 70% → staker rewards pool

**Reference:** [README Penalty Splitting](./README.md#penalty-splitting--rewards-pool)

---

### Q12: My wallet won't connect. What should I do?

**A:**

**Freighter integration issues:**

1. **Freighter extension not installed:**
   - Install from [freighter.app](https://freighter.app)
   - Refresh the page after installing

2. **Wrong network selected:**
   - Click the network badge (red = testnet, green = mainnet)
   - Select the network your contract is deployed to
   - Ensure your wallet account is funded on that network

3. **Account not funded:**
   - You need XLM for transaction fees
   - Use the **"Buy Tokens"** button in the header for testnet
   - Or get testnet XLM from the [Stellar Testnet Faucet](https://laboratory.stellar.org/#?network=test)

4. **Session expired:**
   - Disconnect and reconnect: click the wallet address → "Disconnect"
   - Then click "Connect Wallet" again

**If still stuck:**
- Check browser console for errors (F12 → Console tab)
- Try a different browser or incognito window
- Verify `.env` values match the network

**Reference:** [Frontend Wallet Integration](./frontend/src/context/WalletContext.tsx), [Frontend README Network Switching](./frontend/README.md#network-switching)

---

### Q13: Transaction simulation succeeded but submission failed. Why?

**A:** Stellar distinguishes between two phases:

1. **Simulation** — Dry-run against current ledger (free, read-only)
2. **Submission** — Broadcast signed tx (costs real fees, final)

**State can change between simulation and submission**, causing a mismatch.

**Common scenarios:**

1. **Another transaction withdrew your deposit:**
   - Simulation: deposit found ✓
   - Submission: deposit already withdrawn ✗
   - Error: `NoDepositFound`

2. **Insufficient wallet balance:**
   - Simulation: account had enough XLM
   - Submission: fees or transfers exhausted balance
   - Error: `InsufficientBalance`

3. **Network lag or double-submission:**
   - Your first tx lands just before submission
   - Second attempt finds a duplicate or state mismatch
   - Error: varies

**Solutions:**
1. Simulate again immediately before submitting
2. Verify your balance hasn't changed
3. Use a different RPC endpoint (less lag)
4. Wait for prior transactions to confirm (~5 seconds)
5. Check [Stellar Expert](https://stellar.expert) for your recent tx

**Reference:** [README Simulation vs Submission](./README.md#simulation-vs-submission)

---

### Q14: I got a "LockDurationTooLong" error. What's the maximum lock duration?

**A:** The maximum lock duration is **5 years** (157,680,000 seconds).

**Causes:**
- `unlock_time - now > 157,680,000 seconds`
- Lock duration exceeds 5 years

**Solutions:**
1. **Reduce the lock duration:**
   ```
   unlock_time = now + (60 × 60 × 24 × 365 × 5)  # 5 years max
   ```

2. **Create multiple deposits:**
   - Deposit 1: 5 years from now
   - Deposit 2: another 5 years
   - Total: 10 years of coverage via two contracts

**Reference:** [README Error Codes](./README.md#error-codes) (code 6), [README Constants](./README.md#overview)

---

### Q15: I forgot my password / lost my private keys. Can I recover my funds?

**A:** 

⚠️ **SAFE-HAVEN is non-custodial. There is no password or recovery service.**

**What you can do:**

1. **If you control the Freighter wallet:**
   - Use Freighter's built-in recovery (seed phrase)
   - This restores your account and all wallet access

2. **If you lost the wallet entirely:**
   - Only someone with the original seed phrase can recover it
   - Reach out to Freighter support if it was a security incident

3. **If funds are locked in SAFE-HAVEN:**
   - They stay locked until the `unlock_time`
   - After unlock, anyone with wallet access can withdraw
   - Early exit is available via `cancel_deposit` (with penalty)

4. **If you lost wallet + seed phrase + recovery codes:**
   - Funds are irretrievable until unlock time from that account
   - This is by design for trustlessness

**Community recovery contact:**
- The frontend offers an optional **recovery contact** feature (beta)
- You can designate a trusted person to help recover a backup of your wallet
- See [SettingsPage.tsx](./frontend/src/pages/SettingsPage.tsx) for details

**Reference:** [Security Education](./frontend/SECURITY_EDUCATION.md), [Frontend Recovery Feature](./frontend/src/pages/SettingsPage.tsx)

---

### Q16: Storage entry expired and I can't access my deposit. What happened?

**A:** 

Soroban uses **TTL (time-to-live)** for persistent storage. Entries expire after ~31.5 million ledgers (~5 years) of non-use.

**Why it happened:**
- Your deposit was created over 5 years ago
- No one accessed or modified it during that entire period
- Soroban pruned the expired entry

**Recovery:**

1. **Check if the entry is truly gone:**
   ```bash
   soroban contract invoke ... -- get_vault --depositor G... --deposit_id 1
   # Returns: None (entry expired)
   ```

2. **If lock time has passed:**
   - Re-create the deposit with the same funds
   - Or contact contract admin for evidence of the old deposit

3. **If lock time is still active:**
   - Escalate to the contract operator or security team
   - Preserve evidence: transaction hash, original deposit details
   - See [DISASTER_RECOVERY.md](./DISASTER_RECOVERY.md) for incident response

**Prevention:**
- Contract automatically extends TTL on every write and important read
- Accessing a deposit during withdrawal bumps TTL
- A 5-year inactive deposit is rare in practice

**Reference:** [README Ledger TTL](./README.md#ledger-ttl-and-storage-expiry)

---

## Advanced Topics

### Q17: How do I register for the staker registry?

**A:**

**Purpose:** Registered stakers earn a share of penalties from early deposit exits.

**Registration:**
```bash
soroban contract invoke \
  --id C... \
  --source-account S... \
  --network testnet \
  -- \
  register_staker \
  --staker G... \
  --amount 1000000000   # Stake amount (must be > 0)
```

**What happens:**
1. Your `amount` is recorded (you don't transfer tokens; this is just a registry entry)
2. You're added to the staker list
3. Your proportional share = `your_amount / total_staked`
4. Whenever penalties accumulate (via `cancel_deposit`), 70% goes to the rewards pool
5. You can claim your share anytime via `claim_staker_rewards`

**Updating your stake:**
- Call `register_staker` again with a new amount
- Your previous stake is overwritten

**Claiming rewards:**
```bash
soroban contract invoke ... -- claim_staker_rewards --staker G...
```

**Example:**
```
Total staked: 100,000 tokens
Your stake:    10,000 tokens (10%)
Rewards pool:   5,000 tokens

Your claim: 10,000 / 100,000 × 5,000 = 500 tokens
```

**Reference:** [README Staker Registry](./README.md#staker-registry-functions), [STAKER_REGISTRY_IMPLEMENTATION.md](./STAKER_REGISTRY_IMPLEMENTATION.md)

---

### Q18: I'm an admin. How do I pause/unpause the contract?

**A:**

**Pause** (block new deposits):
```bash
soroban contract invoke ... -- pause --admin G...
```

**Unpause** (re-enable deposits):
```bash
soroban contract invoke ... -- unpause --admin G...
```

**Why pause?**
- Emergency incident or vulnerability discovered
- Planned maintenance
- Network migration
- Existing deposits remain locked; withdrawals still work

**Verify pause state:**
```bash
soroban contract invoke ... -- is_paused
# Returns: true or false
```

**Reference:** [README Admin Functions](./README.md#admin-functions)

---

### Q19: How do I transfer admin rights safely?

**A:**

**Step 1: Propose new admin:**
```bash
soroban contract invoke ... -- transfer_admin --admin G1... --new_admin G2...
```

**Step 2: New admin accepts:**
```bash
soroban contract invoke ... -- accept_admin --admin G2...
```

**Why two-step?**
- Prevents accidental loss of control (typos)
- New admin confirms they control the key
- Original admin can cancel if something goes wrong

**Cancel a pending transfer:**
```bash
soroban contract invoke ... -- cancel_transfer_admin --admin G1...
```

**Check pending admin:**
```bash
soroban contract invoke ... -- get_pending_admin
# Returns: None or the pending address
```

**Reference:** [README Admin Functions](./README.md#admin-functions)

---

### Q20: How do I permanently renounce admin and make the contract trustless?

**A:**

Once you call `renounce_admin`, the contract becomes **fully decentralized** — no admin can ever unpause, emergency-withdraw, or change settings.

```bash
soroban contract invoke ... -- renounce_admin --admin G...
```

**After renouncement:**
- `get_admin()` returns `None`
- No further admin functions are callable
- Withdrawals, cancellations, and staker rewards work normally
- The contract is **immutable** (no upgrades possible in Soroban)

**Use cases:**
- Prove the contract is trustless
- Become eligible for certain DeFi protocols
- Signal long-term commitment to decentralization

⚠️ **Irreversible.** Once renounced, you cannot restore admin rights.

**Reference:** [README Admin Functions](./README.md#admin-functions), [SECURITY.md](./SECURITY.md)

---

### Q21: What is the `deposit_by_ledger` function and when should I use it?

**A:**

**`deposit_by_ledger`** is like `deposit`, but you specify an unlock ledger number instead of a timestamp:

```bash
soroban contract invoke ... -- deposit_by_ledger \
  --depositor G... \
  --token C... \
  --amount 1000 \
  --unlock_ledger 50000000 \  # Release when chain reaches ledger 50M
  --penalty_bps 500
```

**When to use:**
- You want to lock for N ledgers (e.g., "100 ledgers from now")
- You prefer on-chain block progression over wall-clock time
- You want to avoid timezone/timestamp ambiguity

**When NOT to use:**
- You need precise wall-clock timing → use `deposit()` instead
- You're unsure about ledger timing → use `deposit()` instead

**Estimating time:**
```
seconds ≈ (unlock_ledger - current_ledger) × 5
```

**Example:**
- Current ledger: 49,950,000
- Unlock ledger: 50,000,000
- Ledgers to wait: 50,000
- Estimated time: 50,000 × 5 = 250,000 seconds ≈ 2.9 days

**⚠️ Caveats:**
- The 5-second estimate is not exact (±1–2 seconds possible)
- There is no frontend UI support (CLI/SDK only)
- `get_vault()` doesn't find these; use `get_ledger_vault()` instead
- No maximum duration enforced (unlike `deposit()`)

**Reference:** [README deposit_by_ledger](./README.md#deposit_by_ledger), [ADR-001](./docs/adr/ADR-001-dual-deposit-types.md)

---

### Q22: I want to monitor contract health. What should I track?

**A:**

**Key metrics to monitor:**

1. **Contract state:**
   - `is_initialized()` — should be `true`
   - `is_paused()` — should normally be `false`
   - `get_admin()` — verify matches approved address
   - `get_pending_admin()` — should be `None` unless transferring

2. **Balances & solvency:**
   - Admin account XLM balance (pay for fees)
   - Contract token balance (per supported token)
   - Active deposit total (reconcile against balance)
   - Alert if balance < deposit total (indicates a problem)

3. **Storage & TTL:**
   - Total active depositors
   - Total active deposits
   - Storage entry expiration ledger (ensure > lock duration)
   - Alert if entries expire before unlock time

4. **Events & transactions:**
   - Subscribe to `deposit`, `withdraw`, `cancel_deposit`, `emergency_withdraw` events
   - Monitor for failed transactions
   - Track fee consumption over time

5. **Synthetic probes:**
   - Every 15 minutes: create, query, and withdraw a test deposit
   - Ensures the contract is responsive
   - Use a dedicated test account (never production funds)

**Monitoring tools:**
- [MONITORING.md](./MONITORING.md) — detailed guide with alert thresholds
- [stellar.expert](https://stellar.expert) — public explorer
- Custom scripts using Soroban RPC

**Reference:** [MONITORING.md](./MONITORING.md), [Soroban RPC Documentation](https://developers.stellar.org/docs/build/guides/soroban-rpc)

---

### Q23: How do I recover from a contract vulnerability or exploit?

**A:**

**Immediate steps:**
1. Declare incident (SEV-1 if active loss)
2. Pause the contract: `pause(admin)`
3. Preserve evidence: tx hashes, events, balances
4. Do NOT call vulnerable functions again

**Recovery path:**

1. **If safe to recover:**
   - Identify affected deposits
   - Use `emergency_withdraw(admin, depositor, deposit_id)` for each
   - Refunds go to depositor (never admin)

2. **If unsafe or admin compromised:**
   - Build and deploy a replacement contract
   - Use `emergency_withdraw` to migrate funds to new contract
   - Update frontend contract ID
   - Publish migration instructions

3. **Long-term:**
   - Add regression tests
   - Conduct security audit
   - Deploy reviewed replacement
   - Unpause only after sign-off from security lead

**Reference:** [DISASTER_RECOVERY.md](./DISASTER_RECOVERY.md) (Scenario A)

---

### Q24: What happens if I encounter a data mismatch or missing deposit?

**A:**

**Triage:**
1. Check both `get_vault()` (timestamp) and `get_ledger_vault()` (ledger-based)
2. Verify against contract events and transaction history
3. Query multiple RPC endpoints
4. Check if storage entry expired (see Q16)

**If frontend shows deposit but contract doesn't:**
- Likely a cache or indexer issue
- Try different RPC endpoint
- Clear browser cache
- Refresh page

**If contract has deposit but it's inaccessible:**
- Check `get_deposit_ids(depositor)` to confirm ID exists
- Try querying with Stellar CLI
- Preserve transaction hash and ledger number
- Escalate to operator

**If balance mismatch (contract tokens < deposits):**
- SEV-2 incident — reconciliation required
- Preserve all events and transaction history
- See [DISASTER_RECOVERY.md](./DISASTER_RECOVERY.md) (Scenario B)

**Reference:** [DISASTER_RECOVERY.md](./DISASTER_RECOVERY.md) (Scenario B), [MONITORING.md](./MONITORING.md)

---

### Q25: How do I optimize gas costs and reduce transaction fees?

**A:**

**Frontend-level optimizations:**

1. **Batch read operations:**
   - Use `get_deposits_page(offset, limit)` instead of querying each deposit individually
   - Limit pages to 50 deposits per query
   - Use `get_depositors(offset, limit)` to enumerate accounts efficiently

2. **Batch contract queries:**
   - SDK offers `get_vault_batch()` and `get_deposit_batch()` for up to 25 items
   - More efficient than N sequential calls

3. **One transaction = one operation:**
   - Combine multiple deposit operations into one tx if possible
   - But avoid exceeding instruction budget (100M on testnet, 50M on mainnet)

**Contract-level optimizations:**

1. **Minimal call overhead:**
   - Soroban SDK auto-optimizes function calls
   - Keep payloads small (avoid large strings/arrays)

2. **Storage access:**
   - Contract batches reads into single ledger entry when possible
   - Frequent small writes are more expensive than one large write

**Monitoring fee usage:**
```bash
# Simulate to see exact fee estimate
soroban contract invoke ... --simulate
# Simulator output includes fee_charged estimate
```

**Reference:** [README Instruction Budget Limits](./README.md#instruction-budget-limits), [GAS_PROFILING.md](./GAS_PROFILING.md)

---

## Community Contributions

### Q26: How can I contribute FAQ questions and improvements?

**A:**

**Process:**

1. **Identify a missing question:**
   - Review existing FAQ
   - Check [Discord](https://discord.gg/yourserver) and GitHub issues for common questions
   - Note the category (Setup, Usage, Troubleshooting, Advanced)

2. **Propose the question:**
   - Open a GitHub issue with label `documentation: faq`
   - Title: `FAQ: [Question about topic]`
   - Provide:
     - The question you want to add
     - A draft answer (if you have one)
     - Why you think it's important
     - Category (Setup, Usage, Troubleshooting, Advanced)

3. **Draft your contribution:**
   - Fork the repository
   - Edit [FAQ.md](./FAQ.md) (this file)
   - Add your Q&A in the appropriate section
   - Use the existing format (Q##, A##, code examples, references)
   - Test all code examples

4. **Create a pull request:**
   - Push to your fork
   - Open a PR against `main` branch
   - Reference the issue (e.g., `Closes #42`)
   - Include a brief summary of what you added

5. **Feedback & merge:**
   - Maintainers review for accuracy and clarity
   - Address any feedback
   - Merge once approved

**Guidelines:**

- **Accuracy:** Test code examples. Link to relevant docs.
- **Clarity:** Use simple language. Assume readers are non-experts in Soroban.
- **Completeness:** Explain the "why" and the "how."
- **Examples:** Include CLI and frontend examples where applicable.
- **Links:** Cross-reference related FAQs and documentation.

**Reference:** [CONTRIBUTING.md](./CONTRIBUTING.md)

---

### Q27: How do I report an issue with the FAQ or documentation?

**A:**

**For errors or outdated info:**
1. Open a GitHub issue with label `documentation`
2. Include:
   - Which FAQ question/section is wrong
   - What the issue is
   - What you expected
   - Suggested fix (if you have one)

**For missing information:**
1. Open an issue with label `documentation: faq`
2. Suggest:
   - The question you think should be in the FAQ
   - Why it's important
   - Relevant context or Discord conversation

**For typos or formatting:**
1. You can create a quick PR directly
2. Or open an issue (we'll fix it)

**Reference:** [GitHub Issues](https://github.com/kenedybok3/SAFE-HAVEN/issues), [CONTRIBUTING.md](./CONTRIBUTING.md)

---

### Q28: Where can I ask questions not covered by the FAQ?

**A:**

1. **GitHub Discussions:** Post questions in [Discussions](https://github.com/kenedybok3/SAFE-HAVEN/discussions)
2. **Discord:** Join the community [Discord server](https://discord.gg/yourserver) and post in #support or #general
3. **Security issues:** Email [security@safe-haven.dev](mailto:security@safe-haven.dev) (see [SECURITY.md](./SECURITY.md))
4. **Contract/protocol questions:** Check [README.md](./README.md) and [docs/](./docs/)

**Reference:** [CONTRIBUTING.md](./CONTRIBUTING.md), [SECURITY.md](./SECURITY.md)

---

## Quick Reference Table

| Question | Link |
|----------|------|
| What are the prerequisites? | [Q1](#q1-what-are-the-prerequisites-for-running-safe-haven) |
| How do I set up locally? | [Q2](#q2-how-do-i-set-up-a-local-development-environment) |
| How do I deploy? | [Q3](#q3-how-do-i-deploy-to-testnet-or-mainnet) |
| How do I configure the frontend? | [Q4](#q4-how-do-i-configure-the-frontend-for-my-contract) |
| How do I create a deposit? | [Q5](#q5-what-is-a-deposit-and-how-do-i-create-one) |
| How do I withdraw? | [Q6](#q6-how-do-i-withdraw-my-tokens) |
| What are staker rewards? | [Q7](#q7-what-are-staker-rewards-and-how-do-i-claim-them) |
| What are ledger-based deposits? | [Q8](#q8-how-do-i-use-ledger-based-deposits-instead-of-timestamp-based-ones) |
| Can I transfer a deposit? | [Q9](#q9-can-i-transfer-a-deposit-to-someone-else) |
| "FundsStillLocked" error | [Q10](#q10-im-getting-fundsstilllocked-error-what-does-it-mean) |
| Penalty not working | [Q11](#q11-i-set-a-penalty-but-its-not-working-whats-wrong) |
| Wallet won't connect | [Q12](#q12-my-wallet-wont-connect-what-should-i-do) |
| Simulation succeeded but submission failed | [Q13](#q13-transaction-simulation-succeeded-but-submission-failed-why) |
| "LockDurationTooLong" error | [Q14](#q14-i-got-a-lockdurationtoolong-error-whats-the-maximum-lock-duration) |
| Lost private keys | [Q15](#q15-i-forgot-my-password--lost-my-private-keys-can-i-recover-my-funds) |
| Storage entry expired | [Q16](#q16-storage-entry-expired-and-i-cant-access-my-deposit-what-happened) |
| Register as staker | [Q17](#q17-how-do-i-register-for-the-staker-registry) |
| Admin: pause/unpause | [Q18](#q18-im-an-admin-how-do-i-pauseunpause-the-contract) |
| Admin: transfer rights | [Q19](#q19-how-do-i-transfer-admin-rights-safely) |
| Admin: renounce rights | [Q20](#q20-how-do-i-permanently-renounce-admin-and-make-the-contract-trustless) |
| Ledger-based deposits | [Q21](#q21-what-is-the-deposit_by_ledger-function-and-when-should-i-use-it) |
| Monitor contract health | [Q22](#q22-i-want-to-monitor-contract-health-what-should-i-track) |
| Recover from vulnerability | [Q23](#q23-how-do-i-recover-from-a-contract-vulnerability-or-exploit) |
| Data mismatch or missing deposit | [Q24](#q24-what-happens-if-i-encounter-a-data-mismatch-or-missing-deposit) |
| Optimize gas costs | [Q25](#q25-how-do-i-optimize-gas-costs-and-reduce-transaction-fees) |
| Contribute FAQ | [Q26](#q26-how-can-i-contribute-faq-questions-and-improvements) |
| Report FAQ issue | [Q27](#q27-how-do-i-report-an-issue-with-the-faq-or-documentation) |
| Ask questions | [Q28](#q28-where-can-i-ask-questions-not-covered-by-the-faq) |

---

## Feedback

Have a question that isn't answered here? Please:
- Open an issue on [GitHub](https://github.com/kenedybok3/SAFE-HAVEN/issues)
- Ask on [Discord](https://discord.gg/yourserver)
- Email [support@safe-haven.dev](mailto:support@safe-haven.dev)

Last updated: August 28, 2026
