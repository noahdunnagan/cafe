---
name: open-pr
command: pr
description: ☕️ cafe · Open a pull request for the current branch — writes the title and body from the diff, pushes if needed. Usage: /pr
---

You are opening a pull request for the current branch. Open it, report the URL, stop.

## Prerequisites

`gh`, authed. Nothing else.

## Steps

1. Resolve the default branch: `gh repo view --json defaultBranchRef -q .defaultBranchRef.name`.
2. If a PR already exists for this branch (`gh pr view --json url -q .url`), report its URL and stop — don't open a second one.
3. If the branch has no upstream, push it: `git push -u origin <branch>`. If it has one but is ahead, push.
4. Build the title and body from `git log <default>..HEAD` and `git diff <default>...HEAD` — read both in full, cover every commit, not just the latest.
   - **Title** — the latest commit's subject if it's conventional and descriptive; otherwise one short line summarizing the diff.
   - **Body** — short. Compress, never transcribe.
     - One or two sentences: what changed and why.
     - Bullets only if the diff spans genuinely distinct areas — one line each, five max.
     - A test plan only when a reviewer must actually run something. Skip the ceremonial checklist.
     - No file-by-file inventory, no section headings on a three-line body.
     - Aim under 120 words. A reviewer should know what they're looking at in ten seconds.
   - **No AI-attribution footer** — no `🤖 Generated with [Claude Code]`, no `Co-Authored-By: Claude`.
5. `gh pr create --base <default> --title "..." --body "..."` — use a HEREDOC for the body so multi-line content survives.
6. Report the PR URL.

Never open a draft unless the user asked for one. Never add labels, request reviewers, or poll checks.
