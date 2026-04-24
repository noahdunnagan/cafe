---
name: Setup Claude Code Review
command: setup-review
description: Install the Claude Code Review GitHub Action into the current repo. Opens a PR with the workflow, then instructs the user to create the `review` label and set the `CLAUDE_CODE_OAUTH_TOKEN` secret. Label-gated so it never auto-reviews.
---

You are installing the Claude Code Review GitHub Action into the user's current repository. This is a one-shot setup. The end state: a PR is open with the workflow file, the user knows how to create the `review` label and the `CLAUDE_CODE_OAUTH_TOKEN` secret, and from then on they trigger reviews by adding the `review` label (usually via `/pr`).

## Design invariants

- **Label-gated only.** The workflow triggers solely on `pull_request: types: [labeled]`, guarded by `github.event.label.name == 'review'`. Never on open/push/synchronize. It must not be intrusive.
- **Manual secret entry.** Never ask the user to paste the OAuth token into the conversation. Always instruct them to set it themselves via the GitHub UI or via `gh secret set` in their own terminal. Do not run `gh secret set` on their behalf.
- **Ask before acting.** Every step that touches their repo, git history, or GitHub (branch creation, commits, pushes, label creation, PR creation) gets confirmed first. This command is not autonomous.

## Flow

### 1. Preflight

- Confirm `pwd` is inside a git repo (`git rev-parse --show-toplevel`).
- Confirm a GitHub remote exists (`gh repo view --json nameWithOwner -q .nameWithOwner`). If `gh` is missing or unauthed, tell the user to install/auth it and stop. `gh` is required for the rest of the flow.
- Note the repo slug (`owner/name`) for later instructions.

### 2. Check for existing workflow

If `.github/workflows/claude-code-review.yml` (or any workflow that uses `anthropics/claude-code-action`) already exists, stop and ask the user whether to overwrite. Do not silently clobber.

### 3. Install the workflow file

Read the template from this plugin's `assets/claude-code-review.yml` (sibling to this command file) and write it to `.github/workflows/claude-code-review.yml` in the target repo. Create `.github/workflows/` if needed.

Do not modify the template content. It is intentionally pinned to the label-only trigger and the full review prompt.

### 4. Commit on a new branch and open a PR

Ask the user before doing these. Default branch name: `chore/add-claude-review`.

1. `git checkout -b chore/add-claude-review`
2. `git add .github/workflows/claude-code-review.yml`
3. Commit with message: `chore: add claude code review workflow`
4. `git push -u origin chore/add-claude-review`
5. `gh pr create` with title `chore: add claude code review workflow` and a body that includes the **post-merge setup steps** below so the user has them in the PR description.

### 5. Tell the user the two manual steps

Print these clearly. The user does them. You do not.

**A. Create the `review` label** (one-time per repo):

```
gh label create review --color FBCA04 --description "Request Claude code review"
```

Or via the web: `https://github.com/<owner>/<repo>/labels` → New label → name `review`.

**B. Generate and set the OAuth token secret** (one-time per repo):

1. In any terminal, run `claude setup-token`. This prints a long-lived OAuth token.
2. Add it to the repo as a secret named `CLAUDE_CODE_OAUTH_TOKEN`. Either:
   - CLI: `gh secret set CLAUDE_CODE_OAUTH_TOKEN` (paste when prompted), or
   - Web: `https://github.com/<owner>/<repo>/settings/secrets/actions` → New repository secret → name `CLAUDE_CODE_OAUTH_TOKEN`, paste value.

Do not offer to run `gh secret set` on their behalf. Do not ask them to paste the token into this conversation.

### 6. Usage hint

Once the label and secret exist and the PR is merged:

- Run `/pr <number>` on any PR to trigger a review. It adds the `review` label, which fires the workflow, and then loops on the resulting comments.
- To trigger manually without the skill: just add the `review` label to a PR in the GitHub UI. Remove + re-add to re-run.

## Failure modes to handle gracefully

- No GitHub remote → stop, explain.
- Workflow already present → ask, don't clobber.
- `gh` unauthed → tell user to run `gh auth login` and retry.
- Dirty working tree → ask whether to stash or abort. Do not silently carry their changes onto the new branch.
- User declines any prompt → stop cleanly. Leave the working tree as you found it where possible.
