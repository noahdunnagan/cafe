---
name: open-pr
command: pr
description: ☕️ cafe · Open a pull request for the current branch — writes the title and body from the diff, pushes if needed. Usage: /pr
---

You are opening a pull request. Open it, report the URL, stop.

## Prerequisites

`gh`, authed. Nothing else.

## Steps

1. Resolve the default branch: `gh repo view --json defaultBranchRef -q .defaultBranchRef.name`.
2. If a PR already exists for this branch (`gh pr view --json url -q .url`), report its URL and stop — don't open a second one.
3. If the branch has no upstream, push it: `git push -u origin <branch>`. If it has one and is ahead, push.
4. Read `git log <default>..HEAD` and `git diff <default>...HEAD` in full. Cover every commit, not just the latest.
5. Write the title and body per the rules below.
6. `gh pr create --base <default> --title "..." --body "$(cat <<'EOF' … EOF)"` — HEREDOC so multi-line content and backticks survive.
7. Report the PR URL.

Never open a draft unless asked. Never add labels, request reviewers, or poll checks.

## Title

One imperative line that stands on its own in `git log` a year from now. Conventional-commits prefix if the repo uses them. Take the latest commit's subject if it already describes the whole branch; otherwise write one that does.

"Fix bug", "Add patch", "Phase 1", "Moving code from A to B" are all failures — they don't say which bug, which patch, or why.

## Body

**The reviewer already knows this codebase and can read the diff. Your job is the part the diff can't show: why.**

That single idea kills most of what AI writes into PR bodies. Before every sentence, ask: *could they get this by looking at the diff?* If yes, cut it.

Include, in this order, only what applies:

- **Why this exists** — the problem, the bug, the request. One or two sentences. This is the only section that's almost always required.
- **The non-obvious decision** — where you picked one approach over another and a reviewer might wonder why. Only if there was a real fork in the road.
- **What to look at hardest** — if some part carries the risk, point at it. This is the highest-value line in most PR bodies and the most often missing.
- **Known limits** — what this deliberately doesn't handle, what's deferred. Cheap to write, saves a whole review round-trip.
- **Test plan** — only when the reviewer must actually run something to trust it. If the tests in the diff cover it, the diff already said so.

Never include:

- A file-by-file or commit-by-commit inventory. That's the "Files changed" tab.
- Restating what the code does when the name already says it. "Adds a `parseConfig` function that parses the config" is noise.
- Explaining language features, common libraries, or standard patterns the team uses daily.
- Ceremonial checklists nobody reads — "[x] Code follows style guide", "[x] I have tested this".
- Marketing tone. It's a change, not a launch.
- Any AI-attribution footer — no `🤖 Generated with [Claude Code]`, no `Co-Authored-By: Claude`, no mention that this was AI-written.

## Formatting

Write each paragraph and bullet as **one continuous line — no hard wrapping at 80 or 100 characters.** GitHub renders in a narrow column and wraps for you. Hard-wrapped source reads fine in a terminal and ragged in the browser, which is where it will actually be read.

Headings only when there are enough sections to need them. A three-line body doesn't get a `## Summary`.

## Length

Most PRs need three to six sentences. A large or subtle one might need three short paragraphs. If it's running longer than that, the PR is probably too big to review well — say so to the user and offer to split it rather than writing a longer body to compensate.

A reviewer should know what they're looking at, and where to spend their attention, within about ten seconds.
