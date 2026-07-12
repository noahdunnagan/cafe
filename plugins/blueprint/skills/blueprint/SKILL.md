---
name: blueprint
description: cafe · Three-mode planning system — always-active disposition, explicit blueprint generation (/blueprint), and blueprint execution. Scales rigor to task size.
user-invocable: true
disable-model-invocation: false
---

# Blueprint

Three modes. One purpose: get from intent to correct implementation with zero ambiguity.

---

## Mode 1 — Disposition (always active)

This is how you approach work. Not a ceremony — a disposition. These instincts apply to every change, scaled to fit. Complements plan mode by structuring thinking during any planning activity.

### Rules

- **Ask relentlessly.** If something is even slightly ambiguous, ask. If you think you know the answer but aren't 100% sure, ask. Err on the side of annoying me over assuming wrong. The only dumb question is the one you didn't ask that led to a wrong assumption. Once I've answered, that answer is settled.
- **Delegate via subagents.** Break work into discrete tasks and hand them to subagents wherever possible to preserve context window quality. Each subagent task should be self-contained with clear inputs and expected outputs.
- **Scale to the task.** A one-file feature doesn't need a formal 5-phase document. But it still needs the same thinking: understand first, know what "done" looks like, sanity-check yourself, then confirm before building. Use your judgment on how much structure to surface vs. internalize.

### The Phases

#### 1 — Discovery

Understand before you act.

- **Background:** What exists today? Gather the relevant context.
- **Problem:** What's broken, missing, or insufficient? One clear paragraph.

For small tasks this can be a few sentences. For large ones, be thorough.

#### 2 — Requirements

The "done" checklist. What must be true for this work to ship? These are your acceptance criteria — every decision traces back to one of these. Nothing more, nothing less.

For a quick feature, this might be 2-3 bullets. For a system redesign, it's a full list.

#### 3 — Proposed Solution

The plan. Structured so anyone handed it cold could execute with zero guesswork:

- Ordered steps with explicit inputs, outputs, and dependencies
- File paths, function signatures, and data shapes where relevant
- No implicit knowledge or assumed context

For small changes, this can be a brief outline. For large ones, be precise.

#### 4 — Self-Critique

Pressure-test before presenting:

- Is every step **in scope**? Cut anything not tied to a requirement.
- Is the ordering **logically sound**? Would any step fail because a dependency hasn't been met?
- Are there **gaps**? Steps where a reader would need to fill in unstated details?

Revise based on findings. For small tasks, do this internally. For large ones, show your work.

#### 5 — Sync

Quick conversational check-in. A few sentences on what you think we're building and why. "Making sure we're on the same page" not "presenting a document." Ask if anything's off. Don't start building until I confirm.

This step always happens out loud, regardless of task size.

---

## Mode 2 — Generation (`/blueprint`)

Activated by the `/blueprint` command. Produces a `./blueprint.md` requirements document — no code, no pseudocode — detailed enough that any competent developer or fresh Claude session could execute it and produce exactly what was intended.

### Process

1. **Deep Discovery.** Read everything relevant. Understand the full landscape before asking a single question.
2. **Interrogation.** Ask every question you have — all at once, grouped by topic. Don't drip-feed. Get all ambiguity resolved in as few rounds as possible.
3. **Decision-Making.** With answers in hand, make every architectural and design decision. Commit to specifics. No "could do X or Y" — pick one and justify it.
4. **Writing.** Write `./blueprint.md` following the format spec below. Every section is mandatory.

### Blueprint Format Spec

The output file `./blueprint.md` must contain these sections in order:

```
# Blueprint: <title>

## Context
What exists today. The relevant state of the world. A reader with zero prior
context should understand the starting point after reading this section.

## Problem
What's broken, missing, or insufficient. One clear section — not a list of
symptoms, but the core issue.

## Requirements
Numbered list. Each requirement is:
- Specific and testable (has a clear pass/fail condition)
- Independent where possible (one concern per requirement)
- Necessary (traces to the problem statement)

No requirement should require interpretation. "Fast" is not a requirement.
"Responds in under 200ms at p95" is.

## Constraints
Hard boundaries the solution must operate within. Technology choices already
made, backward compatibility needs, performance budgets, regulatory requirements.
Things that limit the solution space.

## Architecture Decisions
Decisions that shape the implementation. Each decision includes:
- **Decision:** What was decided
- **Rationale:** Why this over alternatives
- **Consequences:** What this enables or forecloses

These are final. The executor does not revisit them.

## Verification
How to confirm each requirement is met. Maps 1:1 to the requirements list.
Each entry describes what to check and what the expected result is.
```

### Rules

- **No code.** No pseudocode, no code blocks, no function signatures. The blueprint is a requirements document, not an implementation guide.
- **No ambiguity.** If a sentence could be interpreted two ways, rewrite it until it can't.
- **Decisions are final.** Every "or" must be resolved before writing. The blueprint contains answers, not options.
- **Requirements are testable.** Every single one has a clear pass/fail condition.

---

## Mode 3 — Execution

Activated when told to execute or implement a `blueprint.md` file.

### Rules

- **Don't re-plan.** The blueprint contains the decisions. Your job is to implement them faithfully, not to second-guess them.
- **Don't modify the blueprint.** The blueprint is the source of truth. If something in it is wrong, stop and surface it — don't silently fix it.
- **Track progress.** Work through requirements in order. Know which are done, which are in progress, and which are remaining.
- **Use the disposition.** Mode 1 still applies — delegate to subagents, scale structure to task size. The blueprint tells you *what* to build; the disposition guides *how* you work.

### Blocker Protocol

When you hit something that prevents progress — a requirement that's contradictory, a constraint that's impossible, a dependency that doesn't exist, an ambiguity that wasn't resolved — **STOP**. Do not silently skip it. Do not improvise around it.

State clearly:

1. **What the blocker is.** Specific requirement or decision that can't be implemented as written.
2. **What you tried.** Or why you can't resolve it yourself.
3. **What options exist.** Give me choices so we can unblock quickly.

Then wait for direction before continuing past the blocked requirement.
