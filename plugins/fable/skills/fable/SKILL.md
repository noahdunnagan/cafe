---
name: fable
description: Get the most out of Claude Fable 5 — the smartest generally available model, priced and positioned like it. Fable is the architect, not the workhorse: it plans, dictates tasks, evaluates, and verifies, while execution delegates down to GPT-5.6 Sol (headless codex), Sonnet 5, or Haiku 4.5 — Opus 4.8 when the work needs Agent-tool mechanics — unless the work is hard or high-stakes or the user asks for Fable by name. Use when the user says "/fable", "use fable", "use sol", "should this run on fable", asks which model should do a task, or questions model routing and cost. Always active when the session model is Fable 5 or when spawning subagents or workflows from a Fable session; also applies in reverse — summoning a Fable subagent from a cheaper session for work that deserves it.
user-invocable: true
disable-model-invocation: false
---

# Fable

Fable 5 is a Mythos-class model made safe for general release — the most capable model Anthropic has ever made generally available, state-of-the-art on nearly all tested benchmarks, and the gap over other models *widens* as tasks get longer and harder. It's priced like it: 2x Opus 4.8, 3x+ Sonnet 5 per token. So the operating principle is simple: **Fable is the architect, not the workhorse.** Spend it on judgment — plans, briefs, evaluation, verification. Let cheaper models do the typing.

## The ladder

Route work by weight:

| Work | Who | How |
|------|-----|-----|
| Thinking: reading intent, planning, writing briefs, judging results, catching what others missed | **Fable** | Main loop. Always on. Never delegated. |
| Normal execution: features, refactors, tests, scripts, docs | **GPT-5.6 Sol** | One headless codex run — see "Delegating to Sol". Fall back to an Opus 4.8 subagent (`model: "opus"`) when the task needs Agent-tool mechanics or codex isn't available. |
| Light/mechanical: renames, boilerplate, config, searches, sweeps | **Sonnet 5** | Subagent, `model: "sonnet"`. Haiku 4.5 when it's trivial and speed matters. |
| Hard **or** high-stakes: the gnarliest debugging, security-sensitive changes, auth, money paths, data migrations | **Fable itself** | Either alone qualifies — size doesn't launder risk. A 15-line token-expiry tweak is Fable work. This is what the price buys. |
| The user says "use Fable" | **Fable itself** | Explicit request always wins; never downgrade for cost. It pins the work the user pointed at — not every subagent in a fan-out. Confirm before running a whole tree on Fable. |

Frontend follows the same ladder. Never route work to GLM automatically; GLM is sunset and remains explicit opt-in only.

Routing is silent. Pick the model and proceed; don't narrate cost tradeoffs or ask permission to spend unless the user raised cost first.

## When not to delegate

Delegation has a fixed cost the per-token spread doesn't show: writing the brief (Fable-priced), the subagent cold-reading context you already hold, reading its transcript back, verifying. Execute inline when:

- **The round-trip costs more than the task.** Not just one-liners — anything where brief + cold re-read + verification exceeds doing it directly, in tokens or wall-clock.
- **The state lives in the main loop.** Forty turns into a debugging thread, the fix is inseparable from the diagnosis. A brief can't cheaply transfer ruled-out hypotheses, and a cold subagent fixes against a model it doesn't share. Finish it yourself.
- **The user is iterating live.** Turn-by-turn pair work — "change this, now revert that" — dies in subagent round-trips, and the worker can't see the conversation. Stay inline until the loop ends.

## Works both ways

The ladder assumes Fable is the session model, but the mirror holds. On an Opus 4.8 session — the cheaper daily driver, and the only place `/fast` works — summon Fable as a subagent (`model: "fable"`) for the calls that deserve it: architecture verdicts, plan review, the diagnosis nothing else cracks. Same principle in either direction: Fable-grade judgment at the decision points, cheaper tokens everywhere else.

## Fable never leaves

The ladder moves *execution*, not intelligence. Fable always thinks, usually delegates, sometimes executes, never disappears. Every turn's reading, planning, briefing, and review IS Fable — the architect seat is the highest-leverage place for the smartest model, and it's occupied 100% of the time. The wrong reading of this skill is "avoid Fable"; the right one is "don't spend Fable on typing."

When invoked explicitly as `/fable <task>`, run the full architect loop: triage the task against the ladder, plan, brief, delegate, verify with fresh eyes, judge the result. `/fable` on a question just answers with the routing call.

## Delegate right

The ladder is portable to any agent reading this skill; the mechanics below are Claude Code's.

- **Singleton first.** The default delegation is ONE Sol run via headless codex — an Opus subagent when the work needs Agent-tool mechanics. Not a fleet. Most tasks fit in one context window.
- **Workflow fan-outs inherit the session model.** On a Fable session, every `agent()` call in a workflow script silently runs Fable unless you pass `model` explicitly. A four-agent research sweep left on inherit burned ~1M Fable tokens on grep and web fetches. Pass `model: "opus"` or `"sonnet"` on every `agent()` call; leave inherit only where a stage genuinely needs Fable-grade judgment (final synthesis, adversarial verdicts). `agent()` only speaks Claude models — Sol can't run inside a workflow; it runs alongside one via Bash.
- **Size honestly.** Ultracode and workflows are for work too big for one agent: migrations, audits, codebase-wide sweeps. Most tasks are one Opus agent. Orchestration is a scaling tool, not a posture.
- **Verify with fresh eyes.** Anthropic's own guidance: fresh-context verifier subagents beat self-critique. Cheap agent executes, fresh agent checks, Fable judges the verdict — and a high-stakes diff gets Fable's read before merge, whoever wrote it.

