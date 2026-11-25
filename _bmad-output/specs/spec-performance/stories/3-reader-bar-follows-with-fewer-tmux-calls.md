---
title: 'Reader bar follows with fewer tmux calls'
type: 'refactor'
created: '2026-11-24'
status: 'done'
baseline_commit: 'c056d8b5decf3f0c9ecb5680df3a12176d68bc60'
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
- [x] `tabrail-reader` -- `follow`/`move` from one query and one chained call.
- [x] `tabrail-reader-size` -- `fit` returns early after one query when bar or slot is in place at its height.
- [x] Implementation Notes -- tmux execs per follow before and after; `tabrail-perf-test 5 50` before and after.

**Acceptance Criteria:**
- Given a switch on a tabrail test server, when counting `tmux` execs (`strace -f -e trace=execve` on the `run-shell`, or a counting `tmux` wrapper first in PATH), then follow plus the fit it triggers use at most 4.
- Given `tabrail-perf-test 50`, then the reader median is at most 100 ms (SPEC CAP-5).

## Implementation Notes

tmux execs per tab switch, f4l002, tmux 3.8, 2026-11-25: a counting `tmux` wrapper first in PATH
(logs calls whose parent is `tabrail-reader`) on a throwaway 5-tab server, baseline c056d8b vs this
branch:

| Switch | Before | After |
|---|---|---|
| slot there, bar not active | 13 | 2 |
| bar active and 11 rows (`@reader_big` 11) | 15 | 2 |
| target zoomed | 15 | 2 |
| target without a slot (marked one unset) | 16 | 3 (the `split-window`) |
| slot active in the target: the bar becomes active there, `window-pane-changed` fires `fit` | 20 | 3 (follow 2, fit 1) |

In the usual switch no `fit` fires at all (before either): the swap moves the keyboard only in the
tab left, whose `window-pane-changed` hook finds no reader there.

`tabrail-perf-test 5 50`, back to back, baseline c056d8b (`git archive`, same release binary: no
Rust change) then this branch; exit 0, nothing on stderr either time:

```
before tabs=5 switch=9ms max=29ms switch@hooks=10ms hooks=9.8/s reader=111ms idle=0.08% hook=15.1ms
before tabs=50 switch=20ms max=42ms switch@hooks=17ms hooks=9.8/s reader=136ms idle=0.38% hook=16.3ms
after  tabs=5 switch=8ms max=21ms switch@hooks=8ms hooks=9.9/s reader=70ms idle=0.07% hook=13.5ms
after  tabs=50 switch=19ms max=33ms switch@hooks=15ms hooks=9.8/s reader=95ms idle=0.42% hook=15.6ms
```

- AC 1, met: 2 execs per switch, 3 with the `fit` it triggers. AC 2, met with little margin: 95 ms.
- What is left of the 70-95 ms is mostly fixed cost before the query: the resurrect check's
  `pgrep -f` takes ~42 ms here (~690 processes). No cheap drop-in: `pgrep -x bash` takes as long and
  a builtin-only scan of `/proc/*/cmdline` ~31 ms (reading the command lines is the cost). The next
  lever if CAP-5 needs margin, not done (behaviour of the restore pause, out of scope).
- `follow`: `state` reads the target window, its sidebar and slot (`list-panes -t`) and the reader
  (`list-panes -a -f '#{@reader}'`) chained behind one `display` in a single tmux invocation, also
  `@tabrail_reader`, so `follow` and `fit` skip the header's `show -gv`. `move` is one chained call:
  slot small, `join-pane` if it was out of place, bar small if big or off, swap, then `last-pane` and
  the rezooms as `if -F` bodies, so they check at run time as today's `display` calls did.
- Failure: checked on tmux 3.8, a failing command ends the rest of its `\;` list, one failing inside an
  `if -F` body ends only that body. So a failure before the swap leaves the bar where it was (killed the
  slot between `state` and `move`: "no such pane", bar unchanged, the next switch moves it), and a
  `last-pane` without a last pane does not stop the rezooms (as today, where each was its own call).
- Join decision from the query: `resize-pane` on a hidden pane unzooms the window (checked), and a
  zoomed window's hidden panes keep their geometry, so today's run-time zoom check in `place` was
  always 0 there; `move` joins the slot iff the query found it out of place.
