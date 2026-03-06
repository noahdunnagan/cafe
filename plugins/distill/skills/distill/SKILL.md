---
name: distill
description: Rigorous code refactoring — every line earns its place. Preserves functionality, removes everything that doesn't justify its existence. Use when user says "/distill", "clean this up", "tighten this", "distill", or wants code quality improved without changing behavior.
user-invocable: true
disable-model-invocation: false
---

# Distill

Strip code to its purest form. Same behavior, fewer words, clearer intent.

This is not a linter. Not a formatter. It's an editorial pass where every line must defend its existence.

## Process

### 1 — Read Everything

Read every file in scope. Understand what the code does, how it connects, what it depends on. You cannot cut what you don't understand. Do not skim. Do not assume. Read.

### 2 — Interrogate Every Line

Walk through the code with these lenses. Every line, every construct, every name.

**Existence:** Why is this here? What breaks if I delete it? If nothing breaks and nothing loses clarity, it goes.

**Comments:** Is this comment saying something the code doesn't already say? A comment that restates the code is noise. A comment that explains *why* something non-obvious exists is signal. Kill the noise, keep the signal.

**Naming:** Is this name earning its keep? A variable called `data` in a function that handles three kinds of data is failing. A variable called `x` in a tight math formula is fine. Context determines the right name length. Names should be as short as possible but no shorter.

**Complexity:** Is there a simpler way to express this? If you're using a $5 word when a 10-cent word works, swap it. Nested ternaries, clever bit tricks, unnecessarily generic abstractions, deep callback chains. Replace with the dumb, obvious version unless the complexity has a measurable reason to exist.

**Abstraction:** Is this abstraction pulling its weight? A helper function called once is just indirection. A trait with one implementation is ceremony. A wrapper that adds nothing is a toll booth. Inline it, flatten it, remove the middleman.

**Control flow:** Is this the simplest path through the logic? Early returns instead of nested ifs. Guard clauses instead of deep branches. Linear flow instead of ping-ponging between functions.

**Dead weight:** Unused imports, unreachable branches, TODO comments with no ticket, commented-out code, debug prints left behind. All of it goes.

**Bad practice:** Is this code doing something that will cause real problems? Not style nitpicks. Actual landmines. SQL built from string concatenation, secrets in source, unbounded retries, silent error swallowing, race conditions, auth checks that can be bypassed. If the code works today but is one bad day away from a security hole, data loss, or cascading failure, that's not a cleanup item. That's a flag. Surface it in diagnosis with urgency. "This works, but it shouldn't be done this way because [concrete reason]."

### 3 — Diagnosis

Surface what you found. This is the checkpoint. No changes yet.

Present your findings grouped by pattern, not line-by-line. Keep it tight. Each finding needs a **what** and a **why**. Refactoring without a reason is pointless churn.

Format:
```
Found N things across M files.

**Unnecessary complexity** — [file:fn] uses a generic trait bound where a concrete type works. Only one type ever implements it. Adds indirection for no polymorphism.

**Dead comments** — [file] has 6 comments restating what the code already says. They cost reading time and add zero information.

**Tangled flow** — [file:fn] nests 4 levels deep with if-let chains. Could flatten to guard clauses with early returns. Same logic, half the indentation.

**Naming** — [file] uses `data`, `result`, `temp` for three distinct domain objects. Names should reflect what they carry.
```

Every finding justifies itself. If you can't articulate why something is a problem, it isn't one. Leave it alone.

**Then ask:**

> "These are the patterns I want to address. Proceed?"

If the user said "don't wait, just do" (or similar) upfront, skip the checkpoint and go straight through. Otherwise, wait for the green light.

### 4 — Restructure

After diagnosis (and approval), restructure. This is where you apply what you found.

