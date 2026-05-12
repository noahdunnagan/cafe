# cafe

Skills and commands for [Claude Code](https://docs.anthropic.com/en/docs/claude-code).

## What's included

| Plugin | Type | Description |
|--------|------|-------------|
| `blueprint` | Skill + Command | Three-mode planning — always-active disposition, `/blueprint` for generating requirements docs, and blueprint execution. |
| `workflow` | Commands | `/push` for conventional commits, `/session` for session logging. |
| `rust-guide` | Skill | Opinionated Rust style guide — makes AI-written Rust code look like a human wrote it. |
| `codex` | Skill | Delegate read-only research to OpenAI's Codex CLI using the fast codex-spark model. |
| `distill` | Skill + Command | Rigorous code refactoring. Every line earns its place. Preserves functionality, cuts complexity. |
| `parallel` | Skill + Command | Launch and track parallel work in isolated git worktrees. Prevents duplicates, manages branches. |
| `claude-review` | Commands | `/setup-review` installs the Claude Code Review GitHub Action into a repo (label-gated, non-intrusive). `/pr` runs the review loop. |
| `clog` | Skill | Teaches Claude to search your Claude Code chat history via the [`clog`](https://github.com/noahdunnagan/clog) CLI. Auto-invokes for past-session lookups. |

## Install

Add the marketplace and install what you want:

```
/plugin marketplace add noahdunnagan/cafe
/plugin install blueprint@cafe
/plugin install workflow@cafe
/plugin install rust-guide@cafe
/plugin install codex@cafe
/plugin install distill@cafe
/plugin install parallel@cafe
/plugin install claude-review@cafe
/plugin install clog@cafe
```

The `clog` plugin assumes the `clog` binary is on your `$PATH`. See [noahdunnagan/clog](https://github.com/noahdunnagan/clog) for install.

## License

MIT
