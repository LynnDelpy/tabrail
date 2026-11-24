---
title: 'Reader bar follows with fewer tmux calls'
type: 'refactor'
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

**Problem:** On every tab switch `tabrail-reader follow` starts bash, takes the lock and runs about ten `tmux` client calls (reader, slot, sidebar, zoom flags, heights, resize, swap, last-pane), and the swap's hooks start `tabrail-reader fit` with more; the bar reaches the new tab late (SPEC CAP-5).

**Approach:** `follow` reads everything it decides on with one `list-panes -a` call and applies the move (sizes, swap, keyboard, zoom) in one chained `tmux` call; `fit` fired by that swap returns after one query when nothing is out of place.

## Boundaries & Constraints

**Always:**
- Same outcome as today in every case `tabrail-reader-test`, `tabrail-slot-test` and `tabrail-bar-test` check: place, heights (`SMALL`, `@reader_big`), zoomed tabs stay zoomed, the keyboard rule after a swap, a slot made when the tab has none, `start` within its 10 s rule.
- The `flock` stays: hooks fire in bursts.
- Chained commands go in one `tmux` invocation (`\;`), one command per element; a failing element must not leave the bar out of place (the next `fit` puts it back, as today).
- `tabrail-reader` and `tabrail-reader-size` stay under 100 lines.

**Never:** Change hooks in `tabrail.tmux` beyond what the fewer calls need, the bar's behaviour, `tabrail-chat`, or the slot script (story 5); test on the default socket.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Switch, slot there | bar in tab 1, switch to 3 | bar in 3, tab 1 holds a slot | N/A |
| Switch, no slot | tab 3 without slot | slot made, then swapped | N/A |
| Big bar | `@reader_big` 11 | small before the swap, as today | N/A |
| Zoomed target | tab 3 zoomed | still zoomed after | N/A |
| No reader | reader pane gone | `start` as today | 10 s rule |
| Burst | 5 switches in 1 s | bar ends in the last tab | lock |

</frozen-after-approval>

## Code Map

- `tabrail-reader` -- `reader`, `slot`, `sidebar`, `zoomed`, `rezoom`, `mkslot`, `move`, `start`, `follow`, `away`; 99 lines.
- `tabrail-reader-size` -- `SMALL`, `height`, `small`, `place`, `fit`, `size`; each `display` is a client call.
- `tabrail.tmux:51-79` -- hooks: `session-window-changed`/`client-session-changed` (follow), `window-pane-changed`/`window-resized` (fit), `window-layout-changed` (resize back).
- `tabrail-reader-test`, `tabrail-slot-test`, `tabrail-bar-test` -- behaviour that must not change.

## Tasks & Acceptance

**Execution:**
- [ ] `tabrail-reader` -- `follow`/`move` from one query and one chained call.
- [ ] `tabrail-reader-size` -- `fit` returns early after one query when bar or slot is in place at its height.
- [ ] Implementation Notes -- tmux execs per follow before and after; `tabrail-perf-test 5 50` before and after.

**Acceptance Criteria:**
- Given a switch on a tabrail test server, when counting `tmux` execs (`strace -f -e trace=execve` on the `run-shell`, or a counting `tmux` wrapper first in PATH), then follow plus the fit it triggers use at most 4.
- Given `tabrail-perf-test 50`, then the reader median is at most 100 ms (SPEC CAP-5).

## Implementation Notes

## Spec Change Log

## Review Triage Log

## Verification

**Commands:**
- `bash -n tabrail-reader tabrail-reader-size && wc -l tabrail-reader tabrail-reader-size` -- expected: parse, under 100 lines each.
- `./tabrail-reader-test && ./tabrail-slot-test && ./tabrail-bar-test` -- expected: all ok.
- `./tabrail-perf-test 5 50` -- expected: reader at 50 tabs at most 100 ms.
