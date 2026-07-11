# cafe

Skills and commands for [Claude Code](https://docs.anthropic.com/en/docs/claude-code).

## What's included

| Plugin | Type | Description |
|--------|------|-------------|
| `blueprint` | Skill + Command | Three-mode planning — always-active disposition, `/blueprint` for generating requirements docs, and blueprint execution. |
| `workflow` | Commands | `/push` for conventional commits, `/session` for session logging. |
| `rust-guide` | Skill | Opinionated Rust style guide — makes AI-written Rust code look like a human wrote it. |
| `codex` | Skill | Delegate research, code review, and adversarial sparring to OpenAI's Codex CLI. Auto-selects the best/cheapest available model — no version to maintain. |
| `glm` | Skill | Legacy explicit-only delegation instructions for the sunset GLM model. Never auto-routes frontend or other work. Needs the `glm` shell function and a still-available endpoint. |
| `distill` | Skill + Command | Rigorous code refactoring. Every line earns its place. Preserves functionality, cuts complexity. |
| `parallel` | Skill + Command | Launch and track parallel work in isolated git worktrees. Prevents duplicates, manages branches. |
| `claude-review` | Commands | `/setup-review` installs the Claude Code Review GitHub Action into a repo (label-gated, non-intrusive). `/pr` runs the review loop. |
| `clog` | Skill | Teaches Claude to search your Claude Code chat history via the [`clog`](https://github.com/noahdunnagan/clog) CLI. Auto-invokes for past-session lookups. |
| `tech-stack` | Skill | The canonical, opinionated tech stack for new projects — TanStack web, Rust backends, Railway-first, self-hosted data. Always active when choosing technology. |
| `fable` | Skill | Get the most out of Claude Fable 5 — Fable architects, cheaper models execute. Delegation ladder, workflow model-override gotchas, effort tuning, API reference. |
| `todo` | Skill + Command | Turn "this needs to be done" into a terse GitHub issue on any repo. Title carries the todo, body only when the title can't. |
| `plainspeak` | Skill + Hook | Conversational style. Kills AI reply patterns. No filler openers, hedge stacks, recap closers, sycophancy, formatting theater, or jargon. A SessionStart hook injects it into every session, so it's always on with zero invocation. |

## Install

Add the marketplace and install what you want:

```
/plugin marketplace add noahdunnagan/cafe
/plugin install blueprint@cafe
/plugin install workflow@cafe
/plugin install rust-guide@cafe
/plugin install codex@cafe
/plugin install glm@cafe
/plugin install distill@cafe
/plugin install parallel@cafe
/plugin install claude-review@cafe
/plugin install clog@cafe
/plugin install tech-stack@cafe
/plugin install fable@cafe
/plugin install todo@cafe
/plugin install plainspeak@cafe
```

The `clog` plugin assumes the `clog` binary is on your `$PATH`. See [noahdunnagan/clog](https://github.com/noahdunnagan/clog) for install.

## Use with other AI agents (Codex, Cursor, Copilot, Gemini, Zed, …)

The skills use the cross-vendor [`SKILL.md`](https://agents.md) format, so they work
in far more than Claude Code. `install.sh` symlinks every skill and command into
each AI coding agent it finds on your machine — **no signup, no dependencies, no network**:

```
git clone https://github.com/noahdunnagan/cafe && cd cafe
./install.sh            # detects your agents and links everything in
./install.sh --dry-run  # preview first
./install.sh --project .   # per-repo install (needed for Cursor — it has no global skills dir)
```

| Tier | Agents | What they get |
|------|--------|---------------|
| **Full** (skills + commands) | Claude Code, Codex CLI, Cursor, GitHub Copilot, Gemini CLI, opencode, Zed, Windsurf, Cline, Kilo | native `SKILL.md` — zero conversion |
| **Instructions** | Aider, + any [AGENTS.md](https://agents.md)-aware tool | `AGENTS.md` as always-on context |

Symlinks point back into the clone, so `git pull` updates every agent at once. Use
`--copy` if your filesystem can't symlink (Windows). See [`AGENTS.md`](AGENTS.md) for details.

## License

MIT
