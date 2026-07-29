---
name: token-efficiency
description: ☕️ cafe · Cut daily token burn in Claude Code. Explains where input tokens actually go (cache reads, ~95% of the bill) and the levers that matter, ranked by impact. Use when the user asks about token usage, cost, "why is my bill high", input/output ratios, or how to make sessions cheaper. The always-on disposition (injected via SessionStart hook) covers the in-session behaviors.
user-invocable: true
---

# Token Efficiency

Where the tokens actually go, measured from real Claude Code transcripts on a ~$400/day account: fresh input is negligible (a few thousand tokens per session), output is ~2% of the bill. **95%+ of input is cache reads** — the entire conversation context re-sent on every single API call. One 3,744-message session burned 771M cache-read tokens at an average context of ~330k per call.

The bill is: `(avg context size) × (number of API calls)`. Every rule below attacks one of those two factors.

## The measured facts

- The 5 longest sessions in one day accounted for ~2.6B of ~2.8B cache reads. The median session was 90 messages; the marathons were 1,500–3,700.
- Main-loop calls in marathon sessions ran near the context ceiling (300k+). Subagent calls in the same day averaged ~90k. Same work, quarter the per-call cost.
- One session ran with caching broken (zero cache writes): 2.87M fully-uncached input tokens at 10x the cache-read price. Worst per-call session of the day.
- Output tokens track work done, not waste. In/out ratio comparisons between users are meaningless — lower `in` per `out` is the good direction, and long sessions are what inflate `in`.

## Rules, ranked by impact

**One task, one session.** Context cost is superlinear in session length: a session twice as long has more calls AND a bigger context per call. `/clear` when the task changes. Resuming a 2,000-message session to ask one question re-sends hundreds of k of dead context per turn.

**Compact early, not at the ceiling.** A high context pin with auto-compact near the top means marathons cruise at 300k+/call. If a session must run long, `/compact` proactively at natural checkpoints (after a deploy, after a merge) while context is still ~100k, instead of letting it ride to the threshold.

**Push tool-heavy loops into subagents.** Grep sweeps, test-fix iterations, log spelunking: every tool call in the main loop is another full-context API round. A subagent does those rounds at its own small context and returns one summary. 50 tool calls × 300k main-loop context vs 50 × 90k agent context is the difference.

**Keep bulk out of context.** Big command outputs, file dumps, transcripts → write to a scratchpad file and read back only the slice needed. Never paste a 5k-line log into chat; never re-read a file already in context.

**Don't bust the cache.** Fully-uncached input costs ~10x a cache read. Cache-busters: switching models mid-session, editing always-on hooks/CLAUDE.md mid-day (invalidates every session's prefix), models with no prompt caching (keep those sessions short). Slow first-token on every turn is the tell; start a fresh session.

**Delegate execution down-ladder.** The fable skill already says it: the expensive model plans and judges, cheaper models type. A cheaper model burning the same cache reads costs proportionally less. Marathons especially should not run on the priciest model end to end.

## What to tell the user when they ask about the bill

Report `$`, not input tokens. Input token totals are dominated by cache reads and mostly measure session length, not work. The levers, in order of impact: session length, per-call context size, model tier, output volume.
