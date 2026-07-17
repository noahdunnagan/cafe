---
name: generate-push
command: push
description: ☕️ cafe · Generate a proper conventional commit message from diffs.
---

You are assisting with a Git push.

Look at the current repository state.
Run the necessary git diff commands to understand what changed compared to the current branch.
Generate a single conventional commit message following the Conventional Commits spec.

Format: type[optional scope]: description

Types:
- feat: new feature (MINOR version bump)
- fix: bug fix (PATCH version bump)
- docs: documentation only
- style: formatting, whitespace (no code change)
- refactor: code restructuring (no feature/fix)
- perf: performance improvement
- test: adding/correcting tests
- build: build system or dependencies
- ci: CI configuration
- chore: maintenance (no src/test change)
- revert: reverts a previous commit

Breaking changes: add an exclamation mark after type (e.g. feat!:) or BREAKING CHANGE: in footer.

Keep it short, lowercase, and descriptive.
Then run the exact git commands required to stage, and commit files without pushing. The user will manually push.

NEVER co author claude in the commit. NEVER mention that this was AI generated. This should only be a short quick command for a team member to commit their code from their device.
