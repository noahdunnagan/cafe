---
name: glm
description: Legacy instructions for delegating work to GLM, a sunset model that runs headless inside Claude Code. Use only when the user explicitly says "/glm", "ask glm", "have glm build/do this", "let glm", or "delegate to glm"; never invoke it automatically for frontend/UI or other work.
allowed-tools:
  - Bash
  - Read
  - Edit
  - Grep
---

# GLM

GLM is a sunset model — GLM-5.2, 1M context — that ran as a headless Claude Code agent pointed at Fireworks. Keep this skill for explicit legacy use only; never select GLM based on the task type.

## Explicit invocation only

- **Never auto-delegate to GLM.** Frontend/UI and all other work follow the normal model-routing rules.
- **Use GLM only when the user explicitly asks for it.** If the sunset endpoint is unavailable, report that and continue without GLM.
- **Always review explicitly requested output.** Keep architecture, security, money, migrations, and correctness decisions with the main agent.

## The one invocation

The `glm` function already runs `--bare` by default — lean, no MCP/hooks/CLAUDE.md/plugin overhead (a trivial call is ~1.2k tokens, not ~56k; ~0.7s, not ~3s). The brief carries the context; the worker stays lean.

Two lanes, same GLM-5.2, picked by alias — the difference is which Fireworks endpoint you hit:

| Lane | flag | Endpoint | Use for |
|------|------|----------|---------|
| **Fast** (default) | `--model sonnet` | `routers/glm-5p2` — latency-optimized router | Explicitly requested quick drafts and iteration. |
| **Regular** | `--model opus` | `models/glm-5p2` — standard | Bigger/trickier builds, or a re-run when the fast lane looks rushed. |

**Same model both lanes.** `opus` is not the smarter one — it's the same GLM-5.2, just the standard endpoint instead of the fast router. Default to `sonnet`; reach for `opus` only when a fast-lane result looks off or you want it un-rushed.

**Draft** — read-only, returns code or text, touches nothing:
```bash
glm -p --model sonnet "<brief>"
```

**Build** — writes files in the current repo:
```bash
glm -p --model sonnet --permission-mode acceptEdits "<brief>"
```
GLM writes into the current cwd — confirm you're in the repo you mean before firing, especially when fanning out; a misaimed `acceptEdits` run edits the wrong tree silently. To let it touch a dir outside cwd, add `--add-dir=<abs-path>` — note the `=`. `--add-dir` is variadic and will swallow your prompt if you space-separate it.

> Briefs, and any files GLM reads, leave your machine for Fireworks. Don't paste secrets, `.env` contents, or credentials into a brief, and don't `--add-dir` a path full of them.

**With the receipt** — cost, timing, and a session id to iterate on:
```bash
glm -p --model sonnet --output-format json --permission-mode acceptEdits "<brief>" \
  | jq -r '.result, .total_cost_usd, .session_id'
```

**Iterate without re-explaining** — resume the same GLM session to fix or extend:
```bash
glm -p --resume <session_id> --permission-mode acceptEdits "<follow-up>"
```

> In `acceptEdits` headless mode GLM can edit files but can't run shell commands (no prompt to approve them) — it won't `bun install`, build, or run tests. If the task needs that, do it yourself.

## Brief like you mean it

A fast frontend model with a vague brief returns generic slop. The brief is the whole game. Give GLM:

1. **The exact target.** File paths to create or edit. Don't make it hunt.
2. **The stack and conventions.** Framework, styling system, component library, the design tokens and variables it must *use* rather than invent. `--bare` skips CLAUDE.md, so paste the conventions that matter or point at an example: "match the pattern in `src/components/Card.tsx`."
3. **Acceptance criteria.** What done looks like. Props, states, responsive behavior, accessibility, and empty/loading/error states when they apply.
4. **Hard constraints.** "No new dependencies." "Use the existing `Button`." "Tailwind only, no CSS files."

Thin brief and you'll be rewriting it. Thick brief and it's faster and better than doing it yourself.

## Always review

Treat the output as a draft from a quick, capable hand — never as merged.

- **Build lane:** `git diff` immediately. Confirm it used existing components and tokens instead of reinventing them, added no stray dependencies, didn't over-engineer, and didn't invent APIs that don't exist.
- **Draft lane:** read it before you paste it.

GLM never ran its own output — in `acceptEdits` mode it can't typecheck, build, or run anything. Do that yourself before trusting it; that's where the invented imports and non-existent APIs actually surface.

Its usual tells: inventing props or utilities that don't exist, ignoring your design system for generic markup, over-abstracting a simple component, and confident-but-wrong import paths. Catch those; the win still nets out well ahead.

## Fan out

Cheap and fast means parallelize independent work — multiple Bash calls in one message:
```bash
glm -p --model sonnet --permission-mode acceptEdits "Build <Navbar> per spec: ..."       # call 1
glm -p --model sonnet --permission-mode acceptEdits "Build <Footer> per spec: ..."        # call 2
glm -p --model sonnet --permission-mode acceptEdits "Build <PricingTable> per spec: ..."  # call 3
```
Then review the combined diff once. Only fan out work that's truly independent — no shared files, no shared barrel/`index`, router, or types file, no `package.json` edits. Each worker sees the pre-fan-out tree, so overlapping writes silently overwrite each other. When in doubt, run them in sequence.

## Long builds

For a big component or a slow task, run it in the background (Bash `run_in_background: true`) and keep working. Capture json to a file so you can read the result and session id when it lands — use a unique filename so concurrent runs don't clobber each other:
```bash
glm -p --model opus --output-format json --permission-mode acceptEdits "<big brief>" > "${TMPDIR:-/tmp}/glm-$(date +%s).json"
```

## Prereqs and failure modes

- Requires the `glm` shell function from `~/.zshrc` — points `claude` at Fireworks, runs `--bare` by default, and maps `sonnet`→fast (`routers/glm-5p2`), `opus`→regular (`models/glm-5p2`). The skill never sees the token; it just calls the function. If `glm` isn't found, that function isn't loaded in this shell.
- **Auth or endpoint error** → the Fireworks token or base URL in the `glm` function is stale. Tell the user; don't paper over it.
- **`Model not found` (HTTP 404)** → the function's slot mapping is off. Fast must point under `routers/`, regular under `models/`. Don't swap them.
- **Empty prompt** → ask what to build or draft.
- **GLM goes off the rails** → don't loop arguing with it. Take what's useful, finish it yourself, or re-brief once, tighter.