- `fit`: one call (`display` + the window's reader, then its slot) and returns when the bar or slot is
  across the bottom at the height `height` would give it with the `@reader_big` it would leave;
  otherwise today's `place` and `height`.
- Lines: `tabrail-reader` 83, `tabrail-reader-size` 78, new `tabrail-reader-move` 37 (`state`, `move`;
  in `tabrail-reader` they made 117). README: Files table rows, "Fast:" bullet.
- Edge cases checked on a throwaway server: 5 switches in 1 s end in the last tab (one slot or bar per
  tab); no slot: made, swapped, the tab left holds the slot; no slot and zoomed: stays zoomed on its
  pane; bar active when left: the keyboard back on the pane before it (as baseline).
- Every `tabrail-*-test` passes: reader 34, slot 21, bar 24, shown 17, chat 25, chat-session 13 ok.
- After review: `newslot` holds the slot's split for `mkslot` and `move`; `small()` gone; bar-test's burst
  check sends 5 switches at once (follows queue on the lock) and checks every slot 160x4 at the bottom;
  new `tabrail-move-test` (counting `tmux` first in PATH): a switch at most 4 calls, `fit` in place 1,
  the `join-pane` branch, a zoomed tab without a slot. Re-run: reader 34, slot 21, bar 25, move 5 ok;
  `tabrail-perf-test 5 50` reader=68ms / 92ms, switch 8 / 18 ms, idle 0.08% / 0.37%, hook 13.7 / 15.0 ms.

## Spec Change Log

## Review Triage Log

| Finding | Verdict | Route | Evidence |
|---|---|---|---|
| No committed check of tmux execs per switch or of `fit`'s early return (verification-gap, blind) | medium | patch | Pre-verified gap: no test counts calls, `tabrail-perf-test` exits 0; breaking the early return passes every test. |
| `move`'s `join-pane` branch (slot out of place) never runs in a test (verification-gap, blind) | medium | patch | Pre-verified: every layout change before a switch leaves the slot in place or is fitted first; real flow is `split-window -fv` in the bar's tab, away, back. |
| Switch into a zoomed tab without a slot goes through `move`'s split only in the hand script (blind) | low | patch | slot-test:45 makes its slot with `slots`, not `move`; same new test file. |
| Slot killed between `state` and `move` only hand-checked (blind) | low | reject | A race; unlikely in use, a test needs sourcing internals. |
| `small()` has no callers left (verification-gap) | low | patch | grep: only its definition at tabrail-reader-size:21 and the comment at tabrail-reader:21. |
| Slot split duplicated in `mkslot` and `move` (blind) | medium | patch | Story 5 changes the slot command and its Code Map names only `mkslot`; `move` would keep the old one. |
| Burst check switches 0.2 s apart, follows never wait on the lock; `marked` misses slot geometry (blind, edge) | low | patch | Follow takes 70-95 ms, so no overlap; matrix row names the lock. |
| `tabrail-reader-move` does not say it needs `SMALL`, `PANE`, `D` (blind) | low | patch | Sourcing order is the only thing that makes `-F "$PANE"` non-empty. |
| README "Fast:" bullet omits that `pgrep` is most of the remaining time (blind) | low | patch | Notes measure ~42 of 70 ms in the resurrect check. |
| Notes' test counts stale (bar 24, now 25) (blind) | low | reject | Fix edits this build's spec; counts refreshed at verification. |
| AC 2 from one run with 5 ms margin (blind) | low | reject | Fix edits this build's spec; perf re-run at verification. |
| Failed `join-pane` stops the move (blind, edge) | false | reject | Stopping leaves the bar in place, which the spec requires; the old code swapped it out of place. |
| A failing chain element leaves a zoomed target unzoomed (edge, blind) | low | reject | Needs a vanished pane or a window too short for 4 rows plus zoom; fix adds a guard branch. |
| Slot resized even when `state` knows it is small (blind) | false | reject | Same commands as baseline's `small "$s"`, inside the one call; no exec added, no outcome changes. |
| `move`'s exit status is 1 after `last-pane` fails in its if-body (blind) | false | reject | No caller reads it: `follow`/`away` ignore it, `case` exits 0. |
| Multi-line `@tabrail_reader` breaks `state`'s parse (edge) | low | reject | Nobody sets a multi-line reader command; fix reroutes CMD through `start`. |
| `away` in a session's only tab: `state ""` resolves the closing tab (edge) | false | reject | tmux 3.8: empty `-t` and no `-t` both give the current window (checked), so the old `sidebar ""` did the same. |

## Verification

**Commands:**
- `bash -n tabrail-reader tabrail-reader-size && wc -l tabrail-reader tabrail-reader-size` -- expected: parse, under 100 lines each.
- `./tabrail-reader-test && ./tabrail-slot-test && ./tabrail-bar-test` -- expected: all ok.
- `./tabrail-perf-test 5 50` -- expected: reader at 50 tabs at most 100 ms.
