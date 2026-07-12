---
name: todo
description: cafe · File a terse GitHub issue to track a task on any repo. Use when the user says "/todo", "this needs to be done", "file a todo", "track this", "make an issue for this", or points at work worth remembering later.
user-invocable: true
disable-model-invocation: false
---

# Todo

A todo is a title, not a document. Turn "this needs to be done" into a GitHub issue that reads like a note from a sharp engineer. The failure mode is verbosity. Every rule below exists to prevent it.

## Format

**The title is the todo.** Imperative, specific, names the object and the change. Under 70 characters, no trailing period.

Good: `Add backoff retry to S3 uploader`
Good: `Pin tokio to 1.40 in workspace Cargo.toml`
Bad: `Improve upload reliability` (names no change)
Bad: `Fix issue where uploads sometimes fail due to transient errors` (rambling)

**The body is empty by default.** Add a line only when the title cannot carry it. Three line types exist, all optional, one line each:

```
Done: transient 5xx retries 3x with jitter, hard failures still surface
Where: src/upload/s3.rs
Ref: #142
```

A genuinely multi-step task gets a checklist instead, 3 to 6 items, each one line:

```
- [ ] Wrap put_object in retry loop, max 4 attempts
- [ ] Retry only 5xx and timeouts, never 4xx
- [ ] Test: 503 twice then 200 still succeeds
```

**Hard rules:**

- Never restate the title in the body.
- No headers, no sections, no "Description", no "Acceptance criteria". It is a todo, not a PRD.
- If the title says it all, the body stays empty. Most todos should end up body-empty.
- One issue per task. A request holding several tasks gets several issues.
- No labels unless the user named one and it already exists on the repo. Never invent label taxonomy.

## Assignment

"Have Jace do this" means assign Jace. Resolve the name to a GitHub login:

1. A handle you already know from context or memory: use it.
2. Otherwise match the name against the repo's assignable users: `gh api repos/<owner>/<repo>/assignees --jq '.[].login'`.
3. The requester assigning themselves is `--assignee @me`.
4. Ambiguous or no match: ask, do not guess.

Pass it as `--assignee <login>` on create. If GitHub rejects the assignment (not a collaborator), add one body line instead: `cc @<login>`. Assignment never adds any other text to the issue.

## Process

### 1. Resolve the repo

In order:

1. Explicit `owner/repo` in the request: use it.
2. A project name ("transmit"): look for a local clone at `~/Documents/GitHub/<name>` and read its origin remote, else match against `gh repo list --json name --limit 100`.
3. Nothing named: the current directory's repo via `gh repo view --json nameWithOwner`.
4. None resolve: ask which repo.

### 2. Skip dupes

```sh
gh issue list --repo <repo> --state open --search "<keywords>"
```

If an open issue already covers it, reply with that issue's URL and stop. Do not file a duplicate.

### 3. File it

```sh
gh issue create --repo <repo> --title "<title>" --body "<body, or empty string>" [--assignee <login>]
```

### 4. Report

One line per issue: the URL. Nothing else.
