---
name: open-pr
command: pr
description: ☕️ cafe · Open a pull request for the current branch — writes the title and body from the diff, pushes if needed. Usage: /pr
---

Open a PR for the current branch, report the URL, stop.

1. Default branch: `gh repo view --json defaultBranchRef -q .defaultBranchRef.name`.
2. PR already exists (`gh pr view --json url -q .url`)? Report it and stop.
3. Push if there's no upstream, or if the branch is ahead.
4. Read `git log <default>..HEAD` and `git diff <default>...HEAD` in full — every commit, not just the last.
5. `gh pr create --base <default> --title "..." --body "$(cat <<'EOF' … EOF)"` — HEREDOC so backticks survive.

No drafts, labels, reviewers, or check polling unless asked.

**Title** — one imperative line that stands alone in `git log` a year from now. "Fix bug" and "Phase 1" fail: they don't say which, or why.

**Body** — the reviewer knows this codebase and can read the diff, so write only what the diff can't show. Test every sentence: *could they get this from the diff?* If yes, cut it.

Keep, in order, whatever applies: why it exists (almost always) · a decision a reviewer would question · what to scrutinize hardest (most valuable, most often missing) · what it deliberately doesn't handle · a test plan only if someone must actually run something.

Cut always: file-by-file inventories, restating what a well-named function does, explaining patterns the team uses daily, checklists nobody reads, launch tone, AI-attribution footers.

**Format** — one continuous line per paragraph, no hard wrapping; GitHub wraps in a narrow column already. Headings only once there are enough sections to need them.

**Length** — three to six sentences. Longer usually means the PR is too big: say so and offer to split it instead of writing more.
