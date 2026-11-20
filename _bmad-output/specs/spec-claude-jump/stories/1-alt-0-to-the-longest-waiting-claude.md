---
title: 'Alt+0 to the longest-waiting Claude'
type: 'feature'
created: '2026-11-20'
status: 'ready-for-dev'
route: 'dispatch'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/specs/spec-claude-jump/SPEC.md'
  - '{project-root}/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Several Claude chats run in one session; the sidebar marks `● waits` and `✓` but not how long each has waited, so finding the tab that has blocked work longest means clicking around (SPEC CAP-1..3).

**Approach:** A bash helper `tabrail-jump`, bound to Alt+0 in `tabrail.tmux`, orders this session's live Claude tabs (every `waiting`, then every `done`, each by `@claude_waits` oldest first) and selects the tab after the current one in that order, wrapping, or the first when the current tab is not in it.

## Boundaries & Constraints

**Always:**
- Live Claude: `pane_current_command` is `claude` in the window's content pane, picked as `model::tabs` does (skip `@sidebar` 1 panes; first remaining pane, the active one wins).
- Tier from `@claude`: `waiting`, then `done`; anything else is not in the order. In a tier: `@claude_waits` ascending, missing or non-numeric last, ties by window index.
- This session only (`list-panes -s -t <session_id>`).
- Own file under 100 lines, args `SESSION_ID WINDOW_ID`; prints nothing and exits 0 on every path (`run-shell` shows output and non-zero exits in the pane). Nothing to go to: no `select-window`.
- `bind -n M-0 run-shell "#{@tabrail_dir}/tabrail-jump '#{session_id}' '#{window_id}'"` without `-b`, so quick presses queue and each sees the window the last one selected.

**Never:** Change `src/`, drawing, names or status; edit `~/Dev/vmConfig` (the hook is story 2); poll, keep state files, or use `@claude_since`; visit other sessions or non-Claude tabs (`✓`/`✗` commands, bells). Never touch the default socket: this shell's `TMUX` points at the user's server, so use `tmux -L` or `TMUX=<test socket>,0,0`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Success signal | A done 1000, B waiting 1200, C done 1100, D `@claude` waiting in a plain shell; on idle `shell` | 4 presses: B, A, C, B; D never | N/A |
| Current tab in order | on A | C | N/A |
| No `@claude_waits` | B waiting 1200, E waiting unset, A done 1000 | B, E, A | Non-numeric = unset |
| Only current qualifies | on B, A and C idle | stays on B | No `select-window` |
| Nothing qualifies | no live waiting/done | nothing | Exit 0, silent |
| Claude in a non-active pane | 2 content panes, active one zsh | not counted | As `model::tabs` |

</frozen-after-approval>

## Code Map

- `tabrail.tmux:22-23` -- prefix M binding; add M-0 below with a one-line comment.
- `tabrail-move` -- style: header comment with usage, `D=$(dirname "$(readlink -f "$0")")`, ids as args.
- `src/model.rs:24-45` `tabs()` -- content-pane rule to mirror; `status()` gates on `claude` the same way. Read only.
- `README.md` -- intro bullets ("Mouse:") and Files table (`tabrail.tmux` row: "prefix M").
- `~/Dev/vmConfig/tmux.conf` -- `base-index 1`, Alt+1..9; M-0 free. Read only.
- `@claude_waits` -- story 2's hook sets it; set by hand in tests.

## Tasks & Acceptance

**Execution:**
- [ ] `tabrail-jump` (new, executable) -- the helper per Boundaries and Design Notes.
- [ ] `tabrail.tmux` -- the M-0 binding -- ships with tabrail like prefix M.
- [ ] `README.md` -- bullet: Alt+0 goes to the Claude tab waiting longest (`● waits` before `✓`, oldest first), again for the next; Files table: `tabrail.tmux` names Alt+0, new `tabrail-jump` row.
- [ ] Scratch harness (not committed) -- Verification run plus each I/O row.

**Acceptance Criteria:**
- Given the success-signal server on `shell`, when Alt+0 is pressed four times through the outer server, then the current window goes B, A, C, B.
- Given that server on `shell`, when `send-keys M-0 M-0` sends two presses at once, then it ends on A.
- Given any press, then sidebars change only where `❯` moves and no pane shows output or `returned`.

## Implementation Notes

## Spec Change Log

## Review Triage Log

## Design Notes

One `list-panes -s -F` (window id and index, `@sidebar`, `pane_active`, `pane_current_command`, `@claude`, `@claude_waits`); awk keeps one content pane per window and prints `<0 waiting|1 done> <waits padded, unset 999999999999> <index padded> <window_id>`; `sort`; ids into an array; next after `WINDOW_ID`, wrapping, else first. A 13-line scratch prototype passed the success signal and the two-press case.

## Verification

**Commands:**
- `cargo build --release --offline && bash -n tabrail-jump && wc -l tabrail-jump` -- expected: builds, parses, under 100 lines.
- Harness from the repo root, `S` a scratch dir. A plain `#!/bin/bash` script named `claude` shows as `bash` (tmux reads argv0), hence `exec -a`:
  ```
  printf '#!/bin/bash\nexec -a claude sleep 1d\n' > $S/claude; chmod +x $S/claude
  printf 'set -g base-index 1\nset -g @tabrail_groups %s/groups.conf.example\nsource-file %s/tabrail.tmux\n' $PWD $PWD > $S/t.conf
  T() { tmux -L test "$@"; }; T -f $S/t.conf new -d -s main -n shell
  for t in A:done:1000 B:waiting:1200 C:done:1100; do IFS=: read n s w <<<$t
    T new-window -d -n $n $S/claude; T set -w -t main:=$n @claude $s; T set -w -t main:=$n @claude_waits $w; done
  T new-window -d -n D; T set -w -t main:=D @claude waiting; T set -w -t main:=D @claude_waits 900
  tmux -L outer -f /dev/null new -d -x 120 -y 30 "TMUX= tmux -L test attach -t main"; sleep 2
  for i in 1 2 3 4; do tmux -L outer send-keys M-0; sleep 0.4; T display -p -t main: '#{window_name}'; done
  ```
  -- expected: `B A C B`; `tmux -L outer capture-pane -p` shows `❯` on B, no status on D.
- `tmux -L outer kill-server; tmux -L test kill-server` -- expected: both gone with their sidebars; never `pkill -f`.
