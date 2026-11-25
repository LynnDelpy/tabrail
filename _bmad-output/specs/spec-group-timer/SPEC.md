---
id: SPEC-group-timer
companions:
  - ../../../README.md
  - ../../../AGENTS.md
sources: []
---

> **Canonical contract.** This SPEC and the files in `companions:` are the complete, preservation-validated contract for what to build, test, and validate. Source documents listed in frontmatter are for traceability — consult them only if you need narrative rationale or prose color this contract intentionally omits.

# A timer in a group's header

## Why

Pain for the sidebar's user: a group is one project, and there is no way to measure how long goes into it. The sidebar already times commands and Claude's turns per tab, but nothing the user starts and stops by hand. A stopwatch on the group's own header row, started, stopped and reset with the mouse, answers "how long did I spend on this".

## Capabilities

- **CAP-1**
  - **intent:** The user can start a group's timer from the group's right-click menu; it counts up in the group's header row, right after the name, in every sidebar of every session.
  - **success:** Right-click group A's row, Start timer: A's header shows a running time next to its name within 1 s in every sidebar, and it advances by the seconds that pass.
- **CAP-2**
  - **intent:** The user can stop a running timer and resume it, by a left click on its time or from the menu; stopped, it keeps showing the frozen time, marked as stopped.
  - **success:** A press and release on the running time stops it: the shown value stays the same over the next 3 s and is marked stopped. Another click on it resumes from that value. Menu Stop timer and Start timer do the same.
- **CAP-3**
  - **intent:** The user can reset a group's timer from the menu.
  - **success:** Reset timer, running or stopped: the time is gone from A's header; a later Start timer counts from zero.
- **CAP-4**
  - **intent:** Several groups' timers can run at once, each on its own.
  - **success:** Start A, then B: both count; stopping A leaves B running.
- **CAP-5**
  - **intent:** A timer survives `tabrail-tmux restart`, a tmux server restart and a reboot; a running one counts the time tmux was down.
  - **success:** Stop A at a value, `tmux kill-server`, start a new server with tabrail: A shows the same value. Leave B running, kill the server, wait 10 s, start again: B shows its old value plus the gap.
- **CAP-6**
  - **intent:** A group's timer follows the group when it is renamed.
  - **success:** Rename group A with a time to C: C's header shows the same time, running or stopped as before.
- **CAP-7**
  - **intent:** A group without a timer shows a start button at its header row's right edge; a click on it starts the timer.
  - **success:** Group A never timed: its row ends in a dim `▷` one column in from the right edge; a press and release on it shows a running time after A's name within 1 s in every sidebar and the `▷` is gone; Reset timer brings the `▷` back.

## Constraints

- The time sits in the group's header row right after the group's name, before a folded group's `N tabs`. A long name is cut first; the time is never cut.
- Without a time the row's second-to-last column holds a dim `▷` instead, a space between it and the pane's border, never cut either: a folded group's count goes first, then the name. No `▷` on a running or stopped row (there the time is the button), nor for a name `clean()` changes (its click folds).
- Timer state lives in its own file that survives a reboot: not in the groups file (the user's config) and not only in tmux options (gone with the server).
- Time is wall-clock: a start epoch plus the time accumulated before it, never a count a sidebar process keeps, so restarts neither lose nor double-count time.
- State writes are atomic (temp file renamed into place). A missing or unreadable state file means no timers; the sidebar still draws.
- A press and release on the time or the `▷` toggles the timer; anywhere else on the row folds as now, and a drag that starts on either still moves the group.
- Every sidebar of every session shows a start, stop or reset on its next redraw (`sidebar-refresh`), no restart.
- `README.md` is the behaviour reference: its groups, mouse and menu text and the Files table change with this work.

## Non-goals

- Automatic counting: by the tab shown, focus, idle or activity.
- A history, log or export of measured times; a total across groups.
- Timers per tab.
- Countdowns, alarms or notifications.
- Setting or editing a time by hand in the UI.
- Showing timers in the tmux status line.

## Success signal

On a throwaway tmux server with two groups holding tabs: SGR right-click on group A's row, Start timer, and `capture-pane -p` of every sidebar shows a time after A's name that grows; an SGR press and release on group B's `▷` starts B's the same way; an SGR left press and release on that time freezes it across two captures 3 s apart, marked stopped; Reset timer removes it; with A stopped at a value, `kill-server` and a fresh server running tabrail show A's same value.

## Assumptions

- Shown as `▶ 12m` while running, in the busy colour, and `⏸ 12m` stopped, dim; the format of the tab times (`45s`, `12m`, `1h5m`, past a day e.g. `27h5m`).
- Menu: Start timer when stopped or never started, Stop timer while running; Reset timer only when the group has a time.
- One timer per group name, shared by all sessions; Default gets one too.
- State file: `$XDG_STATE_HOME/tabrail/timers`, else `~/.local/state/tabrail/timers`.
- A timer whose group was removed from the groups file stays in the state file unseen and shows again if a group of that name comes back.
- A click on a stopped time resumes it; a never-started or reset timer starts from its `▷` or the menu.
- The `▷` is drawn as Nerd Font nf-cod-play (U+EB2C), an outlined triangle centred on the text: the terminal's MesloLGS NF draws its own ▷ on the baseline, visibly low; tmux counts it as one column.
