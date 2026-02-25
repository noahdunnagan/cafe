---
name: codex
description: Delegate research or code review to Codex CLI. Use when user says "/codex", "ask codex", "delegate to codex", "have codex look into", "codex review", "use codex to review", "have codex review", "codex research", or wants fast parallel research or code review using OpenAI's Codex agent.
allowed-tools:
  - Bash
---

# Codex

Delegate read-only tasks to OpenAI's Codex CLI. Two modes: fast research (spark) and deep code review (full 5.3).

## Modes

### Research (default)

Fast, shallow research using codex-spark. Good for codebase questions, lookups, and exploration.

```
codex exec \
  -m gpt-5.3-codex-spark \
  -c model_reasoning_effort="low" \
  --ephemeral \
  -s read-only \
  -o /tmp/codex-<hash>.md \
  -C <working-dir> \
  "<prompt>"
```

### Review

Deep code review using the full gpt-5.3-codex model with high reasoning effort. Triggered when the user's prompt starts with `review` (after `/codex`).

```
codex exec \
  -m gpt-5.3-codex \
  -c model_reasoning_effort="high" \
  --ephemeral \
  -s read-only \
  -o /tmp/codex-<hash>.md \
  -C <working-dir> \
  "<prompt>"
```

The review prompt is constructed as:
- If user specifies files: `"Review the following files for bugs, logic errors, performance issues, and style: <files>. <any additional user instructions>"`
- If no files specified: `"Review the codebase for bugs, logic errors, performance issues, and style. <any additional user instructions>"`

## Routing

Parse the user's input after `/codex`:

1. **Starts with `review`** → Review mode (gpt-5.3-codex, high reasoning)
2. **Contains `-m <model>`** → Research mode with model override
3. **Everything else** → Research mode (gpt-5.3-codex-spark, low reasoning)
4. **Empty prompt** → Ask user what to do

## Common Flags

- `--ephemeral` — no session persistence
- `-s read-only` — no file mutations
- `-o <file>` — capture final response for clean extraction
- `-C` — set working directory (defaults to current project root)

After execution, read the output file and present the findings directly. Delete the temp file.

## Examples

### Research

User: `/codex how does the router package handle TLS termination?`

```bash
codex exec -m gpt-5.3-codex-spark -c model_reasoning_effort="low" \
  --ephemeral -s read-only -o /tmp/codex-abc123.md \
  "how does the router package handle TLS termination?"
```

### Research with model override

User: `/codex -m gpt-5.3-codex how does deployment rollback work?`

Extract `-m gpt-5.3-codex` from the prompt. Run with that model instead of spark.

### Review (whole project)

User: `/codex review`

```bash
codex exec -m gpt-5.3-codex -c model_reasoning_effort="high" \
  --ephemeral -s read-only -o /tmp/codex-def456.md \
  "Review the codebase for bugs, logic errors, performance issues, and style."
```

### Review (specific files)

User: `/codex review src/main.rs src/router.rs`

```bash
codex exec -m gpt-5.3-codex -c model_reasoning_effort="high" \
  --ephemeral -s read-only -o /tmp/codex-ghi789.md \
  "Review the following files for bugs, logic errors, performance issues, and style: src/main.rs src/router.rs"
```

### Review with extra instructions

User: `/codex review src/api/ focus on error handling`

```bash
codex exec -m gpt-5.3-codex -c model_reasoning_effort="high" \
  --ephemeral -s read-only -o /tmp/codex-jkl012.md \
  "Review the following files for bugs, logic errors, performance issues, and style: src/api/. focus on error handling"
```

### Parallel research

When the user provides multiple distinct questions, or when the research naturally decomposes into independent sub-questions, run multiple `codex exec` calls concurrently using parallel Bash tool calls.

## Error Handling

```
case:
  empty prompt          -> ask user what to research or review
  codex not installed   -> tell user to install: npm i -g @openai/codex
  auth failure          -> tell user to run: codex login
  timeout (>120s)       -> kill process, report partial output if any
  model not available   -> fall back to default codex model, inform user
```
