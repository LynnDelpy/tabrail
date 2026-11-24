---
title: 'Slots without a waiting bash'
type: 'refactor'
created: '2026-11-24'
status: 'ready-for-dev'
route: 'dispatch'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/specs/spec-performance/SPEC.md'
  - '{project-root}/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Every tab's reader slot keeps a `bash` (3.9 MB RSS) waiting on its `tabrail-slot` sleep (2.4 MB), about 75 MB of bash at 50 tabs (SPEC CAP-6).

**Approach:** `tabrail-slot` starts its `tabrail-slot` sleep in the background and then `exec`s into a plain `sleep infinity`: the pane's process is a sleep whose child is still `tabrail-slot`, which is what resurrect saves and matches.

## Boundaries & Constraints

**Always:**
- Everything before the wait stays as it is (the checks, the marks, `cd /`).
- Resurrect save and restore brings every slot back with the vmConfig `@resurrect-processes` entry `~tabrail-slot->~/Dev/tabrail/tabrail-slot` unchanged.
- Closing the slot's pane, its window or the server leaves no `sleep` behind.
- The reader's swap with a slot and `tabrail-tmux reap` work as before.

**Never:** Edit vmConfig; replace the wait with a new binary; test on the default socket.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| New slot | tab gets a slot | pane process `sleep`, its one child `tabrail-slot infinity`, no `bash` | N/A |
| Pane closed | `kill-pane` on a slot | both processes gone | N/A |
| Window closed | `kill-window` | none left | N/A |
| Server killed | `kill-server` | none left | N/A |
| Restore | resurrect save, kill, restore | every tab has its slot again | N/A |

</frozen-after-approval>

## Code Map

- `tabrail-slot` -- the checks and marks, then `(exec -a tabrail-slot sleep infinity)` under bash.
- `tabrail-slot-test` -- slot checks and the resurrect restore with its own `@resurrect-dir`.
- `~/Dev/vmConfig/tmux.conf:90` -- `@resurrect-processes`; read only.
- `tabrail-reader` `mkslot` -- starts `tabrail-slot` in a split.

## Tasks & Acceptance

**Execution:**
- [ ] `tabrail-slot` -- the background child and the `exec` into a plain sleep; header comment says why.
- [ ] `tabrail-slot-test` -- checks: no `bash` under any slot pane; no `sleep` left after `kill-pane`, `kill-window`, `kill-server`.

**Acceptance Criteria:**
- Given a test server with 3 tabs, then `ps` shows no `bash` for any slot and each slot pane's child is `tabrail-slot`.
- Given `tabrail-slot-test`, then all checks pass, the restore included.

## Implementation Notes

## Spec Change Log

## Review Triage Log

## Verification

**Commands:**
- `bash -n tabrail-slot && ./tabrail-slot-test && ./tabrail-reader-test` -- expected: all ok.
