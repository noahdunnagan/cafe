---
name: Parallel
command: parallel
description: Launch and track parallel work in isolated git worktrees. Usage: /parallel <task>, /parallel status, /parallel merge <id>, /parallel cancel <id>, /parallel clean.
---

You are managing parallel work streams. Activate the parallel skill.

Parse the input after `/parallel` to determine which subcommand to run:

- No subcommand or a task description → launch new parallel task
- `status` → show all tracked work
- `merge <id>` → merge completed branch back
- `cancel <id>` → kill and clean up a task
- `clean` → prune finished entries

Follow the parallel skill instructions for the matched subcommand.
