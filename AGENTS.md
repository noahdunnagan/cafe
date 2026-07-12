# cafe

A collection of **skills** and **slash commands** for AI coding agents, shipped as
plain files over git — no account, no hosted service, no API key.

The skills use the cross-vendor `SKILL.md` format (a folder + a markdown file with
`name`/`description` frontmatter), which Claude Code, Codex, Cursor, Copilot,
Gemini CLI, opencode, Zed, Windsurf, Cline, Kilo, Amp and others read natively.

## Install into your agents

```sh
git clone <this repo> && cd cafe
cargo install --path cli   # then run `cafe` to browse and link skills into every agent
```

Skills install as symlinks back into this checkout, so `cafe update` (or
`git -C <cafe> pull`) refreshes every agent at once. Requires Rust; Unix-only.

## Skills (auto-invoke by description; also runnable as `/name`)

- **rust-guide** — opinionated Rust style; makes AI-written Rust look human-written. Always-on when writing Rust.
- **blueprint** — three-mode planning: always-on disposition, `/blueprint` requirements docs, and blueprint execution.
- **distill** — rigorous refactoring; every line earns its place, behavior preserved.
- **codex** — delegate research / review / sparring to the Codex CLI (auto-selects the best/cheapest Codex model).
- **parallel** — launch and track parallel work in isolated git worktrees.
- **clog** — search your Claude Code chat history via the `clog` CLI.
- **todo** — file a terse GitHub issue for a task on any repo; title carries the todo, body only when the title can't.

## Commands

`/push` · `/session` · `/blueprint` · `/distill` · `/parallel` · `/pr` · `/setup-review` · `/audit` · `/audit-all` · `/migrate-to-agents-md` · `/todo`

> Agents without `SKILL.md` support (e.g. Aider) read this file directly — add it to
> Aider's `read:` list to get the skill guidance as always-on context.
