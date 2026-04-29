---
name: Audit All
command: audit-all
description: Run /audit in parallel across multiple directories using isolated git worktrees. One PR per directory with real fixes; no PR for clean ones. Usage: /audit-all <glob-or-list> (e.g. /audit-all ./plugins/*/, /audit-all packages/foo packages/bar)
---

You are running the four-pass audit (security / performance / safety / cleanup) across multiple directories in parallel, using isolated git worktrees so the agents cannot stomp on each other.

## Setup

Parse the input as either a shell glob (`./plugins/*/`) or a whitespace/comma-separated list of paths.

- Expand globs to a concrete list of directories.
- Drop anything that isn't a directory.
- Drop hidden directories (`.git`, `.github`, `node_modules`, etc.) unless explicitly listed.
- If the resulting list is empty, stop and report what you tried.
- If the list is large (>20), confirm with the user before fanning out.

Print the final list before fanning out so the user can interrupt.

## Fan out

Spawn one Agent per target directory **in a single message** so they run concurrently. For each:

- `subagent_type: general-purpose`
- `isolation: worktree` (so each agent has its own working copy and branch)
- `description`: short, e.g. `Audit <dirname>`
- `prompt`: a fully self-contained brief — the agent does **not** see this conversation. Inline the rules from `commands/audit.md` (or instruct it to read that file first), then tell it to audit `<DIR>` exactly per those rules. End with: "If real fixes exist, branch + commit + push + open a PR and return the PR URL. Otherwise return `No fixes for <DIR>: <one-sentence reason>`. Do not add any AI-attribution trailer or footer to commits or PR bodies."

## Bundle results

When agents return, collect each verdict and print a tight summary:

```
Audited N directories:
  ✓ <dir>  →  <PR URL>
  ✓ <dir>  →  <PR URL>
  ·  <dir>  →  No fixes (<reason>)
  ·  <dir>  →  No fixes (<reason>)

M PRs opened, K directories clean.
```

Do not try to combine the PRs. Each PR stays independent so the user can review and merge them on their own cadence.

## Rules

- Each agent works in its own worktree — they cannot conflict.
- No AI-attribution trailers or footers in any commit or PR body (the global commit-message hook will block these anyway, but say it explicitly so agents don't waste a retry).
- Same conservatism as `/audit`: no churn, no behaviour changes, no hypothetical-future refactors, respect existing idioms.
- Do **not** run agents serially. The whole point is parallelism — single message, multiple Agent calls.

## Don't

- Don't audit directories outside the repo.
- Don't merge worktrees back manually — each agent's PR is the deliverable.
- Don't aggregate findings into a single PR; the user wants them separable.
