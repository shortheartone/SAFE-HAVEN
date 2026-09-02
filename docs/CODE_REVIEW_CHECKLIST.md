# Code Review Checklist and Guidelines

**Document status:** Active  
**Applies to:** All contributors and reviewers  
**Last reviewed:** 2026-09-02

This document defines the code review process for SAFE-HAVEN: the checklist
reviewers use, how feedback is communicated, how reviewers are assigned, and
what metrics indicate a healthy review process.

---

## Table of Contents

1. [Why Code Review Matters for SAFE-HAVEN](#1-why-code-review-matters-for-safe-haven)
2. [Automated Checks](#2-automated-checks)
3. [Review Checklist](#3-review-checklist)
4. [Severity Levels](#4-severity-levels)
5. [Approval Process](#5-approval-process)
6. [Reviewer Assignment](#6-reviewer-assignment)
7. [Code Review Guidelines](#7-code-review-guidelines)
8. [Code Review Metrics](#8-code-review-metrics)
9. [Training and Onboarding](#9-training-and-onboarding)

---

## 1. Why Code Review Matters for SAFE-HAVEN

SAFE-HAVEN manages real user funds on an immutable blockchain. There is no
undo button. A bug that passes review and reaches mainnet can permanently
lock or misdirect funds.

Code review is therefore not a formality — it is the last human line of
defense before code ships. This means:

- Every PR that touches contract logic requires at least one reviewer who
  understands the Soroban security model.
- Reviewers are responsible for what they approve. "I didn't look closely"
  is not an acceptable defense after a mainnet incident.
- The goal is to ship correct code, not to ship fast. Speed is a nice-to-have;
  correctness is non-negotiable.

That said, good reviews are thorough **and** timely. Blocking PRs for months
over minor style issues is also a failure mode. The severity system in §4
exists to separate "must fix" from "nice to fix".

---

## 2. Automated Checks

All automated checks must pass before a human review begins. Do not waste
reviewer time on issues that tooling can catch.

### Checks that run on every PR (CI)

| Check | Job | What it catches |
|---|---|---|
| `cargo fmt --all -- --check` | `lint` | Formatting violations |
| `cargo clippy --all-targets --features testutils -- -D warnings` | `lint` | Rust lints and potential bugs |
| `cargo test --features testutils` | `test` | Regression in contract logic |
| `cargo test --doc --features testutils` | `test` | Broken doc examples |
| `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` | `lint` | Missing or broken doc comments |
| `cargo audit` | `security-audit` | Known CVEs in dependencies |
| `cargo deny check` | `deny` | License violations, banned crates |
| `cargo geiger` | `geiger` | Unsafe Rust code |
| `tsc --noEmit` | `frontend` | TypeScript type errors |
| `npm run build` | `frontend` | Frontend build failure |
| PR title format | `pr-title-lint` | Conventional Commits format |
| PR summary length | `doc-lint` | Empty or template PR description |
| API.md sync | `check-doc-code-sync` | Contract functions not documented in API.md |
| CHANGELOG update | `doc-lint` | Non-docs PRs without CHANGELOG entry |
| WASM size ≤ 64 KB | `build` | Contract binary too large |

### Pre-commit hook (local)

The `.githooks/pre-commit` hook runs `cargo fmt`, `cargo clippy`, ESLint, and
`tsc --noEmit` before each commit. Activate it once after cloning:

```bash
git config core.hooksPath .githooks
```

The CI jobs are authoritative. A PR that passes the pre-commit hook but fails
CI must be fixed before merging.

### Adding a new automated check

New automated checks should be added to `ci.yml` or `docs.yml` with:

1. A descriptive job name
2. A comment explaining what the check catches and why
3. Pinned action versions (for supply-chain security)

Document the new check in this table so reviewers know what is already covered.

---

## 3. Review Checklist

The PR template includes this checklist for the **author** to complete before
requesting review. This section is for **reviewers**: what to look for.

### 3.1 Functionality

- [ ] The implementation matches the PR description and issue requirements.
- [ ] All edge cases described in the issue are handled.
- [ ] Error conditions return the documented `VaultError` variant.
- [ ] No silent failures (errors swallowed without returning or logging).
- [ ] Re-entrancy safe: for any function that calls an external token contract,
      contract state is fully updated **before** the external call.

**Soroban-specific:**
- [ ] `require_auth()` is the **first** meaningful statement in every mutating function.
- [ ] Persistent storage writes call `extend_ttl` (use the `set_*` helpers in `storage.rs`).
- [ ] New storage keys do not collide with existing key variants in `types.rs`.
- [ ] Any function that iterates a collection is paginated or capped.

### 3.2 Tests

- [ ] New behavior is covered by at least one test with a descriptive name.
- [ ] Every error path that can be returned has a corresponding negative test.
- [ ] Auth rejection is explicitly tested for every new mutating function.
- [ ] Integration scenario is tested if the PR adds a new multi-step workflow.
- [ ] Test coverage does not drop below 80%.
- [ ] No `#[ignore]` without a linked issue.

If a PR author claims "existing tests cover this," verify by looking at the
relevant test in `test.rs` — don't take it on faith.

### 3.3 Security

> These items are mandatory for any PR that touches `contract.rs`, `storage.rs`,
> or any function that transfers tokens or authorizes actions.

- [ ] `require_auth()` is first (see above — double-checked here because it's critical).
- [ ] Checks-Effects-Interactions: storage cleared **before** `token.transfer()`.
- [ ] No unbounded iteration over user-controlled data.
- [ ] Integer arithmetic: `i128` operations on `amount × penalty_bps` cannot overflow.
      Use `checked_mul` or `saturating_mul` where risk exists.
- [ ] New error codes are unique — no number reuse in `errors.rs`.
- [ ] No secrets, API keys, or private key material anywhere in the diff.
- [ ] Auth guard on admin-only functions is tested with a non-admin caller.

### 3.4 Performance

- [ ] No new per-item RPC calls in the frontend (use batch queries).
- [ ] No new N+1 storage reads in the contract.
- [ ] New functions that return collections accept a `limit` parameter.
- [ ] WASM size is within budget (CI enforces this; confirm the `build` job passed).

### 3.5 Documentation

- [ ] New public contract functions have `///` doc comments explaining:
      - What the function does
      - All parameters
      - Return value
      - Possible error codes
- [ ] `docs/API.md` is updated for any new/changed/removed public functions
      (CI checks this; confirm `check-doc-code-sync` passed).
- [ ] `CHANGELOG.md` has an entry under `[Unreleased]`.
- [ ] An ADR is included or linked if this is a Type 3 architectural decision.

### 3.6 Frontend-Specific

- [ ] New UI components have semantic HTML elements and ARIA labels where needed.
- [ ] Contract error codes introduced in this PR are mapped to user-facing error messages.
- [ ] No hardcoded network-specific values (contract IDs, RPC URLs must come from `config.ts`).
- [ ] TypeScript: no `any` types without a comment justifying the exception.
- [ ] React hooks follow the Rules of Hooks (no conditional hook calls).

---

## 4. Severity Levels

Reviewers must label each comment with a severity prefix so authors
understand what must be fixed vs. what is optional.

| Label | When to use | Author action required |
|---|---|---|
| `[blocking]` | Correctness bug, security issue, missing test for a critical path, or API contract violation | **Must fix** before the PR can be approved |
| `[important]` | Significant concern that should be addressed, but could be tracked as a follow-up issue with a linked GitHub issue | Fix in this PR, **or** open a follow-up issue and link it in the PR |
| `[suggestion]` | Non-blocking improvement: cleaner code, better abstraction, alternative approach | Author decides. No response required |
| `[nit]` | Minor style, naming, or wording preference | Author decides. No response required |
| `[question]` | Seeking to understand the approach; not requesting a change | Author answers if clarification helps other reviewers |

### Examples

```
[blocking] `require_auth()` is called after the storage read on line 42.
It must be the first call in this function. See CODING_STANDARDS.md §2.6.

[important] The error here swallows the `VaultError::FundsStillLocked` variant.
The frontend has no way to show the user a useful message. Consider returning
the error or mapping it explicitly. I'll open a follow-up issue if you prefer
not to block this PR.

[suggestion] This could be expressed more clearly as an iterator chain:
  let ids: Vec<u32> = entries.iter().map(|e| e.id).collect();

[nit] Typo: "unlcok_time" → "unlock_time".

[question] Why is `extend_ttl` called with `BUMP_TARGET` here rather than
`BUMP_THRESHOLD`? I want to make sure I understand the invariant.
```

### What `[blocking]` always includes

- The exact file and line number (or a quote of the relevant code)
- What is wrong
- What the correct pattern is (or a reference to where it is documented)

A `[blocking]` comment without a suggested fix is incomplete.

---

## 5. Approval Process

### Minimum approvals

| PR type | Approvals required |
|---|---|
| All PRs | ≥ 1 maintainer approval |
| Contract logic changes (`contract.rs`, `storage.rs`) | ≥ 1 explicit CODEOWNERS approval (enforced by GitHub) |
| Security-sensitive changes (auth, key management, penalty logic) | ≥ 1 maintainer with smart contract security experience |
| ADR-backed architectural changes | ADR must be merged before or alongside the implementation PR |

### Merge conditions

A PR may be merged only when all of the following are true:

1. All CI jobs pass (green checkmarks on all required status checks).
2. At least one maintainer has approved.
3. All `[blocking]` reviewer comments are resolved or explicitly dismissed by
   the reviewer who left them.
4. No unresolved questions from CODEOWNERS reviewers.
5. The PR is not in draft state.
6. No merge conflicts with the target branch.

### Who can merge

- Maintainers may merge their own approved PRs to `develop`.
- Maintainers must not merge their own PRs to `main` unless there is no other
  available reviewer (e.g., emergency hotfix with documented justification).
- Contributors (non-maintainers) cannot merge any PR.

### Stale approvals

If a PR receives new commits after being approved, the approval is considered
stale. GitHub's "Dismiss stale reviews" branch protection option enforces this
automatically. The reviewer must re-approve after significant changes.

---

## 6. Reviewer Assignment

### CODEOWNERS

The `CODEOWNERS` file enforces review requirements automatically:

- `contracts/safe-haven/src/contract.rs` → `@kenedybok3`
- `contracts/safe-haven/src/storage.rs` → `@kenedybok3`
- All other files → `@kenedybok3` (default owner)

These assignments ensure that security-critical files always get an expert review.

### Volunteer reviewers

For PRs not covered by CODEOWNERS or where the PR author is the CODEOWNERS
reviewer, assign a volunteer reviewer using this strategy:

1. **Familiarity first:** Assign someone who has recently touched the affected files.
   Use `git log --follow -- <file>` to identify candidates.
2. **Distribute load:** Avoid assigning the same reviewer to more than 3 open PRs
   simultaneously.
3. **New contributor buddy:** First-time contributors should be assigned an
   experienced reviewer who can provide mentorship, not just approval.

### Self-review exception

There is no self-review. Authors may not approve their own PRs.

---

## 7. Code Review Guidelines

### For reviewers

**Before starting a review:**
- Understand the context. Read the linked issue and any referenced ADRs before
  reviewing the diff. A review without context is less useful.
- Check CI. If automated checks are failing, wait for the author to fix them
  before reviewing. There is no point reviewing code that will change when
  formatting/lint errors are fixed.

**During a review:**
- Review the entire diff, not just the changed lines. Changes can have
  interactions with surrounding context.
- For contract changes, mentally trace through the state machine: what happens
  if this function is called twice? What if the token transfer fails mid-execution?
- Ask questions when something is unclear. A confusing code block is a bug
  waiting to happen, even if the current logic is correct.
- Leave positive comments too. Acknowledgment of good work is motivating and
  useful signal for what to do more of.

**Completing a review:**
- Use the correct GitHub review action:
  - **"Approve"** — Ready to merge pending minor nits or author's discretion on suggestions
  - **"Request changes"** — Contains `[blocking]` comments that must be addressed
  - **"Comment"** — Questions only; not a formal approval or block
- Do not "approve" a PR with unresolved `[blocking]` comments.
- Respond to author replies within 2 business days. A review with no follow-up
  blocks the PR unnecessarily.

### For authors

**Before requesting review:**
- Self-review your own diff in GitHub. You will catch things in the diff view
  that you miss in your editor.
- Ensure all automated checks pass.
- Complete the PR checklist in the PR template.
- Keep PRs focused. A PR that touches the contract, frontend, docs, CI, and
  dependencies is hard to review. Break large changes into smaller PRs when
  possible.

**While under review:**
- Respond to every comment, even if just `done` or `good point, updated`.
- If you disagree with a `[blocking]` or `[important]` comment, explain your
  reasoning in the PR — do not simply ignore it.
- If a reviewer's suggestion changes the scope of the PR significantly, agree
  on whether to include it or track it as a separate issue.
- Do not push large refactors mid-review without notifying the reviewer. It
  invalidates their in-progress review.

**After review:**
- Resolve comments that have been addressed. Do not close a conversation thread
  without either making the change or explaining why you did not.
- Request a re-review from the reviewer after addressing `[blocking]` comments.

### Review anti-patterns to avoid

| Anti-pattern | Problem |
|---|---|
| Rubber-stamp approvals ("LGTM 👍" with no comments on a complex diff) | Provides no safety value; maintainer is still liable |
| Nitpicking on auto-fixable style | Wastes time; let `cargo fmt` and ESLint handle it |
| `[blocking]` without a fix suggestion | Leaves the author stuck without guidance |
| Reviewing only the new lines, ignoring context | Misses interactions with surrounding code |
| Waiting for "perfect" before approving | Blocks shipping; use `[suggestion]` for non-critical improvements |
| Verbal approvals on Discord | Not a valid approval; approval must be in GitHub |

---

## 8. Code Review Metrics

Tracking review metrics surfaces bottlenecks and helps improve the process.
Measure the following quarterly:

| Metric | Target | How to measure |
|---|---|---|
| Median time to first review | ≤ 3 business days | GitHub Insights → Pull Requests → Time to First Review |
| Median PR merge time | ≤ 7 business days from opening | GitHub Insights → Merge Time |
| PRs merged without review | 0 | Search `is:merged is:pr review:none` |
| PRs with `[blocking]` comments resolved before merge | 100% | Manual audit of merged PRs |
| PRs failing CI on first push | < 20% | Count CI failures on first-push vs. subsequent pushes |

### Quarterly review process

At each quarterly retrospective:

1. Pull the metrics listed above.
2. Identify the top bottleneck (usually time-to-review or CI failure rate).
3. Propose one concrete improvement (e.g., add a pre-commit check, improve
   documentation, pair contributors on onboarding).
4. Record the improvement and owner in the decision log (`docs/COMMUNICATION_PROCESS.md §7`).

---

## 9. Training and Onboarding

### For new contributors

Before opening your first PR:

1. Read `CONTRIBUTING.md` in full.
2. Read `CODING_STANDARDS.md` — both Rust and TypeScript sections.
3. Read `docs/TESTING_STRATEGY.md` — understand what test coverage is required.
4. Look at 2–3 recently merged PRs to understand the expected quality level.
5. Activate the pre-commit hook: `git config core.hooksPath .githooks`.

### For new reviewers

Before conducting your first review on contract logic:

1. Read the [Soroban Security Model](https://developers.stellar.org/docs/smart-contracts/security) documentation.
2. Familiarize yourself with Checks-Effects-Interactions for Soroban.
3. Read all five ADRs in `docs/adr/` to understand why the codebase is
   structured as it is.
4. Review a low-risk PR (documentation, test addition) as your first review,
   with a more experienced reviewer looking over your review.

### Recurring review training

Once per quarter, the team reviews one past `[blocking]` comment from a
merged PR and discusses:

- Was the blocking issue real?
- Did the reviewer explain clearly enough?
- What pattern does this represent (and should it be automated)?
- Is there a new test or CI check that would have caught it automatically?

This keeps the review process improving rather than stagnating.

---

*For changes to this document, open a PR with type `docs` and request review
from a maintainer. Significant process changes should be discussed in a
GitHub Discussion first.*
