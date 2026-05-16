---
name: codex
description: Delegate research, code review, or collaborative sparring to Codex CLI. Use when user says "/codex", "ask codex", "delegate to codex", "have codex look into", "codex review", "spar with codex", "go back and forth with codex", "use codex to pressure-test", or when a problem is non-trivial enough that a second model's perspective would meaningfully improve the answer.
allowed-tools:
  - Bash
---

# Codex

Three modes. Pick the right one.

| Mode | Model | Effort | Use for |
|------|-------|--------|---------|
| `research` | `gpt-5.3-codex-spark` | low | Fast lookups, codebase questions, shallow exploration |
| `review` | `gpt-5.3-codex` | high | Deep code review, bug hunts, audits |
| `spar` | `gpt-5.5` | xhigh | Architecture, design tradeoffs, gnarly bugs, ambiguous requirements. Multi-turn dialogue between Claude and Codex |

## When to reach for which

- **Default for non-trivial problems → `spar`.** Architecture decisions, "should we use X or Y", subtle bugs where the cause isn't obvious, design tradeoffs, ambiguous requirements. If you'd want a peer to push back on your first instinct, use spar.
- **Lookups and small questions → `research`.** "How does X work?" "Where is Y defined?" Single-shot, cheap, fast.
- **Whole-file or whole-codebase audits → `review`.**

Lean toward `spar` more often than not. A second model with different blind spots catches things solo reasoning misses.

---

## Mode: `spar` (collaborative multi-turn)

Claude and Codex go back and forth until they converge or surface a real disagreement. Both models are peers. Neither defers to the other on authority alone.

### How it works

1. **Open**. Claude states the problem and its initial take, framed adversarially. Captures the session id from the first response.
2. **Loop, streamed live**. Each round prints:
   - `── Round N → Codex ──` followed by Claude's prompt
   - `── Round N ← Codex ──` followed by Codex's response
3. **Self-judged convergence**. No hard cap. Exit when:
   - Genuine agreement reached, or
   - Surviving disagreement is clearly explained (don't paper over it), or
   - Same point would be repeated (Claude or Codex looping), or
   - Sanity backstop: ~8 rounds without convergence. Call it and surface the impasse.
4. **Synthesis**. Print `── Synthesis ──` with the joint position. Call out what survived as disagreement, honestly. The synthesis is the deliverable, not a summary of who said what.

### Mechanics

**First round** (captures session id):

```bash
codex exec \
  -m gpt-5.5 \
  -c model_reasoning_effort=xhigh \
  -c service_tier=fast \
  -s read-only \
  -C <cwd> \
  -o /tmp/codex-spar-<id>-r1.md \
  "<framed opening prompt>"
```

Parse the session id from the codex output. Codex prints it near the start of execution.

**Subsequent rounds**:

```bash
codex exec resume <session_id> \
  -o /tmp/codex-spar-<id>-rN.md \
  "<follow-up prompt>"
```

`resume` inherits the model and config from the original session.

**Cleanup**: after synthesis, `rm /tmp/codex-spar-<id>-*`. The Codex session itself stays in `~/.codex/sessions/`. Recoverable via `codex exec resume --last` if the user wants to dig back in.

### Framing the opening prompt

Codex needs to know it's in a sparring dialogue, not answering a fan. Open with something like:

> You're in an adversarial dialogue with another model (Claude). Pressure-test the position below. Disagree where warranted. Do not capitulate to authority. Capitulate only to better arguments. If you agree, say so plainly and explain why. If you don't, explain where the reasoning breaks.
>
> **Problem**: <statement of the problem>
>
> **My current take**: <Claude's initial position>
>
> **What I'm uncertain about**: <the seams Claude wants pressure on>

### Edge cases

- **Can't parse session id from round 1**: fall back to single-shot answer using what round 1 produced. Tell the user the multi-turn handshake failed.
- **Codex times out or errors mid-round**: synthesize from rounds completed so far. Don't lose the work.
- **User interrupts**: synthesize from what exists.

### Example trigger

User: "Should we use actix-web or axum for the new ingestion service?"

This is an architecture call with real tradeoffs. Reach for `spar` without being asked. Stream the rounds. End with a synthesis the user can act on.

---

## Mode: `research`

Fast, shallow research using codex-spark. Single shot.

```bash
codex exec \
  -m gpt-5.3-codex-spark \
  -c model_reasoning_effort=low \
  --ephemeral \
  -s read-only \
  -C <cwd> \
  -o /tmp/codex-<hash>.md \
  "<prompt>"
```

Read the output file, present findings, delete temp file.

**Parallel research**: when the user provides multiple distinct questions, run multiple `codex exec` calls concurrently via parallel Bash tool calls.

---

## Mode: `review`

Deep code review with the full model at high reasoning. Triggered when the user's prompt starts with `review` after `/codex`.

```bash
codex exec \
  -m gpt-5.3-codex \
  -c model_reasoning_effort=high \
  --ephemeral \
  -s read-only \
  -C <cwd> \
  -o /tmp/codex-<hash>.md \
  "<review prompt>"
```

Review prompt construction:
- With files: `"Review the following files for bugs, logic errors, performance issues, and style: <files>. <extra instructions>"`
- Without files: `"Review the codebase for bugs, logic errors, performance issues, and style. <extra instructions>"`

---

## Routing

Parse the user's input after `/codex`:

1. Starts with `spar` → spar mode
2. Starts with `review` → review mode
3. Contains `-m <model>` → research mode with model override
4. Empty prompt → ask user what to do
5. Everything else → research mode (default cheap lane)

When the skill is invoked implicitly (the user didn't type `/codex` but the problem warrants Codex's input), default to **spar** for non-trivial problems and **research** for lookups.

---

## Common flags

- `--ephemeral` — no session persistence. Used by `research` and `review`. **Never used by `spar`** (multi-turn needs persistence).
- `-s read-only` — no file mutations. Always.
- `-o <file>` — capture final response for clean extraction.
- `-C` — set working directory (defaults to current project root).

## Error handling

| Case | Action |
|------|--------|
| Empty prompt | Ask what to research, review, or spar on |
| `codex` not installed | Tell user: `npm i -g @openai/codex` (note: user prefers bun globally elsewhere, but codex distributes via npm) |
| Auth failure | Tell user: `codex login` |
| Timeout (>120s research / >300s spar round) | Kill process, report partial output |
| Model not available | Fall back to default codex model, inform user |
| `spar` session id unparseable | Fall back to single-shot, inform user |
