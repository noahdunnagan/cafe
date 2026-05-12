---
name: clog
description: Search Claude Code chat history (past sessions, threads, conversations) via the `clog` CLI. Use when the user asks to find an old session, recall what was discussed about something, look up which branch or repo a chat was on, count sessions by topic, or pull message text from an archived conversation. Triggers include "find that session", "search my chats", "what did we discuss about X", "look up the conversation where", "which chat covered X", "claude history", "past sessions", "remember when we worked on", "find sessions about".
allowed-tools:
  - Bash
---

# clog

`clog` indexes every Claude Code session on this machine (`~/.claude/projects/<encoded-cwd>/<sessionId>.jsonl`) and exposes search, listing, metadata, and stats. Default output is JSONL, one object per line, designed to pipe into `jq`.

## When to reach for it

Use `clog` when the user wants information *about* past Claude conversations, not about code or files.

- "What did I work on yesterday?" → `clog list --limit 10 --pretty`
- "Find the session where I was debugging X." → `clog search "X"`
- "Which branch was that chat on?" → `clog show <id-prefix>`
- "How many sessions per repo?" → `clog stats`
- "Pull the transcript of that auth bypass investigation." → `clog show <id> --messages`

It cannot see the *current* session. Sessions become searchable once their writes flush to disk.

## Commands

### `clog list`

```
clog list [--cwd <substr>] [--branch <substr>] [--title <substr>] [--oldest] [-l <n>]
```

Newest first by default. All filters are substring matches. `--title` is case-insensitive. Combine freely.

### `clog search`

```
clog search <regex> [--case-sensitive] [--cwd <substr>] [--branch <substr>] [-l <n>]
```

Regex over user and assistant message bodies. Case-insensitive by default. Returns up to 3 snippets per matching session. `--limit` caps total sessions (default 50).

### `clog show`

```
clog show <id-or-prefix> [--messages]
```

Prefix match on session id. Errors if ambiguous and lists the candidates. `--messages` dumps full user and assistant text chronologically.

### `clog stats`

```
clog stats
```

Totals plus session counts grouped by cwd and branch.

## Output shapes

All commands emit JSONL on stdout by default. `--pretty` switches to human-readable tables and colors. Never parse `--pretty`.

`list` and `show` emit one `Session` per line:

```json
{"session_id":"...","path":"...","cwd":"...","git_branch":"...","ai_title":"...","first_ts":"...","last_ts":"...","message_count":42}
```

`search` flattens `Session` and adds `snippets`:

```json
{"session_id":"...","cwd":"...","ai_title":"...","snippets":[{"role":"user","timestamp":"...","text":"…match context…"}]}
```

`show --messages` emits the session header then one `Message` per line:

```json
{"role":"user","timestamp":"...","text":"..."}
```

`stats` emits a single object:

```json
{"sessions":122,"messages":29701,"by_cwd":{...},"by_branch":{...}}
```

## Recipes

Always pipe JSONL through `jq` for clean extraction.

Find every session on a branch, formatted for quick scanning:

```sh
clog list --branch noah/discord-commands \
  | jq -r '"\(.session_id[:8])  \(.ai_title // "-")"'
```

Search for a phrase scoped to one repo:

```sh
clog search 'authentication bypass' --cwd mono \
  | jq -r '"\(.session_id[:8])  \(.ai_title // "-")"'
```

Pull the full transcript of the most recent session:

```sh
id=$(clog list --limit 1 | jq -r .session_id)
clog show "$id" --messages \
  | jq -r 'select(.role) | "=== \(.role) @ \(.timestamp) ===\n\(.text)\n"'
```

Top repos by session count:

```sh
clog stats | jq '.by_cwd | to_entries | sort_by(-.value) | .[:10]'
```

Recover the message text where a regex hit fired:

```sh
clog search 'TanStack.*supply.chain' \
  | jq -r '.snippets[] | "[\(.role) @ \(.timestamp)] \(.text)"'
```

## Common mistakes

Do not grep the JSONL files directly. They contain hook output, tool results, and raw JSON envelopes. `clog` strips that and gives you clean message text.

Do not use `--pretty` when piping to anything but a terminal. It emits ANSI codes.

Do not pass a full UUID to `show` when a prefix works. 8 characters is plenty unique.

Do not assume the current session is searchable. It is not, until its writes have flushed.

## Workflow for "find that old chat where..."

1. `clog search '<distinctive phrase>'` to narrow candidates.
2. Skim `session_id`, `ai_title`, `cwd`, `git_branch` from the JSON.
3. `clog show <prefix>` for metadata, or `clog show <prefix> --messages` for the full transcript.
4. If the phrase is too noisy, add `--cwd` or `--branch` filters and re-run.
