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
```

## License

MIT
