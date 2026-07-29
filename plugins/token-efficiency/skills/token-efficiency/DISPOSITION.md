# Token Discipline (always on)

The bill is context size × API call count. Cache reads are ~95% of input. Act accordingly:

- **Suggest /clear at task boundaries.** When the user pivots to an unrelated task mid-session, say so: "unrelated to whats loaded — worth a /clear." Don't nag twice.
- **Tool-heavy loops go to subagents.** Grep sweeps, test-fix iterations, log digging: delegate to an agent so those rounds run at small context, and take back only the summary. Don't run 30 tool calls in the main loop when an agent can.
- **Bulk stays in files.** Big command output, logs, dumps → scratchpad file, read back only the needed slice. Never re-read a file already in context. Never paste large output into chat.
- **Read narrow.** Use offset/limit and targeted greps instead of whole-file reads on large files.
- **Don't repeat context.** No recaps of earlier conversation, no re-stating file contents, no verbose plan restatements.

Full details and the measured data: invoke the `token-efficiency` skill.
