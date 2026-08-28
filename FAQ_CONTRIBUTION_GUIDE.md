# Contributing to the SAFE-HAVEN FAQ

Welcome! This guide explains how to contribute FAQ questions, answers, improvements, and corrections to the SAFE-HAVEN project.

## Table of Contents

- [Why contribute?](#why-contribute)
- [Getting started](#getting-started)
- [Contribution types](#contribution-types)
- [Process](#process)
- [Guidelines](#guidelines)
- [Code of conduct](#code-of-conduct)

---

## Why Contribute?

The FAQ is a community resource. By contributing, you:

- **Help other users** solve problems faster
- **Reduce support burden** by documenting common issues
- **Improve project quality** by identifying gaps and edge cases
- **Earn recognition** as a project contributor
- **Shape the future** of SAFE-HAVEN documentation

---

## Getting Started

### Prerequisites

- A GitHub account
- Git installed locally
- Basic familiarity with Markdown
- Familiarity with SAFE-HAVEN (user or developer experience)

### Fork and clone

1. Fork the repository: https://github.com/kenedybok3/SAFE-HAVEN/fork
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/SAFE-HAVEN.git
   cd SAFE-HAVEN
   ```
3. Add upstream remote:
   ```bash
   git remote add upstream https://github.com/kenedybok3/SAFE-HAVEN.git
   ```

---

## Contribution Types

### 1. Add a new FAQ question & answer

**When to do this:**
- You see a common question repeated in Discord, GitHub issues, or support tickets
- A key feature or troubleshooting step is not documented
- You've solved a problem that others might encounter

**How:**

1. Check existing FAQs to avoid duplication
2. Create a branch: `git checkout -b docs/faq-new-question`
3. Edit `FAQ.md` and add your Q&A in the appropriate category
4. Follow the existing format (see below)
5. Push and open a PR

**Format template:**

```markdown
### Q##: [Your question here?]

**A:** [Your answer here]

**Key points:**
- Point 1
- Point 2

**Code example:**
```bash
command here
```

**Reference:** [Link to related docs](./path), [Another reference](./path)
```

### 2. Improve an existing answer

**When to do this:**
- An answer is unclear or missing details
- Code examples are outdated or incorrect
- A solution doesn't work for all cases
- Cross-references are broken

**How:**

1. Create a branch: `git checkout -b docs/faq-improve-answer-Q#`
2. Edit the relevant Q&A
3. Test code examples if applicable
4. Push and open a PR with title: `docs(faq): improve Q# answer`

### 3. Report an error or typo

**When to do this:**
- An answer is factually incorrect
- Code examples don't work
- Cross-references are broken
- Formatting is broken

**How (quick fix):**
- Open a GitHub issue with label `documentation`
- Include: which Q&A, what's wrong, suggested fix

**How (if you can fix it):**
- Create a branch: `git checkout -b docs/faq-fix-error-Q#`
- Fix the error
- Test if applicable
- Push and open a PR

### 4. Suggest a new topic for the FAQ

**When to do this:**
- You identify a gap in the FAQ (something not covered)
- You want to gather feedback before writing a full answer

**How:**

1. Open a GitHub issue with label `documentation: faq`
2. Include:
   - The question or topic
   - Why it's important
   - Suggested category (Setup, Usage, Troubleshooting, Advanced)
   - Any context or examples

---

## Process

### Step 1: Identify the need

**Option A: Found in discussion**
- Discord, GitHub issues, support emails
- Record the question and context

**Option B: Discovered yourself**
- You're learning SAFE-HAVEN and hit a gap
- Document what you learned

**Option C: Existing issue**
- Check the [FAQ GitHub project board](https://github.com/kenedybok3/SAFE-HAVEN/projects/faq-backlog)
- Pick a `help-wanted` or `good-first-issue` label

### Step 2: Create an issue (optional but recommended)

This helps maintainers understand your intent and provide early feedback.

```markdown
**Title:** FAQ: [Your topic]
**Labels:** `documentation`, `faq`

**Description:**
- Question/topic: "How do I..."
- Why it's needed: [Explain]
- Suggested answer outline: [Brief points]
- Category: Setup / Usage / Troubleshooting / Advanced
```

### Step 3: Fork, branch, and implement

```bash
# Fetch latest
git fetch upstream
git checkout upstream/main

# Create a feature branch (use a descriptive name)
git checkout -b docs/faq-withdraw-early-exit
# or
git checkout -b docs/faq-new-question

# Edit FAQ.md or create a new document
# (Always edit FAQ.md for new Q&As)
nano FAQ.md
```

### Step 4: Test your work

**For code examples:**
1. Copy the code snippet
2. Test it against testnet/local contract
3. Ensure it works as documented
4. Note any assumptions (network, account balance, etc.)

**For explanations:**
1. Have a non-expert review it
2. Ensure clarity and accuracy
3. Check cross-references

### Step 5: Commit and push

```bash
# Stage changes
git add FAQ.md

# Commit (use a clear message)
git commit -m "docs(faq): add Q# about [topic]"

# Push to your fork
git push origin docs/faq-withdraw-early-exit
```

### Step 6: Create a pull request

Go to: https://github.com/kenedybok3/SAFE-HAVEN/pull/new/your-branch

**PR title (keep under 70 characters):**
- `docs(faq): add Q# about [topic]`
- `docs(faq): improve Q# answer`
- `docs(faq): fix typo in Q#`

**PR description:**

```markdown
## Summary
[Brief explanation of what you added/changed]

## Type of change
- [ ] New FAQ question & answer
- [ ] Improved existing answer
- [ ] Fixed error or typo
- [ ] Other: ___

## Category
- [ ] Setup & Installation
- [ ] Usage & Basic Operations
- [ ] Troubleshooting
- [ ] Advanced Topics
- [ ] Community Contributions

## What was tested
[Describe what you tested or verified]

## References
- Closes #[issue number if applicable]
- Related to: [Discord discussion / support ticket]
```

### Step 7: Review and iterate

Maintainers will:
1. Check accuracy and clarity
2. Verify code examples work
3. Ensure formatting and style consistency
4. Request changes if needed

You can update your PR by:
```bash
git add FAQ.md
git commit -m "docs(faq): address review feedback"
git push origin docs/faq-withdraw-early-exit
```

### Step 8: Merge!

Once approved, maintainers merge your contribution. You'll be added to the contributors list.

---

## Guidelines

### Format & Style

**Markdown structure:**
- Use H3 (`###`) for Q&A headers
- Use **bold** for emphasis
- Use backticks for code/variable names
- Use code blocks (```bash, ```rust, etc.) for multi-line examples

**Example:**

```markdown
### Q##: How do I [do something]?

**A:** [Clear, concise answer]

**Steps:**
1. First step
2. Second step
3. Third step

**Code example:**
\`\`\`bash
command here
\`\`\`

**Note:** Any important caveats or assumptions.

**Reference:** [Link](./path), [Another](./path)
```

### Accuracy

- **Test code examples** before submitting
- **Link to relevant docs** (README, MONITORING, etc.)
- **Note assumptions** (network, balance, permissions, etc.)
- **Be precise** about error codes, limits, and behavior
- **Distinguish** between facts and opinions/best practices

### Clarity

- **Use simple language** (assume non-Soroban experts read this)
- **Explain the "why"** not just the "how"
- **Provide context** (when would you use this? when wouldn't you?)
- **Use examples** that are realistic and complete
- **Proofread** for typos and grammatical errors

### Organization

- **Place in correct category**: Setup, Usage, Troubleshooting, Advanced
- **Avoid duplicates**: Check existing FAQs first
- **Link related answers**: Use internal cross-references
- **Maintain numbering**: Don't renumber Q&As; use the next available number

### Content scope

**In scope for FAQ:**
- Common setup and configuration issues
- How to use core features
- Error explanations and solutions
- Best practices
- Admin and monitoring tasks

**Out of scope (direct to other docs):**
- API reference (see README.md)
- Architecture deep-dives (see docs/adr/)
- Video tutorials (see separate medium)
- Personalized support (direct to Discord/email)

---

## FAQ Categories Explained

### Setup & Installation (Q1–Q4)

Prerequisites, development environment, deployment, configuration.

**Examples:**
- Prerequisites and tools
- Local dev setup
- Testnet/mainnet deployment
- Frontend configuration

### Usage & Basic Operations (Q5–Q9)

How to use core features: deposits, withdrawals, staking, transfers.

**Examples:**
- Creating deposits
- Withdrawing funds
- Earning staker rewards
- Using deposit_for
- Ledger-based deposits

### Troubleshooting (Q10–Q16)

Error messages, wallet issues, state mismatches, and recovery.

**Examples:**
- Error code explanations
- Wallet connection problems
- Simulation vs. submission failures
- Lock duration limits
- Storage expiration
- Fund recovery

### Advanced Topics (Q17–Q25)

Admin functions, monitoring, disaster recovery, optimization.

**Examples:**
- Staker registry
- Admin operations (pause, transfer, renounce)
- Health monitoring
- Incident recovery
- Gas optimization
- Data mismatch troubleshooting

### Community Contributions (Q26–Q28)

Meta: How to contribute to FAQ and get support.

**Examples:**
- Contributing to FAQ
- Reporting issues
- Where to ask questions

---

## Common Mistakes to Avoid

1. **Not testing code examples**
   - Run them yourself first
   - Test both success and error paths

2. **Outdated references**
   - Link to current documentation
   - Check GitHub for renamed files or URLs

3. **Assuming too much knowledge**
   - Explain Soroban terms the first time you use them
   - Provide context for new users

4. **Typos and formatting**
   - Use a spell-checker
   - Ensure Markdown renders correctly (preview in GitHub)

5. **Scope creep**
   - FAQ answers should be concise
   - Complex topics → link to full docs
   - Personalized support → direct to Discord

6. **Duplicate content**
   - Search existing FAQs first
   - If related, cross-reference instead of repeating

7. **Missing references**
   - Always include links to relevant docs
   - Help users dig deeper if interested

---

## Recognition

Contributors are recognized in multiple ways:

1. **GitHub contributor badge** (automatic after merge)
2. **Contributors list** in main README (manual update)
3. **Community mentions** in release notes
4. **Discord role** (optional, ask in #contributors)

---

## Need Help?

- **Questions about the process?** Ask in [GitHub Discussions](https://github.com/kenedybok3/SAFE-HAVEN/discussions)
- **Want feedback before writing?** Open a draft PR
- **Unsure if your topic fits?** Open an issue and ask
- **Want to brainstorm?** Join [Discord](https://discord.gg/yourserver) #documentation

---

## FAQ Maintenance

**Maintainers commit to:**
- Reviewing new contributions within 48 hours
- Keeping links and examples current
- Updating obsolete information
- Removing duplicate Q&As

**Community helps by:**
- Reporting broken links or outdated examples
- Suggesting improvements
- Testing code examples
- Helping other contributors with feedback

---

## Code of Conduct

By contributing, you agree to:
- Treat all community members with respect
- Provide constructive feedback
- Focus on improving the project
- Report issues privately if they're security-related
- Give credit where due

See [CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md) for full details.

---

## Summary Checklist

Before submitting a PR, verify:

- [ ] I've read and understood this guide
- [ ] My contribution is in the appropriate category
- [ ] I've tested code examples (if applicable)
- [ ] I've linked to relevant documentation
- [ ] I've used consistent Markdown formatting
- [ ] I've explained the "why" in addition to the "how"
- [ ] I've proofread for typos and clarity
- [ ] I haven't duplicated existing Q&As
- [ ] My PR title is concise and descriptive
- [ ] My PR description explains the changes

---

**Thank you for contributing to SAFE-HAVEN! 🚀**

Your effort helps make this project more accessible and useful for everyone.

---

**Last updated:** August 28, 2026
