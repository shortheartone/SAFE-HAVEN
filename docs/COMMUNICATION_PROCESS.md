# Team Communication and Decision-Making Process

**Document status:** Active  
**Applies to:** All SAFE-HAVEN contributors and maintainers  
**Last reviewed:** 2026-09-02

---

## Table of Contents

1. [Communication Channels](#1-communication-channels)
2. [Response Time Standards](#2-response-time-standards)
3. [Decision-Making Framework](#3-decision-making-framework)
4. [Escalation Paths](#4-escalation-paths)
5. [Meeting Cadence](#5-meeting-cadence)
6. [Meeting Agendas and Minutes](#6-meeting-agendas-and-minutes)
7. [Decision Log](#7-decision-log)
8. [Feedback Process](#8-feedback-process)
9. [Remote Collaboration Guidelines](#9-remote-collaboration-guidelines)
10. [Communication Standards](#10-communication-standards)

---

## 1. Communication Channels

| Channel | Purpose | Expected audience | Async or sync |
|---|---|---|---|
| **GitHub Issues** | Bug reports, feature requests, tracking work | All contributors | Async |
| **GitHub Discussions** | Design questions, open-ended proposals, community Q&A | All contributors | Async |
| **GitHub Pull Requests** | Code review, implementation feedback | Contributors + reviewers | Async |
| **Discord `#safe-haven-dev`** | Day-to-day development chat, quick questions | Active contributors | Async (check daily) |
| **Discord `#safe-haven-security`** | Confidential security discussion | Maintainers only | Async |
| **Email (maintainer list)** | Formal notices, external security reports, legal | Maintainers | Async |
| **Scheduled video calls** | Sprint planning, architecture reviews, retrospectives | Contributors by invitation | Sync |

### Channel selection guide

Use the right channel for the message — routing signals matter.

- **Permanent knowledge** (decisions, architectural rationale, runbooks) → GitHub Issues or Discussion, then documented in a markdown file
- **Temporary coordination** (PR reviews, quick clarifications) → GitHub PR comments or Discord
- **Security concerns** → Email or Discord `#safe-haven-security` only. **Never** GitHub Issues
- **Community questions** → GitHub Discussions `Q&A` category
- **Blocking a release** → GitHub Issue with `blocking` label, ping `@kenedybok3` on Discord

### What does NOT belong on Discord

- Architectural decisions (write a GitHub Discussion or ADR instead)
- Bug reports (open a GitHub Issue)
- Feature requests (open a GitHub Issue with `enhancement` label)
- Security vulnerabilities (use `#safe-haven-security` or email — see `SECURITY.md`)

Anything discussed informally on Discord that results in a decision must be
written up and linked to the relevant issue or PR. "We decided on Discord"
is not an acceptable decision record.

---

## 2. Response Time Standards

These are expectations, not hard contracts. Contributors are volunteers or
part-time. Maintainers should communicate unavailability in advance.

| Type | Expected response | Escalation after |
|---|---|---|
| Security vulnerability report | 48 hours (acknowledgement only) | Email maintainer directly |
| PR review request | 3 business days | Ping reviewer on Discord |
| Issue triage (new bug) | 5 business days | Label `needs-triage`, ping maintainer |
| Issue triage (new feature request) | 2 weeks | Label `backlog`, milestone for future sprint |
| GitHub Discussion reply | 5 business days | No formal escalation; low urgency |
| Discord DM to a maintainer | 24 hours during work days | Try a different maintainer |

**Maintainers on leave** should set an out-of-office note in their GitHub
profile bio or Discord status and name a substitute reviewer in the
`#safe-haven-dev` channel.

---

## 3. Decision-Making Framework

Not all decisions are equal. Using the wrong process wastes time; using no
process creates confusion. This framework matches decision weight to process.

### Decision types

| Type | Description | Process | Who decides |
|---|---|---|---|
| **Type 1: Minor** | Cosmetic, reversible, low blast radius (variable rename, doc fix, small refactor) | No process needed. Use your judgment, make the change, get it reviewed normally. | PR author with reviewer approval |
| **Type 2: Significant** | Changes behavior, affects multiple files, has tradeoffs (new feature, API change, dependency addition) | Open a GitHub Issue first. Discuss approach in the issue. Implement once approach is agreed. | PR author + maintainer sign-off |
| **Type 3: Architectural** | Changes the public contract interface, storage layout, security model, or multi-contract interactions | Required ADR (see `docs/adr/README.md`). ADR must be approved before implementation begins. | Maintainers consensus |
| **Type 4: Governance** | Contract upgrade, admin transfer, key rotation, or on-chain governance action | Follow `GOVERNANCE.md`. Requires on-chain vote and timelock. | Community vote + maintainers |

### How to identify the right type

When unsure, ask yourself:

1. If this goes wrong, can it be reverted with a one-line fix? → Type 1
2. Would another contributor need context to understand why this was done? → Type 2
3. Does this change what the contract can do, how it stores data, or who can authorize actions? → Type 3
4. Does this require deploying a new contract or executing an on-chain governance action? → Type 4

If still unsure, treat it as one level higher. It's better to over-communicate
on an architectural decision than to under-communicate.

### Consensus model

SAFE-HAVEN uses **rough consensus** for Types 1–3:

- Approval from at least **one maintainer** is required to merge any PR.
- If two maintainers disagree on a Type 2 or Type 3 decision, they must
  write up the tradeoffs in the GitHub Issue and discuss asynchronously.
  If unresolved in 5 business days, the senior maintainer makes the call
  and documents the rationale.
- For Type 3 decisions, the ADR is the record of consensus. A merged ADR
  means the decision was accepted.

---

## 4. Escalation Paths

### Technical conflict (two reviewers disagree on approach)

1. Both parties write their position in the PR or GitHub Issue (not Discord).
2. Tag `@kenedybok3` for tiebreaker input.
3. If the decision is architectural, require an ADR before proceeding.
4. If still unresolved after 5 business days, the repository owner makes the call.
   The outcome is documented in the issue.

### Blocked PR (no reviewer activity)

1. Wait 3 business days after requesting review.
2. Ping the reviewer in the PR comments (`@reviewer ping — review needed`).
3. Wait 2 more business days.
4. Post in `#safe-haven-dev` Discord with a link to the PR.
5. Wait 1 more business day.
6. Tag a different maintainer to review.

### Stale issue (no progress for 30 days)

1. Add the `stale` label.
2. Comment asking if the issue is still relevant and if anyone is working on it.
3. If no response in 14 more days, close with a "closing stale" comment.
   Issues can be reopened at any time with new information.

### Code of conduct violation

Report to the repository owner (`@kenedybok3`) via private Discord message
or email. Do not discuss the incident publicly.

---

## 5. Meeting Cadence

SAFE-HAVEN is an open-source project; formal meetings are kept to a minimum.
The following cadence applies when the project has active contributors
coordinating on a release.

| Meeting | Frequency | Duration | Participants | Primary goal |
|---|---|---|---|---|
| **Sprint Planning** | Start of each sprint (every 2 weeks) | 45 minutes | Active contributors | Agree on sprint scope; assign issues |
| **Sprint Review** | End of each sprint (every 2 weeks) | 30 minutes | Active contributors | Demo completed work; identify blockers |
| **Architecture Review** | As needed (before Type 3 decisions) | 60 minutes | Maintainers + relevant contributors | Discuss ADR, reach consensus |
| **Retrospective** | Monthly | 30 minutes | Active contributors | Process improvement; what went well / what didn't |
| **Release Review** | Before each release tag | 30 minutes | Maintainers | Go/no-go decision, final checklist |

### Meeting norms

- **Async first.** If a question can be answered in a GitHub Discussion, do not
  schedule a meeting.
- **Agenda required.** No agenda → no meeting. Agendas are posted at least
  24 hours before the call.
- **Start and end on time.** Respect participants' time. If the meeting
  runs long, continue async in GitHub.
- **No decisions without notes.** Every meeting must produce written notes
  (see §6) within 24 hours of the call.
- **Video optional.** Voice-only is fine. No one is required to turn on a camera.

---

## 6. Meeting Agendas and Minutes

### Agenda template

Agendas are posted as a GitHub Discussion (category: "Meetings") at least
24 hours before the meeting.

```markdown
## Sprint Planning — YYYY-MM-DD — HH:MM UTC

**Participants:** @username1, @username2, ...
**Duration:** 45 min
**Link:** [video call link]

### Agenda

1. Review previous sprint (5 min)
   - Completed: #issue1, #issue2
   - Carried over: #issue3 (blocked by X)
2. Demo / show-and-tell (10 min)
3. Upcoming sprint scope (25 min)
   - Candidates: #issue4, #issue5, #issue6
   - Capacity check: who is available this sprint?
4. Blockers and dependencies (5 min)
```

### Minutes template

Minutes are posted in the same GitHub Discussion thread as the agenda,
within 24 hours of the call.

```markdown
## Minutes — Sprint Planning — YYYY-MM-DD

**Attendees:** @username1, @username2
**Absent:** @username3 (notified)
**Recorded by:** @username1

### Decisions made

| Decision | Rationale | Follow-up |
|---|---|---|
| Include #issue4 in sprint | Low risk, contributor available | @username2 picks up |
| Defer #issue5 | Depends on #issue3 which is blocked | Revisit next sprint |

### Action items

| Item | Owner | Due date |
|---|---|---|
| Open ADR for #issue6 | @username1 | YYYY-MM-DD |
| Update CHANGELOG for v1.2.0 | @username2 | Before release |

### Next meeting

Sprint Review: YYYY-MM-DD HH:MM UTC
```

---

## 7. Decision Log

Significant decisions that do not warrant a full ADR (Type 2 decisions) are
logged here. Each entry links to the GitHub Issue or Discussion where the
decision was discussed.

> **Format:** Add a new row at the top. Do not edit past entries.

| Date | Decision | Rationale | Link | Decided by |
|---|---|---|---|---|
| 2026-09-02 | Adopt this communication process document | Addresses issue #421: team needs documented channels and decision framework | #421 | @kenedybok3 |
| 2026-09-02 | Testing target set at 80% coverage, not 100% | 100% coverage would require testing Soroban SDK internals; 80% is realistic for contract logic | #419 | @kenedybok3 |
| 2026-09-02 | Jekyll + minima for docs site | Minimal setup, free on GitHub Pages, no custom toolchain, easy for contributors to edit | #420 | @kenedybok3 |

> For architectural decisions (Type 3), see `docs/adr/README.md`.  
> For on-chain governance decisions (Type 4), see `GOVERNANCE.md`.

---

## 8. Feedback Process

### Code review feedback

Feedback in PRs must follow the severity convention defined in the
`CODE_REVIEW_CHECKLIST.md`:

- **`[blocking]`** — Must be fixed before merge. Correctness or security issue.
- **`[important]`** — Should be fixed but could be a follow-up issue if minor.
- **`[suggestion]`** — Non-blocking idea or improvement. Author decides.
- **`[nit]`** — Minor style or readability note. Author decides, no response required.
- **`[question]`** — Seeking understanding, not requiring a change.

### Documentation feedback

Use the "Was this page helpful?" link at the bottom of each documentation
page. This opens a pre-filled GitHub Issue. Any contributor can file
documentation issues.

### Process feedback

Process improvements are discussed in the monthly retrospective. To raise
a process concern before the next retrospective:

1. Post in `#safe-haven-dev` Discord with tag `[process]`.
2. If the concern is substantive, open a GitHub Discussion (category: "Processes").
3. If a process change is agreed, update the relevant document and open a PR.

### Contributor feedback

Maintainers should acknowledge new contributor PRs within 3 business days
with either a review or an ETA for review. Feedback must be:

- **Specific**: reference the exact line or function
- **Actionable**: explain what to change and why
- **Kind**: separate the code from the person

Reviewers who repeatedly leave unconstructive feedback will be asked to
improve by a maintainer.

---

## 9. Remote Collaboration Guidelines

SAFE-HAVEN is a remote-first, asynchronous project. Contributors work across
time zones.

### Writing over talking

Documentation, GitHub Issues, and PR descriptions are the source of truth.
Verbal or Discord-only agreements that are not written down do not exist.

Before starting significant work:
1. Check for an existing issue.
2. If none exists, open one and assign yourself.
3. Comment with your intended approach before investing more than an hour.

This prevents duplicate work and ensures the team can follow your reasoning.

### Time zone courtesy

- Do not expect same-day responses from contributors in other time zones.
- When posting in Discord, include enough context that someone reading it
  8 hours later can understand and respond without needing clarification.
- Schedule meetings in UTC. Always include a UTC time in calendar invites.
- Record sync meetings and share the notes publicly (excluding any sensitive
  discussions) so contributors who could not attend can stay informed.

### Availability and absence

- If you are unavailable for more than 3 business days, post in `#safe-haven-dev`
  or update your GitHub profile bio.
- If you have an open PR awaiting review or a blocking issue assigned to you,
  name a substitute or update the issue with your expected return date.
- No contributor is obligated to respond on weekends or holidays.

### Working in public

Prefer public GitHub comments over private Discord messages for
technical discussions. This builds a searchable archive and lets
other contributors learn from the conversation.

Exception: security discussions, personnel matters, and private
contributor feedback must remain private.

---

## 10. Communication Standards

### Clarity and completeness

Every communication (issue, PR, Discord message, email) should include:

- **What** you are describing or proposing
- **Why** it matters or is needed
- **What** a reader should do with the information (action required, FYI, decision needed)

Vague requests create back-and-forth overhead. State what you need clearly.

### GitHub Issues

Issues are the project's primary task tracker.

| Field | Requirement |
|---|---|
| Title | Concise, specific. Starts with a verb: "Add X", "Fix Y", "Document Z" |
| Description | At minimum: what is broken/missing, why it matters, reproduction steps (for bugs) |
| Labels | At least one label: `bug`, `enhancement`, `documentation`, `security`, `chore` |
| Milestone | Required for issues targeted at a specific release |
| Assignee | Required when someone starts work on the issue |

### Pull Request descriptions

See the PR template (`.github/pull_request_template.md`) for the required
structure. Every PR must have:

- A meaningful summary (not the template placeholder text)
- A list of changes
- Description of how it was tested
- Completed checklist

### Commit messages

Commits must follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <short summary>

[optional body]

[optional footer: Closes #NNN]
```

Types: `feat`, `fix`, `test`, `docs`, `ci`, `chore`, `refactor`, `security`, `perf`

Examples:
```
feat(contract): add deposit_by_ledger function
fix(frontend): handle FundsStillLocked error in withdraw flow
docs: add communication process document
test(contract): add auth rejection test for emergency_withdraw
```

The PR title is also validated by CI (`pr-title-lint` job in `ci.yml`) and must
follow the same conventional commits format.

### Inclusive language

- Use gender-neutral language: "they/them" as singular, "contributor" instead of
  "he/she".
- Avoid jargon that excludes non-native English speakers.
- Avoid phrases that normalize exclusion: "blacklist/whitelist" → "blocklist/allowlist",
  "master" → "main".

---

*For questions about this process, open a GitHub Discussion or ping a maintainer.*
*This document is reviewed at the start of each major release cycle.*
