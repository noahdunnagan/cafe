---
name: PR Review Loop
command: pr
description: Run the full PR review loop. Adds the `review` label, polls for Claude Code Review, reads comments, fixes or replies, resolves threads, pushes, repeats until clean. Usage: /pr <number> or /pr (auto-detects current branch PR). Requires the workflow from /setup-review to be installed.
---

You are running the PR review loop for this repository. This is a fully automated cycle.

## Prerequisites

This command assumes the `claude-review` workflow is installed in the repo and the `CLAUDE_CODE_OAUTH_TOKEN` secret is set. If the workflow is missing, run `/setup-review` first.

## Setup

Parse the input:
- If a PR number is given, use it.
- If no number, detect the current branch's PR via `gh pr view --json number -q .number`.
- If no PR exists, stop and tell the user.

Ensure the PR is marked ready (not draft). Add the `review` label if not already present. Adding the label is what triggers the workflow.

## The Loop

Repeat this cycle until the review passes clean:

### 1. Wait for Claude Code Review

Poll `gh pr checks <number>` every 10 seconds. Watch for the `claude-review` check.
- If it passes: check for comments anyway (a pass with comments still needs attention).
- If it fails: check logs with `gh run view` to determine if it's a review failure (has comments) or an infra failure (bad credentials, timeout, etc). If infra failure, report to the user and stop.
- If it's "skipping": a second run was triggered while the first was still going. Wait for the non-skipping run.

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
1. Stage and commit the fixes (clear commit message referencing the review).
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
