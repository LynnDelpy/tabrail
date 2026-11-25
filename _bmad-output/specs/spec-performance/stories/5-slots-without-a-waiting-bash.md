---
title: 'Slots without a waiting bash'
type: 'refactor'
created: '2026-11-24'
status: 'done'
baseline_commit: '1a8f5b7575a48323a559ae2a56a0fec1d92fe55f'
route: 'dispatch'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/specs/spec-performance/SPEC.md'
  - '{project-root}/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Every tab's reader slot keeps a `bash` (3.9 MB RSS) waiting on its `tabrail-slot` sleep (2.4 MB), about 75 MB of bash at 50 tabs (SPEC CAP-6).

**Approach:** `tabrail-slot` starts its `tabrail-slot` sleep in the background and then `exec`s into a plain `sleep infinity`: the pane's process is a sleep whose child is still `tabrail-slot`, which is what resurrect saves and matches. Run in its pane's shell (a slot resurrect restored), it respawns its pane as itself first, so neither shell nor bash stays and the next save still finds `tabrail-slot` (Lynn, 2026-11-25).

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
- [x] `tabrail-slot` -- the background child and the `exec` into a plain sleep; header comment says why.
- [x] `tabrail-slot-test` -- checks: no `bash` under any slot pane; no `sleep` left after `kill-pane`, `kill-window`, `kill-server`.

**Acceptance Criteria:**
- Given a test server with 3 tabs, then `ps` shows no `bash` for any slot and each slot pane's child is `tabrail-slot`.
- Given `tabrail-slot-test`, then all checks pass, the restore included.

## Implementation Notes

- The Approach alone breaks resurrect for restored slots: resurrect types `tabrail-slot` into the
  restored pane's shell, so the pane's process is that shell and its child would be the plain
  `sleep`, which the next save records and `~tabrail-slot` no longer matches (the restore check
  failed so; 45 of the user's 59 live slots are restored ones, zsh -> bash -> sleep). So a slot run in
  its pane's shell (`pane_pid == $PPID`) respawns its pane as itself, guarded in one tmux command by
  that same condition (the reader may have taken the pane meanwhile); the respawned run is a new
  slot: sleep -> tabrail-slot, no shell. It respawns once (`-e TABRAIL_SLOT_RESPAWNED=1`, skipped
  when set) as `exec '$0'`, so a default-shell that forks (fish) is replaced and cannot loop.
  Checks, marks and `cd /` unchanged.
- `tabrail-slot-test` maps the resurrect entry to `$R/./tabrail-slot`, so respawned restored slots
  are told from new ones by `pane_start_command`. Helpers `allslots` (moved), `pids`, `gone`,
  `badslots` in `tabrail-test-lib`, to keep the test under 100 lines. README: Files table and the
  resurrect note.
- Verified also by hand: a second save after the restore records `:tabrail-slot infinity` for every
  slot, and a second restore brings them back the same way.

- Memory, f4l002's live server before this change (`ps` RSS, 2026-11-25): 59 slots, each a `bash
  tabrail-slot` (3.7-3.9 MB, 217 MB together) over its named sleep; 45 of them restored, under a zsh
  each (409 MB together), 13 made by `mkslot` under the server. After a restart and the next restore
  each slot is a plain `sleep` over its named sleep: ~626 MB less at that count. Running slots keep
  the old shape until they are made again (restore, new tab).
- Verification after review: `bash -n tabrail-slot`; slot 23, reader 34, bar 24, shown 17 ok. By hand:
  `default-shell /sbin/fish`, a slot from `slots` and one typed into a fish pane each respawn once and
  end as sleep -> `tabrail-slot infinity`; after a test restore 0 shells or tmux clients of the server
  left outside its process tree.

## Spec Change Log

- 2026-11-25, Approach (frozen, renegotiated by Lynn): the literal Approach failed the restore check:
  a restored slot runs in its pane's zsh, so the next save records `sleep infinity` and the restore
  after it loses every slot. Options put to Lynn: respawn the pane as itself (bash and zsh go, ~626 MB
  on her 59 slots) or exec the bash into the named sleep under the zsh (217 MB, the zsh stays). She
  chose the respawn; Approach amended. Known-bad state avoided: a restored slot whose pane child is a
  plain `sleep`. KEEP: checks, marks and `cd /` unchanged; the respawn guarded in one tmux `if` by
  `pane_pid == $PPID`; the background named sleep plus `exec sleep infinity`; the kill-pane,
  kill-window, kill-server and restore checks (a second save/restore was checked by hand).

## Review Triage Log

| Finding | Verdict | Route | Evidence |
|---|---|---|---|
| Endless respawn when `default-shell -c` forks (fish, a trap in zshenv): every slot sees `pane_pid == $PPID` (blind, edge, verification-gap) | medium | patch | Reviewer reproduced the fork with fish on a throwaway server; f4l002's zsh execs (`zsh -c` prints `ps`, no zshenv trap), so latent here. A fork would also leave fish's new slots unrestorable. |
| A killed named child goes unnoticed: `sleep infinity` keeps the pane, the next save records no `tabrail-slot` (blind, edge) | low | patch | Before, the waiting bash exited and `slots` made a new slot. Needs a `pkill`/OOM; document it. |
| README resurrect note still says the slot waits in a child named `tabrail-slot` (blind) | low | patch | It waits as a plain sleep; the child only carries the name. |
| The guard `#{==:#{pane_pid},$PPID}` against the reader taking the pane is untested (verification-gap) | low | reject | A ms window between two calls; a test needs a shim swapping the pane between them; the Spec Change Log KEEP pins the guard. |
| Second save after a restore not in the test (blind, verification-gap) | low | reject | `badslots` after the restore requires the pane's one child to be `tabrail-slot infinity`, which is what resurrect's ps strategy saves; KEEP wording corrected. |
| Matrix and ACs not extended for the respawn (blind) | low | reject | Fix edits this build's spec. |
| `pids`/`gone` one level deep; no orphan check after a respawn (blind) | low | reject | Slots are one level (sleep, child); the patch asks for a by-hand check that no restored shell is orphaned. |
| Extra `tmux display` per new slot (blind) | low | reject | Once per new slot or tab, not per switch; folding it into line 16 touches a line the Boundaries keep. |
| `$0` unquoted or relative in the respawn command (blind, edge) | low | reject | `$0` is the path from the resurrect entry or `$D`, plain on every install seen; fix needs two quoting layers. |
| `exec tmux if` failing leaves no fallback (edge) | false | reject | The `if` only skips when the reader respawned the pane, which kills this shell and process too. |
| A `default-command` wrapper without exec hides the shell from `pane_pid == $PPID` (edge) | false | reject | Resurrect saved the wrapper's child before too: same as the baseline. |
| Code Map, tasks, Verification and RSS not filled in (blind) | low | reject | Fix edits this build's spec; RSS and results recorded at verification. |

## Verification

**Commands:**
- `bash -n tabrail-slot && ./tabrail-slot-test && ./tabrail-reader-test` -- expected: all ok.
