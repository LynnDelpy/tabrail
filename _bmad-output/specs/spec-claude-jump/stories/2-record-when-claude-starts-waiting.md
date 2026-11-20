---
title: 'Record when Claude starts waiting'
type: 'feature'
created: '2026-11-20'
status: 'draft'
route: 'dispatch'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/specs/spec-claude-jump/SPEC.md'
  - '{project-root}/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** The Claude state hook records when a prompt started (`@claude_since`) but not when Claude began waiting for the user, so Alt+0 (story 1) cannot order waiting and done tabs by how long they have waited (SPEC CAP-4).

**Approach:** `~/Dev/vmConfig/tmux-claude-state` sets the window option `@claude_waits` to the event's epoch wherever it sets `waiting` (Notification) and on `Stop`; tabrail's README lists it among the hook's optional inputs.

## Boundaries & Constraints

**Always:**
- Use the hook's own `$now` and `setw`. Set `@claude_waits` in the same branch that sets `@claude waiting` (same filter: `idle_prompt` and "waiting for your input" still change nothing), and on `Stop` beside `@claude done`. Each such event overwrites it.
- `SessionEnd` unsets `@claude_waits` with the other three options.
- Header comment names `@claude_waits` and that tabrail's Alt+0 reads it.
- The hook still prints nothing (stdout of some hooks goes into Claude's context) and exits 0.
- vmConfig is a separate git repo: the hook change is committed there on `lde/claude-waits`; the README change goes on the tabrail story branch.

**Never:** Poll, add timers, or derive from `@claude_since`. Change other states, `@claude_took`'s format, `tmux-formats.conf`, `~/.claude/settings.json`, or anything in tabrail besides README. Unset `@claude_waits` on other events (stale outside waiting/done is harmless, SPEC). Commit vmConfig's other untracked files. Run the hook against the default socket: every running Claude calls it, and this shell's `TMUX` points at the user's server.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Permission prompt | `Notification`, `permission_prompt` | `@claude` waiting, `@claude_waits` = now | N/A |
| Idle reminder after a turn | `Notification`, `idle_prompt` or "waiting for your input" | nothing changes; `@claude_waits` keeps the Stop epoch | N/A |
| Turn ends | `Stop` | `@claude` done, `@claude_took` set, `@claude_waits` = now | N/A |
| Session ends | `SessionEnd` | no `@claude*` option left on the window | N/A |
| Outside tmux | `TMUX_PANE` unset | nothing | exit 0 |
| Dead pane or server | `setw` fails | nothing | silent, exit 0 (as today) |

</frozen-after-approval>

## Open Questions

- How the hook change lands. `tmux-claude-state` was never committed (untracked in `~/Dev/vmConfig`), every running Claude calls that copy, and the checkout is on `lde/agent-journal` (commit 15c13e7 today, `.zshrc` modified), likely in use by the agent-journal build. Options:
  - A: worktree `~/Dev/vmConfig-claude-waits` on `lde/claude-waits` from `main`; commit 1 adds the file as it is, commit 2 the change; live copy untouched. Running Claudes record nothing until you merge and swap the untracked copy (`rm`, then pull); Alt+0 orders them by window index meanwhile.
  - B: A, then copy the tested file over the live one. Waits recorded from each Claude's next event; a bug hits every running Claude at once; the later pull still needs the untracked copy removed.
  - C: branch in `~/Dev/vmConfig` itself. Live at once, but switches `lde/agent-journal` and its modified `.zshrc` away under a running build; safe only after agent-journal merges.

## Code Map

- `~/Dev/vmConfig/tmux-claude-state` -- header lines 2-4; `SessionEnd` `unsetw` line 16; Notification filter line 30; `Stop` sets line 37. 40 lines, bash + jq.
- `README.md` "Optional inputs" first bullet -- lists `@claude`, `@claude_since`, `@claude_took`.
- `tabrail-jump` -- story 1, assumed landed: reads `@claude_waits`, unset sorts last in its tier; no change here.

## Tasks & Acceptance

**Execution:**
- [ ] `tmux-claude-state` (where the Open Question puts it) -- set and unset `@claude_waits` per Boundaries; header comment -- CAP-4.
- [ ] `README.md` -- add `@claude_waits` (epoch when Claude began waiting for the user, a question or a finished turn; Alt+0 orders by it) to the hook bullet -- README documents the hook's inputs.
- [ ] Scratch harness (not committed) -- each I/O row via Verification.

**Acceptance Criteria:**
- Given a pane on `tmux -L test`, when permission-prompt JSON is piped into the hook, then `@claude_waits` equals `date +%s` at the call (within 1 s).
- Given that pane 2 s later, when `Stop` JSON is piped in, then `@claude_waits` is 2 s later than before and `@claude` is `done`.
- Given story 1 landed and stand-in Claude tabs X, Y, Z, when the hook gets `Stop` for Y, 1 s later `Stop` for X, then a permission prompt for Z, then Alt+0 from `shell` goes Z, Y, X.
- Given any event, then the hook's stdout is empty and its exit status 0.

## Implementation Notes

## Spec Change Log

## Review Triage Log

## Verification

**Commands:**
- `bash -n $H` (`H` = the edited hook) -- expected: no output.
- Harness:
  ```
  T() { tmux -L test "$@"; }; T -f /dev/null new -d -s main
  P=$(T display -p -t main: '#{pane_id}'); SOCK=$(T display -p -t main: '#{socket_path}')
  hook() { echo "$1" | TMUX=$SOCK,0,0 TMUX_PANE=$P $H; echo "rc=$?"; }
  hook '{"hook_event_name":"Notification","notification_type":"permission_prompt","message":"Claude needs your permission to use Bash"}'; date +%s; T show -w -t $P
  sleep 2; hook '{"hook_event_name":"Stop"}'; T show -w -t $P
  hook '{"hook_event_name":"Notification","notification_type":"idle_prompt","message":"Claude is waiting for your input"}'; T show -w -t $P
  hook '{"hook_event_name":"SessionEnd"}'; T show -w -t $P; T kill-server
  ```
  -- expected: each call prints only `rc=0`; `@claude waiting`, `@claude_waits` = the printed date; then `done`, 2 s later; unchanged after `idle_prompt`; nothing after `SessionEnd`.
- Story 1's harness with the state set by `hook` (`TMUX_PANE` = each tab's claude pane, events as in AC 3) instead of `set -w` -- expected: Z, Y, X.