**Rules:**
- **Functionality is sacred.** The code must do exactly what it did before. Not approximately. Exactly. If you're unsure whether a change alters behavior, don't make it. Flag it and ask.
- **Don't rearrange for sport.** Moving code around without improving clarity is churn, not improvement. Every structural change needs a reason.
- **Preserve the author's intent.** If the original code solves a problem in a particular way for a reason you can identify, respect that choice. Distillation improves expression, not strategy.
- **Group related logic.** Code that works together should live together. If understanding function A requires reading function B, they should be close.
- **Flatten depth.** Reduce nesting. Use early returns, guard clauses, and extraction to keep indentation shallow.

**When you don't understand something, stop and ask.** This is not optional. If a piece of code looks wrong, redundant, or weirdly structured but you can't explain *why* it's that way, do not touch it. Ask: "Why does this work like this?" The weird thing might be the point. Retries that look excessive exist because the upstream API is flaky and someone learned that the hard way.

If something smells like a workaround or a hard-won fix, treat it as load-bearing until proven otherwise. A 10-second question beats a confident deletion that breaks prod.

Interview the user. If intent is unclear on multiple things, batch the questions. "Before I touch these, I want to understand why they exist." This is a feature of the skill, not a failure of confidence.

### 5 — Verify

Before presenting changes, verify:

- [ ] Does the code produce identical behavior for all inputs?
- [ ] Have you removed something that looked dead but was actually used via reflection, macros, or dynamic dispatch?
- [ ] Did you accidentally tighten a public API (removed a pub method, changed a signature)?
- [ ] Are error paths still handled the same way?
- [ ] If tests exist, would they still pass?

If anything is uncertain, flag it explicitly. Don't ship doubt.

### 6 — Sweep

You cleaned your corner. Now look around the room.

Trace outward from the distilled code. What calls it? What does it call? What renders it, consumes it, tests it? If those adjacent files are messy, they're in scope.

**How to sweep:**
- Follow the dependency graph in both directions. Callers, consumers, shared types, templates, tests.
- Apply the same interrogation lens (step 2) to what you find. Same bar, same questions.
- Stop when you hit code that's already clean or when the connection to the original target becomes tenuous.

**After sweeping, ask the user:**

> "These files are connected to what I just distilled and could use the same pass: [list files with one-line reasons]. Want me to clean these up too?"

Only proceed if they say yes. Never silently expand scope. But always surface the opportunity. Cleaning one function while its caller is a mess just moves the eyesore.

The goal: when someone reviews the PR, the whole area looks intentional. Not one pristine file surrounded by rough edges.

### 7 — Present

Show the distilled code. For each file changed, briefly state what changed and why. Keep the explanations tight. One line per change category is plenty.

Format:
```
## <file_path>
- Removed: <what and why>
- Simplified: <what and why>
- Restructured: <what and why>
```

If the code was already clean, say so. Don't manufacture changes to justify the pass.

---

## What Distillation Is Not

- **Not a rewrite.** The architecture stays. The approach stays. The expression improves.
- **Not a style enforcement.** Don't impose conventions the codebase doesn't use. If the project uses 2-space indent and camelCase, you use 2-space indent and camelCase. Distill within the existing style, not over it.
- **Not feature work.** Never add functionality. Never change behavior. Never "improve" error messages or add logging or handle a new edge case. That's scope creep.

---

## Severity Tiers

Not all code needs the same intensity. Scale the pass to what's in front of you.

**Deep distillation** — New or recently written code. Fresh from an agent or a fast coding session. Apply full interrogation. This is where the most waste lives.

**Light distillation** — Mature, tested code that works. Touch only what clearly needs it. Stable code has earned trust. Don't disturb working systems for marginal gains.

**Targeted distillation** — User points at a specific area. Focus there. Skip the sweep step. Don't wander into adjacent files looking for problems.

---

## The Bar

When you're done, someone reading this code should think: "this was written by someone who cares." Not because it's clever. Because it's clear. Because nothing is wasted. Because every line is there for a reason and that reason is obvious.

That's the bar. Hit it.
