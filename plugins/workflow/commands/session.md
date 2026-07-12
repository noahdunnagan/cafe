---
name: session-log
command: session
description: cafe · Update SESSION.md from the recent git diff and commit history — a running log of what changed this session.
---

# Session Log

Read the current session log at `SESSION.md` in the project root (create it if it doesn't exist).

Also look at the recent git diff and commit history to understand what changed this session:
`git diff --stat HEAD~5` or similar to see recent changes
`git log --oneline -10` for recent commits

Based on the code changes and conversation context, suggest:
What problems were tackled this session
What got solved and **how** (brief technical summary)

Present your suggestions and ask me to confirm or adjust them.
Then replace the contents of `SESSION.md` with the new log in this format:

```
## <today's date>

## Problems
- <Short and sweet bit of info about the problem which needs fixing>

### Tackled
- <problem description>
- <what was solved>: <brief how - e.g. "added meilisearch to the shared context">
```

Keep entries concise. This is a personal log, not documentation.
