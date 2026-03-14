---
name: parallel
description: Launch and track parallel work streams in isolated git worktrees. Prevents duplicate work, tracks active branches, and manages the lifecycle of concurrent tasks. Use when user says "/parallel", "do this in parallel", "work on this separately", or wants to run independent tasks without touching the working tree.
user-invocable: true
disable-model-invocation: false
allowed-tools:
  - Bash
  - Read
  - Write
  - Edit
  - Glob
  - Grep
  - Agent
---

# Parallel

Run independent tasks in isolated worktrees. Track what's running. Prevent duplicates.

## State

All state lives in `.parallel/` at the repo root (gitignored).

```
.parallel/
  manifest.json    # array of tracked work items
```

Each entry in the manifest:

```json
{
  "id": "<short-id>",
  "task": "<one-line description>",
  "branch": "<branch-name>",
  "worktree": "<worktree-path>",
  "status": "running" | "done" | "merged",
  "created": "<ISO timestamp>",
  "agentId": "<agent-id if available>"
}
```

## Commands

Parse the user's input after `/parallel`:

### `/parallel <task description>`

Launch a new parallel task.

1. **Read the manifest.** If `.parallel/manifest.json` exists, load it. Otherwise initialize empty array.
2. **Check for duplicates.** Compare the task description against existing `running` entries. If something looks like the same work (fuzzy match on description), warn the user and ask to confirm before proceeding.
3. **Generate identifiers.** Create a short kebab-case branch name from the task (e.g., `parallel/presign-urls`). Generate a 4-char random ID.
4. **Ensure `.parallel/` is gitignored.** Check `.gitignore` for `.parallel/` entry. Add it if missing.
5. **Launch the agent.** Use the Agent tool with `isolation: "worktree"` and `run_in_background: true`. Pass the full task description as the prompt. Include context about the repo and what files are likely relevant.
6. **Record in manifest.** Add the entry with status `running` and the agent ID.
7. **Confirm to user.** Print the task, branch name, and ID. Keep it brief.

### `/parallel status`

Show all tracked work.

1. Read the manifest.
2. For each `running` entry, check if the worktree/branch still exists (`git worktree list`, `git branch`).
3. Print a table: ID, status, branch, task description.
4. If any entries have stale worktrees (worktree deleted but status still `running`), mark them `done` and note the branch.

### `/parallel merge <id or branch>`

Merge completed work back.

1. Find the entry by ID or branch name.
2. If worktree still exists, warn that work may still be in progress. Ask to confirm.
3. Run `git merge <branch>` from the main working tree.
4. Update status to `merged` in manifest.
5. Clean up: remove the worktree if it still exists (`git worktree remove`).

### `/parallel clean`

Remove all `done` and `merged` entries from the manifest. Prune any orphaned worktrees.

### `/parallel cancel <id or branch>`

1. Find the entry.
2. Remove the worktree (`git worktree remove --force`).
3. Delete the branch (`git branch -D`).
4. Remove from manifest.

## Agent Prompt Template

When launching the worktree agent, use this prompt structure:

```
You are working on an isolated task in a git worktree. Your changes will be on a separate branch.

## Task
<user's task description>

## Context
- Repo: <repo root>
- Base branch: <current branch>
- Your branch: <generated branch name>

## Instructions
- Focus only on the described task. Do not make unrelated changes.
- Commit your work with clear commit messages when done.
- If you need clarification, stop and report what you need. Do not guess.
```

## Rules

- Always check the manifest before launching. Duplicates waste compute.
- Branch names use `parallel/` prefix for easy identification.
- The manifest is the source of truth. If a worktree is gone but the branch exists, the work was completed.
- Never touch the user's working tree. That's the whole point.
- Keep status output compact. One line per item.
