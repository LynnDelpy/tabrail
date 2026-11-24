---
title: 'Speed test at 5 and 50 tabs'
type: 'chore'
created: '2026-11-24'
status: 'ready-for-dev'
route: 'dispatch'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/specs/spec-performance/SPEC.md'
  - '{project-root}/_bmad-output/specs/spec-performance/cost-today.md'
  - '{project-root}/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Tab switches get slow with many tabs (53 live, tmux server at 35% of a core), and nothing measures it, so no change to the sidebars, the reader bar or the hook can show it helped (SPEC CAP-7).

**Approach:** A test script `tabrail-perf-test [N...]` (default `5 50`) builds, per N, a throwaway tabrail server with N tabs and the reader bar on, shown in an outer server, and prints one line per N: switch median, switch median while hook calls arrive 10/s, reader median, idle tmux CPU, hook ms per call.

## Boundaries & Constraints

**Always:**
- Source `tabrail-test-lib` (`L=pt`): its servers, `t`/`o`, `rd`, `side`, `cleanup` trap. A fresh inner server per N; wait (with a timeout) until every tab has a sidebar and a slot or the reader.
- Distinct hand-given tab names (`rename-window tabK`), so a row is found by its text.
- Switch time: from `t select-window -t :K` to the outer's `capture-pane -p` showing `❯` on `tabK`'s row; `EPOCHREALTIME`; 20 switches spread over the tabs; median and max.
- Reader time: from the same `select-window` to `rd '#{window_index}'` = K.
- Idle CPU: after a settle, utime+stime of the inner server's pid (`/proc/<pid>/stat` fields 14, 15) over `IDLE` seconds (default 60), in % of one core.
- Hook: `~/Dev/vmConfig/tmux-claude-state` 100 times with `TMUX=<inner socket>,0,0` and `TMUX_PANE` a content pane, PreToolUse/PostToolUse JSON alternating; ms per call. The 10/s load for the second switch median uses the same call in a background loop, killed after.
- Output per N, one line: `tabs=50 switch=..ms max=..ms switch@hooks=..ms reader=..ms idle=..% hook=..ms`. Exit 0 unless the setup times out.
- Under 100 lines.

**Never:** Touch the default socket or the user's tmux; `pkill -f`; leave a server, background loop or menu file behind; change any file but the new script and README's Files table.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Default | no args | lines for 5 and 50 | N/A |
| One count | `tabrail-perf-test 5` | one line | N/A |
| Interrupted | Ctrl-C mid-run | servers, loop and menu files gone | trap |
| Setup stuck | a tab never gets its sidebar | message naming N | exit 1, cleaned up |
| No vmConfig hook | file missing | `hook=n/a`, `switch@hooks` skipped | N/A |

</frozen-after-approval>

## Code Map

- `tabrail-test-lib` -- servers `-L tabrail-$L`/`-$L-o`, `serve`, `inner`, `rd`, `side`, `marked`, `cleanup` (menu files by server pid).
- `tabrail-reader-test` -- style of a test script on top of the lib.
- `tabrail.tmux` -- `after-new-window` ensure hook gives each new tab its sidebar and slot.
- `~/Dev/vmConfig/tmux-claude-state` -- read only; called with the inner server's socket.
- `README.md` -- Files table: add the script.

## Tasks & Acceptance

**Execution:**
- [ ] `tabrail-perf-test` (new, executable) -- per Boundaries.
- [ ] `README.md` -- Files table row for `tabrail-perf-test`.
- [ ] Implementation Notes -- the baseline output of `tabrail-perf-test 5 50` on today's main.

**Acceptance Criteria:**
- Given no tabrail test servers, when `tabrail-perf-test 5 50` runs, then it prints two complete lines and exits 0, and afterwards `tmux -L tabrail-pt ls` and `tmux -L tabrail-pt-o ls` fail and no `$XDG_RUNTIME_DIR/tabrail/<its pids>-*` file exists.
- Given the same run twice, then switch medians differ by less than a third.

## Implementation Notes

## Spec Change Log

## Review Triage Log

## Verification

**Commands:**
- `bash -n tabrail-perf-test && wc -l tabrail-perf-test` -- expected: parses, under 100 lines.
- `cargo build --release --offline && ./tabrail-perf-test 5 50` -- expected: two lines; the baseline.
