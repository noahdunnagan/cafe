---
name: Code Audit
command: audit
description: Audit a directory for security, performance, safety, and cleanup issues. Applies real fixes only — no churn. Opens a PR if there are real fixes, otherwise reports clean. Usage: /audit <path> (defaults to repo root).
---

You are auditing a codebase for real, concrete improvements. Be conservative. Apply real wins only.

## Setup

Parse the input:
- If a path is given, audit that directory.
- If no path, audit the repo root.
- Read any `CLAUDE.md` or `AGENTS.md` at the repo root, then in the audited directory if present. These contain project-specific rules and constraints — obey them.

## Four passes

Apply real wins only. Skip subjective preferences. Do not change behaviour.

### 1. 🔒 Security
Hardcoded secrets, auth bypasses, command injection, path traversal, SSRF, XSS, SQL injection, unsafe deserialization, unvalidated input at trust boundaries, unsafe URL construction, `eval`/`innerHTML` with untrusted data. If exploitable, flag and fix.

### 2. ⚡ Performance
Patterns that hurt at scale. N+1 queries, unbounded collections, blocking async, missing indexes, redundant tree walks, repeated work in tight loops, unnecessary allocations. Flag only things with measurable impact — never theoretical micro-optimisations.

### 3. 🛡️ Safety
Things that crash or leak. Missing error propagation, silent error swallowing, race conditions, resource leaks (file handles, connections, listeners), unhandled edge cases (empty input, null, concurrent mutation), missing `await`, snapshot/mutation hazards, panics in production paths.

### 4. ✂️ Cleanup
Dead code, over-nested control flow that wants guard clauses, 10-line blocks that could be 3, duplicated logic, redundant intermediate variables. Be aggressive about distillation but never at the cost of clarity.

## Rules

- **Stay inside the audited directory.** Mention issues in shared or sibling code but do not modify them.
- **Respect existing code.** Do not replace working idioms purely on principle. Deliberate choices stay deliberate.
- **Honour project constraints.** If `CLAUDE.md`/`AGENTS.md` forbids a syntax or pattern (e.g. sandboxed runtimes, banned APIs), do not introduce it.
- **WHY comments only.** Never add WHAT comments. Match the existing comment style.
- No new features. No refactors for hypothetical futures. No abstractions the current code doesn't need.

## If you applied fixes

1. If the project clearly maintains a `CHANGELOG.md`, add a dated entry and bump the version. If not, skip this step — don't impose a changelog convention.
2. `git checkout -b audit/<short-descriptive-name>`
3. Commit with a conventional message and a body that lists fixes grouped by category (🔒 / ⚡ / 🛡️ / ✂️).
4. `git push -u origin audit/<short-descriptive-name>`
5. `gh pr create --title "chore(<area>): code audit pass" --body "<categorized list>"`
6. Return the PR URL plus a tight summary.

## If no real fixes

Don't branch, commit, or PR. Return: `No fixes for <area>: <one-sentence reason>`.

## Don't

- Add `Co-Authored-By: Claude` (or any AI-attribution) trailers to commits.
- Add `🤖 Generated with [Claude Code]` (or any AI-attribution) footers to commit messages or PR bodies.
- Add features.
- Refactor for hypothetical future needs.
- Add WHAT comments.
- Replace working code purely on principle.
