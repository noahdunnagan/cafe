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

**Body — four sentences. A hard cap, not a target.** One paragraph, one continuous line, no headings, no bullets, no bold labels. The reviewer knows this codebase and is about to read the diff, so the only things worth the space are why this exists and the one place you'd want them looking hardest. Add a single line of test plan only if someone must actually run something.

Everything else comes out, however tempting: what changed, file-by-file inventories, how the code works, restating well-named functions, patterns the team uses daily, what you already verified, checklists, launch tone. Test every sentence — *could they get this from the diff?* If yes, cut it. If four sentences genuinely can't carry the PR, the PR is too big: say so and offer to split it instead of writing more.

**Never** any AI attribution — no `Co-Authored-By` trailer, no 🤖 footer, no "generated with" line — in the body or in any commit on the branch.
