## Summary

<!-- Required: Replace this text with a description of what this PR does and why.
     Minimum 50 non-whitespace characters or the doc-lint CI check will fail. -->

Closes #<!-- issue number -->

---

## Type of Change

<!-- Check all that apply -->
- [ ] `feat` — New feature
- [ ] `fix` — Bug fix
- [ ] `test` — Test additions or corrections
- [ ] `docs` — Documentation only
- [ ] `refactor` — Code change that does not add a feature or fix a bug
- [ ] `perf` — Performance improvement
- [ ] `ci` — CI/CD configuration
- [ ] `chore` — Build, dependency, or maintenance task
- [ ] `security` — Security fix or hardening

---

## Changes

<!-- List the concrete changes made in this PR. Be specific. -->

-
-
-

---

## Testing

<!-- Describe how you tested these changes. -->

**Automated tests:**
```bash
# Paste the test command(s) you ran and their output summary
make check
```

**Manual testing (if applicable):**
<!-- Describe any manual steps, smoke tests, or testnet verification -->

**Test coverage impact:**
<!-- If this PR touches contract logic, include cargo-tarpaulin summary.
     Coverage must stay at or above 80%. -->

---

## Breaking Changes

<!-- Does this change break backward compatibility?
     If yes, describe what breaks and what migration is needed. -->

- [ ] This PR introduces a breaking change

<!-- If checked, describe the impact and required migration steps: -->

---

## Documentation

- [ ] `CHANGELOG.md` updated under `[Unreleased]`
- [ ] `docs/API.md` updated if contract API changed
- [ ] `README.md` updated if usage or setup changed
- [ ] New public functions/types have `///` doc comments
- [ ] ADR written if this is an architectural decision (see `docs/adr/README.md`)
- [ ] Migration guide added if this is a breaking change

---

## Code Review Checklist

### For the PR Author — complete before requesting review

**Functionality**
- [ ] The code does what the PR description says it does
- [ ] Edge cases and boundary conditions are handled
- [ ] Error paths return the correct `VaultError` variant
- [ ] No debug/dead code left in (no `println!`, `dbg!`, `console.log`, `TODO` without a linked issue)
- [ ] All `unwrap()` / `expect()` calls in non-test code are justified with a comment

**Tests**
- [ ] New behavior is covered by at least one test
- [ ] Every new `VaultError` variant that can be returned is tested with a negative test
- [ ] `require_auth()` is tested for every new mutating function
- [ ] Test names follow the `test_<function>_<scenario>_<outcome>` convention
- [ ] Test coverage is ≥ 80% (run `cargo tarpaulin --features testutils` to verify)
- [ ] No tests use `#[ignore]` without a linked issue explaining why

**Security** _(skip if only docs/CI change)_
- [ ] `require_auth()` is the **first** call in every new mutating contract function
- [ ] State is cleared/updated **before** any external token transfer (CEI pattern)
- [ ] No unbounded loops over user-controlled input
- [ ] New integer arithmetic cannot overflow (use `checked_*` or `saturating_*`)
- [ ] New error codes do not reuse existing code numbers (check `errors.rs`)
- [ ] No secrets, keys, or credentials in code or comments

**Performance** _(skip if only docs/CI change)_
- [ ] No new N+1 query patterns (iterating per-deposit instead of batching)
- [ ] New functions that iterate collections use a `limit` parameter
- [ ] Instruction budget impact considered for any new loop or storage access

**Style and readability**
- [ ] Formatting passes: `cargo fmt --all -- --check` (Rust), `npm run lint` (frontend)
- [ ] Clippy passes: `cargo clippy --all-targets --features testutils -- -D warnings`
- [ ] TypeScript type check passes: `npm run typecheck` (if frontend changed)
- [ ] Public functions and types have `///` doc comments explaining behavior and error conditions
- [ ] Commit messages follow Conventional Commits format (enforced by `pr-title-lint` CI)

**Contract-specific** _(skip if no contract changes)_
- [ ] `docs/API.md` updated for any new/changed/removed public functions
- [ ] Storage layout changes are documented in an ADR
- [ ] New persistent storage keys do not conflict with existing keys
- [ ] TTL bump is called for all new persistent storage writes
- [ ] WASM size is within 64 KB limit (`make check-wasm-size`)

**Frontend-specific** _(skip if no frontend changes)_
- [ ] New UI components are accessible (semantic HTML, ARIA labels where needed)
- [ ] New contract error codes are mapped to user-facing messages
- [ ] No hardcoded contract addresses or network-specific values (use `config.ts`)
- [ ] Wallet connection and disconnect paths tested manually

---

## Reviewer Notes

<!-- Optional: anything you want reviewers to pay special attention to,
     known limitations, or areas where you want specific feedback -->

---

## Severity Key (for reviewers)

| Label | Meaning |
|---|---|
| `[blocking]` | Must be fixed before merge. Correctness, security, or test gap. |
| `[important]` | Should be fixed. Can be a follow-up issue if scope is clearly bounded. |
| `[suggestion]` | Non-blocking improvement. Author decides. No response required. |
| `[nit]` | Minor style or wording. Author decides. |
| `[question]` | Seeking understanding. Not requesting a change. |
