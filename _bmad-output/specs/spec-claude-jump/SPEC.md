---
id: SPEC-claude-jump
companions:
  - ../../../README.md
  - ../../../AGENTS.md
sources: []
---

> **Canonical contract.** This SPEC and the files in `companions:` are the complete, preservation-validated contract for what to build, test, and validate. Source documents listed in frontmatter are for traceability — consult them only if you need narrative rationale or prose color this contract intentionally omits.

# Alt+0: go to the Claude that has waited longest

## Why

Pain, for the one user of the sidebar. Several Claude chats run in one session; a waiting Claude is stalled work and the user is the bottleneck. The sidebar marks `● waits` and `✓ 3m12s` but shows no time for how long either has waited, so finding the right tab means clicking around.

## Capabilities

- **CAP-1**
  - **intent:** Alt+0 goes to the Claude tab of this session that has waited longest for the user: every `waiting` tab first, oldest first, then every `done` tab, oldest first.
  - **success:** With tab A `done` since 10:00, B `waiting` since 10:20, C `done` since 10:10, Alt+0 from an idle shell tab goes to B.
- **CAP-2**
  - **intent:** Pressing Alt+0 again goes to the next tab in that order after the current one, wrapping; from a tab not in the order it goes to the first.
  - **success:** With A, B, C above, presses from the idle shell tab go B, A, C, B.
- **CAP-3**
  - **intent:** Only tabs where the claude process runs count.
  - **success:** A tab with `@claude` `waiting` running a plain shell is never visited.
- **CAP-4**
  - **intent:** The Claude state hook records when Claude began waiting for the user, both for a question (`waiting`) and a finished turn (`done`).
  - **success:** After a permission prompt, `tmux show -wv @claude_waits` gives that moment's epoch; after the turn ends, the epoch of `Stop`.

## Constraints

- The wait start is a new window option `@claude_waits` (epoch), set by `~/Dev/vmConfig/tmux-claude-state` on `Notification` (where it sets `waiting`) and on `Stop`; `@claude*` naming per AGENTS.md. Not polled, not derived from `@claude_since` (the prompt time).
- Live Claude: `pane_current_command` is `claude` in the tab's content pane (sidebar panes skipped, the active pane if several), as `model.rs` picks it.
- This session's tabs only.
- The binding ships in `tabrail.tmux`, like prefix M; the helper is its own file under 100 lines.
- Sidebar drawing, tab names and status unchanged.
- `README.md` is the behaviour reference: its keys and the hook's optional inputs (`@claude_waits`) change with this work.
- Verification only on a throwaway tmux server, with a stand-in process whose `argv[0]` is `claude` (`exec -a claude sleep 1d`; a script named `claude` shows as `bash`) and `@claude` / `@claude_waits` set by hand; never the user's tmux.

## Non-goals

- Other sessions.
- Non-Claude tabs: finished `✓` and failed `✗` commands, bells.
- A queue indicator in the sidebar (count, next marker, wait timer on `● waits`).
- Clearing or hiding stale `@claude` state in the sidebar (spec-tab-program CAP-4).
- Desktop or sound notifications.

## Success signal

On a throwaway tmux server running tabrail, with stand-in Claude tabs A (`done`, oldest), B (`waiting`), C (`done`, newest) and D (`@claude` `waiting`, plain shell), four Alt+0 presses from an idle shell tab land on B, A, C, B; D is never visited.

## Assumptions

- With no live waiting or done Claude besides the current tab, Alt+0 does nothing.
- A waiting or done tab without `@claude_waits` (Claude started before the hook change) sorts last in its tier.
- `pane_current_command` stays `claude` while Claude runs tool commands; they do not take the terminal's foreground.
- `@claude_waits` is read only while `@claude` is `waiting` or `done`; left over in other states it is harmless.
