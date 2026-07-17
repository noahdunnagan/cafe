# cafe

Skills and slash commands for AI coding agents, shipped as plain files over git —
no account, no hosted service, no API key.

The skills use the cross-vendor [`SKILL.md`](https://agents.md) format, so one copy
works across **Claude Code, Codex, Cursor, Copilot, Gemini CLI, opencode, Zed,
Windsurf, Cline, and Kilo**.

## Skills

| Plugin | Type | Description |
|--------|------|-------------|
| `blueprint` | Skill + Command | Three-mode planning — always-active disposition, `/blueprint` for requirements docs, and blueprint execution. |
| `rust-guide` | Skill | Opinionated Rust style guide — makes AI-written Rust look like a human wrote it. |
| `distill` | Skill + Command | Rigorous refactoring. Every line earns its place; behavior preserved, complexity cut. |
| `codex` | Skill | Delegate research, review, and adversarial sparring to OpenAI's Codex CLI. Auto-selects the best/cheapest model. |
| `fable` | Skill | Get the most out of Claude Fable 5 — Fable architects, cheaper models execute. |
| `tech-stack` | Skill | The canonical, opinionated stack for new projects — TanStack web, Rust backends, Railway-first. |
| `parallel` | Skill + Command | Launch and track parallel work in isolated git worktrees. |
| `plainspeak` | Skill + Hook | Kills AI reply patterns — no filler openers, hedge stacks, recap closers, or jargon. |
| `clog` | Skill | Search your Claude Code chat history via the [`clog`](https://github.com/noahdunnagan/clog) CLI. |
| `todo` | Skill + Command | Turn "this needs doing" into a terse GitHub issue on any repo. |
| `claude-review` | Commands | `/setup-review` installs a label-gated review Action; `/pr` runs the review loop; `/audit` audits a dir. |
| `workflow` | Commands | `/push` for conventional commits, `/session` for session logging. |
| `glm` | Skill | Legacy explicit-only delegation for the sunset GLM model. |

## Install — the `cafe` CLI (any agent)

The `cafe` CLI detects every agent on your machine and links the skills in. Browse
them with descriptions, pick a subset, update, or uninstall — all interactive.

```sh
git clone https://github.com/noahdunnagan/cafe && cd cafe
cargo install --path cli
cafe                # interactive menu
```

| Command | Does |
|---------|------|
| `cafe install` | Browse skills, pick agents, link them in |
| `cafe list` | Every skill with its description |
| `cafe update` | `git pull` — refreshes every linked agent at once |
| `cafe uninstall` | Remove cafe's links (leaves your own files untouched) |

Skills install as symlinks back into the checkout, so a single `cafe update`
reaches every agent. Requires [Rust](https://rustup.rs); Unix-only (macOS/Linux).

## Install — Claude Code plugins

Claude Code can install directly from the plugin marketplace, which also wires up
hooks like plainspeak's always-on SessionStart:

```
/plugin marketplace add noahdunnagan/cafe
/plugin install blueprint@cafe      # …or any plugin from the table above
```

## Conductor

[Conductor](https://conductor.build) runs **Claude Code and Codex** — it has no
skills store of its own, so it surfaces whatever those agents load. Installing
into Claude Code / Codex (either method above) is all it takes; cafe's skills
appear in Conductor's `/` menu automatically. Pick *one* install method, though —
using the CLI and the plugin marketplace together shows every command twice.

## Other agents

Agents without `SKILL.md` support (e.g. Aider) read [`AGENTS.md`](AGENTS.md)
directly — add it to their instruction file for the same guidance as always-on
context. The `clog` plugin needs the [`clog`](https://github.com/noahdunnagan/clog)
binary on your `$PATH`.

## License

MIT
