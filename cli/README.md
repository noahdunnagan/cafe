# cafe CLI

A friendly installer for cafe's skills across every AI coding agent on your
machine. Browse skills with their descriptions, pick a subset, update, and
uninstall cleanly.

## Install

```
cargo install --path cli      # from the cafe checkout
```

The binary remembers the checkout it was built from, so `cafe` works from
anywhere. Moved the clone? Set `CAFE_HOME=/path/to/cafe`.

## Use

```
cafe            # interactive menu
cafe install    # browse skills, pick agents, link them in
cafe list       # every skill + description
cafe update     # git pull — refreshes every linked agent at once
cafe clean      # remove dead links left by renamed/removed skills
cafe uninstall  # remove cafe's links (leaves your own files alone)
```

Skills install as symlinks back into the checkout, so a single `cafe update`
reaches every agent. Re-running `cafe install` is safe — links have fixed names,
so it refreshes in place instead of duplicating, and self-prunes dead links.
`cafe clean` does that prune on demand across every agent.

## Conductor

Conductor runs Claude Code & Codex and has no skills dir of its own, so
installing into those already reaches its `/` menu — the CLI detects Conductor
and confirms this after installing. Don't also install cafe via `/plugin`, or
each command shows up twice.

## Scope

Detects each agent and symlinks skills + commands into its dir, never clobbering
a real file. Skills + commands only — Claude plugin hooks (e.g. plainspeak's
always-on SessionStart) still need `/plugin install …@cafe`. Unix-only
(macOS/Linux). Cursor has no global skills dir, so it's per-project only — not
yet handled by the CLI.