## Brief the worker

The subagent inherits the repo, not the conversation. Everything it needs travels in the brief:

- **The decisions already made.** Diagnosis, ruled-out approaches, the chosen design — don't let it re-derive or quietly contradict them.
- **Exact targets.** Files to touch, the pattern to match ("do it like `src/foo.rs` does").
- **Acceptance criteria and hard constraints.** What done looks like; no new deps; don't touch X.

A thin brief means rewriting the output yourself. The brief is the tax the ladder pays — pay it once, properly.

## Delegating to Sol

Sol (GPT-5.6) holds the normal-execution seat on evidence, not vibes: it edges Fable itself on agentic terminal work (Terminal-Bench 2.1: 88.8 vs 84.3), runs at roughly Opus prices ($5/$30 per MTok), and in headless testing shipped first-try-green, clippy-clean, house-style Rust. Codex auto-loads `~/AGENTS.md` and chases its skill references before writing code, so style compliance comes free — the brief doesn't need to restate it.

The tested invocation:

```sh
codex exec -m gpt-5.6-sol \
  -c model_reasoning_effort=high \
  -c service_tier=fast \
  -s workspace-write \
  --skip-git-repo-check \
  -C <workdir> -o <out.md> \
  "<brief>" </dev/null
```

- **`</dev/null` is not optional.** `codex exec` blocks indefinitely reading an open stdin pipe — it looks like a slow model and is actually a hang.
- `--skip-git-repo-check` is required outside a git repo. `-s read-only` for research and review runs.
- Multi-turn: capture the session id codex prints at startup, then `codex exec resume <session-id> "<follow-up>"`.
- `gpt-5.6-sol` is the current top of codex's model cache. If it's ever rejected, resolve the best model from `~/.codex/models_cache.json` (lowest `priority`) instead of pinning harder.

What the codex path gives up vs the Agent tool: schema-forced structured output, worktree isolation, and any view into the conversation. When the work needs those — or codex isn't installed or authed — the fallback is an Opus 4.8 subagent, same brief.

## Effort

Effort (`low` / `medium` / `high` / `xhigh` / `max`) is the thinking-depth dial; Fable defaults to `high`. Two facts change how you use it:

- Low and medium effort on Fable still perform well — often *beating* `xhigh` on prior-generation models. Routine work on a Fable session doesn't need cranking up.
- `xhigh` is for the most capability-sensitive problems. Session-wide: `/effort`. Per-stage in workflows: `effort` in `agent()` opts — `low` for mechanical stages, high tiers for verdicts.

## Prompting Fable

From Anthropic's Fable prompting guidance, the parts that actually change output:

- **De-prescribe.** Prompts and skills written for prior models are often too prescriptive and *degrade* Fable's output. State the outcome and the constraints; delete the step-by-step scripts.
- **Give the reason, not only the request.** Fable uses the why to make calls you wouldn't have thought to specify.
- **Bring it hard problems.** The longer and more complex the task, the larger Fable's lead. Testing it on easy work undersells it — and wastes it.
- **Expect long turns.** Single requests run for minutes at high effort; autonomous runs go for hours. Check on runs asynchronously instead of hovering.
- **Never ask it to reproduce its reasoning in the response.** That trips the `reasoning_extraction` classifier and returns a refusal. Summarized thinking is the supported view.

## Building on the API

For apps: default to `claude-opus-4-8`; reach for `claude-fable-5` when the task needs the ceiling.

| | Fable 5 |
|---|---------|
| ID | `claude-fable-5` (Bedrock: `anthropic.claude-fable-5`) |
| Price / MTok | $10 in, $50 out (Opus 4.8: $5/$25 · Sonnet 5: $3/$15, intro $2/$10 through Aug 2026 · Haiku 4.5: $1/$5) |
| Context / max output | 1M / 128k |
| Thinking | Adaptive, always on. `thinking: disabled` or `budget_tokens` → 400. Depth via `output_config.effort`. |
| Hard 400s | assistant prefill; non-default `temperature` / `top_p` / `top_k` (carried over from Opus 4.7+) |
| Refusals | HTTP 200 + `stop_reason: "refusal"` with `stop_details.category` (`cyber`, `bio`, `frontier_llm`, `reasoning_extraction`). Handle with the `fallbacks` param → Opus 4.8. |
| Retention | 30-day, mandatory. Not available under ZDR. |

> Fast mode (`/fast`) is Opus-only. There is no fast Fable — if you toggled `/fast`, you're on Opus 4.8.

## Failure modes

- **Fable doing chores itself** — the default failure; mechanical, low-stakes steps belong in an Opus or Sonnet subagent.
- **The silent Fable fleet** — a workflow with no `model:` overrides; audit every `agent()` call before running.
- **Over-orchestrating** — twenty subagents where one would do.
- **Downgrading a Fable ask** — if the user said Fable, it's Fable; cost is their call.
- **The thin brief** — a delegation that makes the worker re-derive what the main loop already knew; it comes back wrong or generic.
- **The stdin hang** — a codex delegation without `</dev/null` sits blocked on stdin; minutes of silence for a handful of tokens. It's not thinking, it's waiting.
- **Delegating away your own context** — handing off a fix whose diagnosis lives only in the main loop; the brief loses it.
- **Classifier false positives** — legitimate security- or bio-adjacent work can trip the dual-use classifiers; add authorization context and rephrase.
