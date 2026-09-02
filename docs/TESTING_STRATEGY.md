# Testing Strategy and Guidelines

**Document status:** Active  
**Applies to:** All contributors to SAFE-HAVEN  
**Last reviewed:** 2026-09-02

---

## Table of Contents

1. [Testing Philosophy](#1-testing-philosophy)
2. [Testing Pyramid](#2-testing-pyramid)
3. [Smart Contract Tests (Rust)](#3-smart-contract-tests-rust)
   - [Unit Test Guidelines](#31-unit-test-guidelines)
   - [Coverage Requirements](#32-coverage-requirements)
   - [Test Data Management](#33-test-data-management)
4. [Frontend Tests (TypeScript)](#4-frontend-tests-typescript)
   - [Unit and Integration Tests](#41-unit-and-integration-tests)
   - [End-to-End Tests](#42-end-to-end-tests)
5. [Integration Test Scenarios](#5-integration-test-scenarios)
6. [Performance Testing](#6-performance-testing)
7. [Security Testing](#7-security-testing)
8. [Test Environment Requirements](#8-test-environment-requirements)
9. [Coverage Tracking](#9-coverage-tracking)
10. [Best Practices](#10-best-practices)

---

## 1. Testing Philosophy

Testing in SAFE-HAVEN protects real user funds. An undetected bug in the
contract can result in funds being permanently locked or stolen — consequences
that are irreversible on-chain. Test coverage is therefore a first-class
requirement, not an afterthought.

**Core principles:**

- Every code change that modifies contract logic requires at least one new test.
- A PR that decreases coverage requires a written justification in the PR description.
- Tests must be deterministic. Flaky tests are treated as bugs.
- Tests document intent. A test name should read like a specification sentence.
- Security-sensitive paths (auth, penalty calculation, admin transfer) require
  both positive (happy-path) and negative (adversarial) test cases.

---

## 2. Testing Pyramid

SAFE-HAVEN follows the standard testing pyramid with ratios calibrated for a
smart-contract project:

```
              ┌─────────────┐
              │    E2E       │  ~10%
              │  (Playwright)│
             ─┴─────────────┴─
           ┌───────────────────┐
           │   Integration      │  ~20%
           │  (cross-layer      │
           │   flows + RPC)     │
          ─┴───────────────────┴─
       ┌──────────────────────────┐
       │       Unit Tests          │  ~70%
       │  (contract logic +        │
       │   frontend utilities)     │
      ─┴──────────────────────────┴─
```

| Layer | Target | Runner | Where |
|---|---|---|---|
| Unit | 70% of total tests | `cargo test` / `vitest` | CI on every PR |
| Integration | 20% of total tests | `cargo test` + smoke tests | CI on every PR |
| E2E | 10% of total tests | Playwright | CI on PRs to `main`/`develop` |

### Why this ratio?

Contract unit tests run against an in-process `Env` (Soroban testutils) and
execute in milliseconds with full control over ledger state. They are cheap
and should be the primary safety net. Integration and E2E tests are slower
and require more setup; they validate the system behaves correctly across
boundaries, not every edge case.

---

## 3. Smart Contract Tests (Rust)

### 3.1 Unit Test Guidelines

All contract tests live in `contracts/safe-haven/src/test.rs` and use the
`soroban-sdk` testutils environment.

#### Setup helpers

Always use the project's shared helpers:

```rust
// Standard setup: deploys contract, sets up admin/alice/fee_recipient/token
fn setup() -> (Env, SafeHavenClient, Address, Address, Address, Address);

// Setup with configurable limits (max_deposit, max_lock_secs)
fn setup_with_limits(max_deposit: Option<i128>, max_lock_secs: Option<u64>)
    -> (Env, SafeHavenClient, Address, Address, Address, Address);

// Advance ledger time by N seconds
fn advance_time(env: &Env, seconds: u64);
```

Never replicate setup boilerplate inline. If the shared helpers do not cover
your scenario, extend them or add a new focused helper.

#### Test naming

Test names must describe the scenario and expected outcome:

```
test_<function>_<scenario>_<outcome>

// Good
test_deposit_zero_amount_returns_invalid_amount_error
test_withdraw_before_unlock_time_returns_funds_still_locked
test_transfer_admin_same_address_returns_invalid_admin_error

// Bad
test_deposit_fail
test_withdraw_1
test_admin
```

#### Test structure (Arrange-Act-Assert)

```rust
#[test]
fn test_cancel_deposit_with_penalty_splits_to_fee_and_staker_pool() {
    // Arrange
    let (env, vault, _admin, alice, fee_recipient, token) = setup();
    let token_client = TokenClient::new(&env, &token);
    let unlock = env.ledger().timestamp() + 3600;
    let deposit_id = vault.deposit(&alice, &token, &1000, &unlock, &500); // 5% penalty

    // Act
    vault.cancel_deposit(&alice, &deposit_id);

    // Assert
    // fee_recipient gets 30% of penalty (1000 * 500/10000 * 30% = 15)
    assert_eq!(token_client.balance(&fee_recipient), 15);
    // remaining 70% stays in rewards pool
    // vault no longer holds alice's deposit
    assert!(vault.get_vault(&alice, &deposit_id).is_none());
}
```

#### Coverage categories

Every function in `contract.rs` must have tests covering:

| Category | Description |
|---|---|
| **Happy path** | Normal inputs, expected outputs |
| **Boundary values** | Min/max amounts, exact unlock timestamps |
| **Error cases** | Every `VaultError` variant the function can return |
| **Auth** | Unauthorized callers are rejected |
| **State isolation** | Operations on one deposit don't affect others |

#### Auth tests

Every mutating function must have an explicit auth-rejection test:

```rust
#[test]
fn test_deposit_requires_depositor_auth() {
    let (env, vault, _admin, alice, _fee, token) = setup();
    env.mock_auths(&[]); // clear mock_all_auths
    let result = vault.try_deposit(&alice, &token, &1000,
        &(env.ledger().timestamp() + 3600), &0);
    assert!(result.is_err());
}
```

#### Ledger-based deposit tests

`deposit_by_ledger` tests must use ledger sequence values rather than
timestamps. Use the `env.ledger().set()` API to set a specific sequence:

```rust
fn advance_ledger(env: &Env, ledgers: u32) {
    env.ledger().set(LedgerInfo {
        sequence_number: env.ledger().sequence() + ledgers,
        ..env.ledger().get()
    });
}
```

### 3.2 Coverage Requirements

| Target | Requirement |
|---|---|
| Overall contract coverage | ≥ 80% |
| `contract.rs` function coverage | 100% (every public fn tested) |
| `errors.rs` variant coverage | 100% (every error variant triggered) |
| Security-critical paths | 100% (see below) |

**Security-critical paths requiring 100% coverage:**

- `require_auth()` calls in all mutating functions
- `cancel_deposit` penalty calculation and split
- `transfer_admin` → `accept_admin` two-step flow
- `emergency_withdraw` funds-go-to-depositor invariant
- `renounce_admin` permanent removal
- `pause` / `unpause` deposit gating

#### Measuring coverage

Use `cargo-tarpaulin` for Rust coverage measurement:

```bash
# Install (one-time)
cargo install cargo-tarpaulin

# Run with coverage report
cargo tarpaulin --features testutils --out Html --output-dir coverage/

# View report
open coverage/tarpaulin-report.html
```

CI does not yet enforce the 80% threshold automatically. Until it does, PRs
that touch contract logic must include the tarpaulin summary output in the PR
description if any file's coverage drops.

Track the target in Makefile for convenience:

```bash
make coverage     # runs tarpaulin and opens the report
```

### 3.3 Test Data Management

Soroban tests use in-process state that is fully isolated per `Env` instance.
There is no shared database or persistent state between tests.

**Rules:**

- Each test must create its own `Env` instance via `setup()`.
- Do not share `Env` between tests. Soroban's testutils are not designed for it.
- Token amounts in tests use small values (`1_000`, `10_000`) to make arithmetic
  assertions readable. Do not use production-scale amounts (`10^14`) unless
  testing boundary conditions.
- Addresses must be generated with `Address::generate(&env)` — never hardcode
  address strings in tests.
- Timestamps must be relative to `env.ledger().timestamp()` — never use
  absolute Unix timestamps that will become stale.

**Standard test values:**

| Variable | Value | Rationale |
|---|---|---|
| Alice's initial balance | `10_000` | Generous enough for multiple deposit tests |
| Default deposit amount | `1_000` | Clean number for penalty arithmetic |
| Default lock duration | `3600` (1 hour) | Short enough to advance easily |
| Default penalty | `0` or `500` (5%) | Zero for simple tests; 500 bps for penalty tests |

---

## 4. Frontend Tests (TypeScript)

### 4.1 Unit and Integration Tests

Frontend unit and integration tests use **Vitest** with `happy-dom`.

#### What to test with Vitest

- Utility functions in `src/lib/` (formatting, validation, Stellar SDK wrappers)
- Custom hooks in `src/hooks/` (mock the Stellar SDK client)
- Component rendering and interaction via `@testing-library/react`
- Error boundary behavior

#### Test file location

Co-locate test files with source:

```
src/lib/formatting.ts
src/lib/formatting.test.ts     ← preferred

src/hooks/useDeposits.ts
src/hooks/useDeposits.test.ts
```

Alternatively, use `src/__tests__/` for tests that span multiple modules.

#### Running frontend unit tests

```bash
cd frontend
npm run test:watch        # watch mode (development)
npx vitest run            # single run (CI)
```

#### Coverage for frontend

```bash
npx vitest run --coverage
```

Target: **≥ 80%** for `src/lib/` and `src/hooks/`. Components are lower
priority; aim for ≥ 60% on components given the DOM-mocking overhead.

#### Mocking the Stellar SDK

The Stellar SDK and Freighter wallet are not testable in happy-dom. Mock them
at the module level:

```typescript
vi.mock('../lib/stellar', () => ({
  simulateAndSubmit: vi.fn().mockResolvedValue({ result: 'ok' }),
  getContractInfo: vi.fn().mockResolvedValue({ version: '1.0.0' }),
}))
```

Never make real RPC calls in unit tests. Tests that require real network
access belong in the E2E layer.

### 4.2 End-to-End Tests

E2E tests use **Playwright** targeting Chromium. They validate critical user
journeys against a running frontend connected to a testnet or local node.

#### Critical paths (must have E2E coverage)

1. **Wallet connection** — connect Freighter, display wallet address
2. **Deposit flow** — enter amount, set unlock time, submit, verify deposit appears
3. **Withdraw flow** — advance time (local only), initiate withdraw, verify balance
4. **Early cancel** — cancel a locked deposit, verify penalty deducted
5. **Admin emergency withdraw** — admin bypasses lock, funds return to depositor
6. **Pause / unpause** — deposits rejected while paused, accepted after unpause

#### Running E2E tests

```bash
cd frontend

# Against local dev server (requires local Stellar node)
npm run test           # playwright test

# With UI explorer
npm run test:ui

# Against testnet (set VITE_CONTRACT_ID in .env first)
PLAYWRIGHT_BASE_URL=http://localhost:5173 npm run test
```

#### E2E test data management

- Use the testnet faucet or local funded keypairs — never real mainnet funds.
- Tests must clean up after themselves where possible (cancel deposits after testing).
- Do not hardcode keypair secrets in test files; read from environment variables.
- The smoke test in `frontend/tests/smoke.spec.ts` serves as the canonical E2E
  baseline; extend it rather than creating a parallel suite.

---

## 5. Integration Test Scenarios

Integration tests validate that multiple contract functions work correctly in
sequence. These live in `test.rs` and use the same Soroban testutils environment.

### Required integration scenarios

| Scenario | Functions exercised |
|---|---|
| Full deposit lifecycle | `deposit` → `time_remaining` → `withdraw` |
| Early exit lifecycle | `deposit` → `cancel_deposit` → staker reward claim |
| Third-party deposit | `deposit_for` → beneficiary `withdraw` |
| Admin transfer lifecycle | `transfer_admin` → `accept_admin` → old admin rejected |
| Trustless conversion | `renounce_admin` → `emergency_withdraw` rejected |
| Pause gate | `pause` → `deposit` rejected → `unpause` → `deposit` accepted |
| Re-deposit | `deposit` → `withdraw` → `deposit` (same depositor, new ID) |
| Pagination | Deposit 30 times → `get_depositors(0, 10)` → `get_depositors(10, 10)` etc. |
| Batch query | `get_deposit_batch` with 25 IDs → clamped at `MAX_BATCH_SIZE` |
| Staker rewards | `register_staker` → `cancel_deposit` (penalty) → `claim_staker_rewards` |
| Ledger-based deposit | `deposit_by_ledger` → advance sequence → `withdraw` |

### Cross-layer integration

Test that the frontend correctly interprets contract error codes:

```typescript
// In a Vitest integration test
it('displays FundsStillLocked error to user', async () => {
  const mockError = new VaultError(4) // FundsStillLocked
  mockSimulateAndSubmit.mockRejectedValueOnce(mockError)
  // ... render component, attempt withdraw, assert error message shown
})
```

---

## 6. Performance Testing

### Contract performance (instruction budget)

Every Soroban transaction has a CPU instruction budget (~50M on mainnet).
Functions that iterate over collections are budget-sensitive.

**Manual performance testing steps:**

1. Deploy to testnet.
2. Create a large number of deposits (50+) for a single depositor.
3. Call `get_depositors(0, 50)` and `get_deposit_batch` with 25 IDs.
4. Observe the simulated instruction cost in the stellar-cli output.
5. Ensure no call exceeds 40M instructions (80% of mainnet budget as safety margin).

**Benchmark targets:**

| Operation | Instruction budget limit |
|---|---|
| `deposit` | < 5M instructions |
| `withdraw` | < 5M instructions |
| `cancel_deposit` | < 8M instructions |
| `get_deposit_batch(25 IDs)` | < 15M instructions |
| `get_depositors(offset=0, limit=50)` | < 20M instructions |

### Frontend performance

| Scenario | Target |
|---|---|
| Dashboard load with 20 deposits | < 2s |
| Dashboard load with 100 deposits | < 4s |
| Initial page load (no wallet) | < 1s |

Use Playwright's `page.metrics()` and browser devtools Network tab to measure
frontend performance. The `TESTING_GUIDE.md` file has detailed instructions
for measuring RPC call reduction.

---

## 7. Security Testing

Security testing is a mandatory part of every release cycle, not optional.

### Static analysis (automated, every PR)

| Tool | What it checks | Where it runs |
|---|---|---|
| `cargo clippy` | Rust lints including potential bugs | CI `lint` job |
| `cargo audit` | Known CVEs in dependencies | CI `security-audit` job |
| `cargo deny` | License compliance + banned crates | CI `deny` job |
| `cargo geiger` | Unsafe Rust code blocks | CI `geiger` job |
| `eslint` | TypeScript linting | CI `frontend` job |
| `tsc --noEmit` | TypeScript type safety | CI `frontend` job |

### Manual security review (every PR touching contract logic)

Every PR that modifies `contract.rs` or `storage.rs` must be reviewed with the
following checklist as part of the code review:

- [ ] `require_auth()` is the **first** call in every new mutating function
- [ ] All storage writes clear state before any token transfer (CEI pattern)
- [ ] No unbounded loops over user-controlled collections
- [ ] New error codes do not reuse existing error code numbers
- [ ] Penalty calculations use integer arithmetic without overflow risk
- [ ] Any new `i128` arithmetic uses `checked_*` or `saturating_*` operations

### Adversarial test cases

The following attack vectors must have explicit test cases:

| Attack | Test |
|---|---|
| Admin impersonation | Non-admin calling `emergency_withdraw` → `Unauthorized` |
| Re-entrancy via token callback | State cleared before `token.transfer()` in all paths |
| Integer overflow in penalty | Max `amount` × max `penalty_bps` does not overflow i128 |
| Double-spend via concurrent cancel | Cancel after unlock returns `VaultAlreadyUnlocked` |
| Front-running on admin transfer | `accept_admin` with wrong caller fails |
| Unauthorized pause | Non-admin calling `pause` → `Unauthorized` |

### Periodic security tasks

| Cadence | Task |
|---|---|
| Every release | Run `cargo audit` and `cargo deny check` manually |
| Every release | Review all changed public contract functions against security checklist |
| Quarterly | Review SECURITY.md and update supported versions table |
| Annually | Commission external security audit (see SECURITY.md) |

---

## 8. Test Environment Requirements

### Local development

| Requirement | Version / Notes |
|---|---|
| Rust | 1.81+ (see `rust-toolchain.toml`) |
| `wasm32-unknown-unknown` target | `rustup target add wasm32-unknown-unknown` |
| `cargo test` with `testutils` feature | `cargo test --features testutils` |
| Node.js | 20 LTS |
| npm | bundled with Node.js 20 |
| Playwright browsers | `npx playwright install chromium` |

Install all Rust dev tools in one command:

```bash
make install-tools
```

### CI environment

CI runs on `ubuntu-latest`. Environment requirements are managed by the CI
workflow files (`.github/workflows/ci.yml`). All required tools are installed
within the jobs; contributors do not need to match the exact CI environment
locally, but the local environment must pass `make check` before pushing.

### Local Stellar node (for smoke tests and E2E)

```bash
# Start local Stellar node
stellar network start local --background

# Full local dev setup (build + deploy + frontend)
make dev

# Run smoke tests against local node
make smoke-test-local
```

### Testnet

- Use the [Stellar testnet faucet](https://friendbot.stellar.org/) to fund accounts.
- Never use mainnet keys or real funds for testing.
- Testnet contract IDs change with every deployment; update `frontend/.env` after redeployment.
- See `FAUCET.md` for testnet funding instructions.

---

## 9. Coverage Tracking

### Current state

Coverage is not yet enforced automatically in CI. The 80% target is a
contributor requirement tracked via PR review.

### Adding coverage enforcement to CI

The following CI step should be added to `ci.yml` when `cargo-tarpaulin` is
stable enough for the CI environment:

```yaml
- name: Check test coverage
  run: |
    cargo install cargo-tarpaulin --locked
    cargo tarpaulin --features testutils --fail-under 80 --out Xml
  # fail-under 80 exits non-zero if coverage < 80%
```

### Coverage reporting

Until CI enforcement is in place:

1. Run `cargo tarpaulin --features testutils --out Html` locally.
2. Include the total coverage percentage in your PR description when modifying
   contract logic.
3. If coverage drops below 80%, add tests before requesting review.

### Frontend coverage

```bash
cd frontend
npx vitest run --coverage --reporter=text-summary
```

Include the summary in your PR description when modifying `src/lib/` or `src/hooks/`.

---

## 10. Best Practices

### Do

- Write tests before or alongside code (test-driven where practical).
- Keep each test focused on a single behavior.
- Use descriptive assertion messages: `assert_eq!(actual, expected, "message if it fails")`.
- Group related tests with a descriptive comment block.
- Run `make check` before pushing (format + lint + test + audit + deny).
- Add a test for every bug you fix — prevents regression.

### Don't

- Don't use `#[ignore]` to silence a failing test. Fix it or delete it.
- Don't use `unwrap()` in test helpers without a comment explaining why it won't panic.
- Don't depend on test execution order. Each test must be fully self-contained.
- Don't test implementation details (private functions). Test behavior through
  the public interface.
- Don't leave TODO comments in test files. Open a GitHub issue instead.
- Don't skip E2E tests with `test.skip` in a PR targeting `main`. Fix the test
  or document the skip with a linked issue.

### Test review checklist

When reviewing a PR that includes tests, verify:

- [ ] Tests are named following the `test_<function>_<scenario>_<outcome>` convention
- [ ] Arrange-Act-Assert structure is clear (use blank lines between sections)
- [ ] Every error path in new code has a corresponding test
- [ ] `require_auth()` is tested for every new mutating function
- [ ] No real network calls in unit/integration tests
- [ ] Test data follows the conventions in [§3.3](#33-test-data-management)
- [ ] Coverage does not drop below 80%

---

*For questions or to propose changes to this document, open an issue or PR.*
*This document is owned by the maintainers and reviewed with every major release.*
