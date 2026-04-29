---
name: PR Review Loop
command: pr
description: Run the full PR review loop. If no PR exists for the current branch, opens one first. Adds the `review` label, polls for the code review check, reads comments, fixes or replies, resolves threads, pushes, repeats until clean. Usage: /pr <number> or /pr (auto-detects current branch, opens PR if needed). Requires the workflow from /setup-review to be installed.
---

You are running the PR review loop for this repository. This is a fully automated cycle.

## Prerequisites

This command assumes the code-review workflow is installed in the repo and the `CLAUDE_CODE_OAUTH_TOKEN` secret is set. If the workflow is missing, run `/setup-review` first.

## Setup

Parse the input:
- If a PR number is given, use it.
- If no number, detect the current branch's PR via `gh pr view --json number -q .number`.
- If no PR exists for the current branch, **create one** (see below) before continuing.

### Creating a PR when none exists

1. Refuse if the current branch is the repo's default branch (`main` / `master` / whatever `gh repo view --json defaultBranchRef -q .defaultBranchRef.name` returns). Tell the user to make a feature branch first — do not create a PR from the default branch into itself.
2. Refuse if there are uncommitted changes. Ask the user to commit (or stash) first; do not silently sweep them in.
3. If the branch has no upstream, push it: `git push -u origin <branch>`.
4. Build the PR title and body:
   - **Title** — use the latest commit's subject if it's conventional and descriptive; otherwise summarize the diff in one short line.
   - **Body** — a Summary section (what changed and why, bullet form) and a Test plan section (checklist). Read `git log <default>..HEAD` and `git diff <default>...HEAD` to write this — cover all commits, not just the most recent.
   - **No AI-attribution footer** — no `🤖 Generated with [Claude Code]`, no `Co-Authored-By: Claude` (the global hook will block these anyway).
5. `gh pr create --title "..." --body "..."` (use a HEREDOC for the body so multi-line content survives).
6. Capture the new PR number and proceed.

Then ensure the PR is marked ready (not draft). Add the `review` label if not already present. Adding the label is what triggers the workflow.

## The Loop

Repeat this cycle until the review passes clean:

### 1. Identify and wait for the review check

Detect the check name from the workflow file rather than hardcoding it:

1. Find the review workflow: `grep -l 'anthropics/claude-code-action' .github/workflows/*.yml 2>/dev/null`. If multiple match, prefer one triggered by `pull_request: types: [labeled]`. If none match, stop and tell the user to run `/setup-review`.
2. Read the matched file. The check name is the job key (the YAML key under `jobs:`). Common values: `code-review`, `claude-review`. Capture it as `$CHECK`.

Then poll `gh pr checks <number>` every 10 seconds. Watch for `$CHECK`.
- If `$CHECK` passes: still fetch comments — a pass with comments needs attention.
- If `$CHECK` fails: check logs with `gh run view` to determine review failure (has comments) vs infra failure (bad credentials, timeout). If infra failure, report and stop.
- If `$CHECK` is "skipping": a second run was queued while the first was running. Wait for the non-skipping run.

### 2. Read all review comments

Fetch comments via `gh api repos/{owner}/{repo}/pulls/{number}/comments`.

For each comment:
- Read the comment body, file path, and line number.
- Read the surrounding code to understand context.
- Decide: is this a legitimate issue (security, correctness, real bug) or a style nit?

### 3. Address each comment

For legitimate issues:
1. Fix the code.
2. Reply to the comment explaining what you changed (keep it brief).
3. Resolve the thread via `gh api graphql` (find the thread ID, mark resolved).

For style nits or things you're skipping:
1. Reply briefly explaining why you're skipping it.
2. Resolve the thread.

Never leave a comment without a reply. Never leave a thread unresolved.

### 4. Push and repeat

After addressing all comments:
1. Stage and commit the fixes (clear commit message referencing the review). Do **not** add a `Co-Authored-By: Claude` trailer or a `🤖 Generated with [Claude Code]` footer — these commits are the user's authorship. Same rule for any PR body edits you make in this loop.
2. Push to the branch.
3. The workflow only fires on label events, so to request another review cycle: remove the `review` label and re-add it (`gh pr edit <num> --remove-label review && gh pr edit <num> --add-label review`). Then go back to step 1.

### 5. Exit conditions

Stop the loop when:
- A review cycle produces zero new comments (clean pass).
- The review check passes with no new comments.
- An infra error prevents the review from running (report to user).
- Three consecutive cycles with the same unresolvable comments (report to user).

When the loop exits clean, remove the `review` label and report the result.

## Rules

- Work from the PR's branch. Use `git checkout <branch>` if you are not already on it.
- Do not squash or amend commits. Each fix round is its own commit.
- Do not merge the PR. Just report that it's clean and ready.
- If the review action has auth issues (bad credentials), stop immediately and tell the user to regenerate `CLAUDE_CODE_OAUTH_TOKEN` via `claude setup-token` and update the repo secret.
- Be efficient with `gh api` calls. Batch where possible.
