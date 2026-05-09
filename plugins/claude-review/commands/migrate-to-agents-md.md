---
name: Migrate to AGENTS.md
command: migrate-to-agents-md
description: Rename every `CLAUDE.md` in the repo to the vendor-neutral `AGENTS.md` convention, symlink `CLAUDE.md → AGENTS.md` so Claude Code still finds them, and update doc references. One commit, mechanical.
---

You are migrating a repository from Claude-specific `CLAUDE.md` config files to the vendor-neutral `AGENTS.md` convention. End state: every `CLAUDE.md` is renamed to `AGENTS.md`, a relative symlink at the original path keeps Claude Code working, and any docs that reference the filename are updated.

## Preflight

1. Confirm `pwd` is the root of a git repo.
2. Confirm the working tree is clean (`git status --porcelain` returns nothing). If not, ask the user whether to stash or abort. Do not silently carry their changes onto the migration commit.
3. List every tracked `CLAUDE.md` file: `git ls-files | grep -E '(^|/)CLAUDE\.md$'`. If none, report "No CLAUDE.md files found" and stop.

Print the file list and the planned reference updates (next step) before doing anything destructive.

## Plan reference updates

Find every tracked file that contains the literal string `CLAUDE.md`, excluding the files about to be renamed:

```
git grep -l 'CLAUDE\.md' -- ':!*/CLAUDE.md' ':!CLAUDE.md'
```

For each match, decide whether the reference should be rewritten:
- **Yes** — file-tree diagrams, doc body text, comments pointing readers to the file, code that reads the path.
- **No** — `CHANGELOG.md` historical entries, archived migration notes, anything where the old name is part of the record. When in doubt, leave it.

Show the user the planned rewrites (file + line) before applying.

## Apply

Ask the user for confirmation, then:

1. For each `CLAUDE.md`:
   - `git mv <path>/CLAUDE.md <path>/AGENTS.md`
   - `ln -s AGENTS.md <path>/CLAUDE.md` (relative symlink — works on every clone, no absolute path baked in)
   - `git add <path>/CLAUDE.md`
2. Apply the planned reference updates (literal `CLAUDE.md` → `AGENTS.md`).
3. `git status` to confirm the diff matches the plan.

## Commit

Single conventional commit:

- title: `chore: migrate CLAUDE.md → AGENTS.md`
- body: list of files renamed + reference updates, grouped

Do **not** add a `Co-Authored-By: Claude` trailer or a `🤖 Generated with [Claude Code]` footer. (The global hook will block these anyway, but state it so the commit doesn't need a retry.)

## Verify

After the commit:
- `cat AGENTS.md` works at every renamed path.
- `cat CLAUDE.md` (the symlink) returns the same content.
- `git log --follow AGENTS.md` shows the original history (rename detection should keep blame intact).

Report the final state and the number of files renamed + references updated.

## Don't

- Don't migrate `~/.claude/CLAUDE.md` or any path outside the current repo.
- Don't migrate files whose name only happens to contain `CLAUDE.md` (e.g. `CLAUDE.md.bak`) — the find command above is anchored.
- Don't update historical changelog entries that mention the old name.
- Don't push the commit — the user pushes when ready.
