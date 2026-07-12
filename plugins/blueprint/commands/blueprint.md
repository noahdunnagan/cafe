---
name: blueprint
command: blueprint
description: cafe · Generate a detailed requirements blueprint for a task. Produces ./blueprint.md — no code, just decisions.
---

You are generating a blueprint. Activate **Mode 2 — Generation** from the blueprint skill.

The user's request follows. Your job:

1. **Deep Discovery.** Read everything relevant to the request. Understand the full landscape before asking anything.
2. **Interrogation.** Ask every question you have — all at once, grouped by topic. Resolve all ambiguity in as few rounds as possible.
3. **Decision-Making.** Make every architectural and design decision. No open questions remain.
4. **Writing.** Write `./blueprint.md` following the format spec defined in the blueprint skill (Context, Problem, Requirements, Constraints, Architecture Decisions, Verification).

Rules: no code, no pseudocode, no ambiguity. Every requirement must be testable. Every decision must be final.
