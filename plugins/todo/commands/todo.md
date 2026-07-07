---
name: todo
command: todo
description: "[cafe] File a terse GitHub issue for a task. Title carries the todo, body only when the title can't. Usage: /todo <task> [repo or project]"
---

You are filing a todo. Activate the todo skill.

The user's request follows. It may name a repo or project; if not, use the current directory's repo. "Have <name> do this" means assign that person, resolved per the skill. Extract the task, write the title-first issue per the skill, create it with `gh`, and reply with the issue URL. Several tasks in one request means one issue each.
