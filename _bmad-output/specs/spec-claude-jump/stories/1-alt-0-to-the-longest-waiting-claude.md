---
title: 'Alt+0 to the longest-waiting Claude'
type: 'feature'
created: '2026-11-20'
status: 'done'
route: 'dispatch'
review_loop_iteration: 1
baseline_revision: 'f14007bdbe2c38a04bf6c61b66d35079d754c614'
followup_review_recommended: false
context:
  - '{project-root}/_bmad-output/specs/spec-claude-jump/SPEC.md'
  - '{project-root}/AGENTS.md'
deferred:
  - summary: >-
      tabrail-test-lib's cleanup returns before kill-server has finished, so a tabrail-*-test started
      right after another can attach to the dying server and fail every check.
    evidence: |-
      Seen by the implementer in 4 of 8 back-to-back tabrail-jump-test runs ("server exited
      unexpectedly"); with a few seconds between runs every run passed. Affects every test on the lib.
    location: >-
      tabrail-test-lib cleanup()
    severity: medium
  - summary: >-
      AGENTS.md's tmux pitfalls lack three lessons from this build.
    evidence: |-
      list-clients also lists every sidebar's control-mode client (filter client_control_mode); a
      script named claude shows as bash in pane_current_command (tmux reads argv0, hence exec -a);
      a key binding's run-shell without -b holds that client's later keys until the job ends.
    location: >-
      AGENTS.md, tmux pitfalls
    severity: low
  - summary: >-
      ~/Dev/vmConfig/tmux.conf's key list comment does not mention Alt+0 (story 1 may not edit vmConfig).
    evidence: |-
      tmux.conf:37-39 says Alt+1..9 picks a tab and M comes with tabrail.tmux; Alt+0 now comes with
      tabrail.tmux too. Story 2 commits to vmConfig and can add it there.
    location: >-
      ~/Dev/vmConfig/tmux.conf:37-39
    severity: low
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

- `tabrail.tmux:32-33` -- prefix M binding; add M-0 below with a one-line comment.
- `tabrail-move` -- style: header comment with usage, `D=$(dirname "$(readlink -f "$0")")`, ids as args.
- `src/model.rs:24-46` `tabs()` -- content-pane rule to mirror: skip panes with `@sidebar`, `@reader` or `@reader_slot` 1 (line 30), first remaining pane, the active one wins; `status()` gates on `claude` the same way. Read only.
- `README.md` -- intro bullets (after "Mouse:"); the status bullet (lines 79-82: Claude's `✓ 3m` done versus the plain `✓` of a command that finished out of sight, which Alt+0 never visits); Files table (`tabrail.tmux` row: "prefix M"; the test rows, lines 210-226).
- `tabrail-test-lib` -- what committed tests share: set `L`, then source it; `t`/`o` are the inner and outer servers on `-L tabrail-$L` and `-L tabrail-$L-o`, `check`, `settle`, `inner`, `side`, `cleanup` on EXIT (only its own servers' menu files); its conf starts a reader bar (`sed -i '/@tabrail_reader/d' "$T/conf"` drops it, as `tabrail-colour-test` does).
- `tabrail-colour-test:1-25`, last line -- a committed test's shape: header comment listing what it covers, `L=…`, `source …/tabrail-test-lib`, an `outer` helper, one `check` per line, `exit $((fails > 0))`.
- `~/Dev/vmConfig/tmux.conf` -- `base-index 1`, Alt+1..9; M-0 free. Read only.
- `@claude_waits` -- story 2's hook sets it (`$EPOCHSECONDS`, whole seconds); set by hand in tests.

## Tasks & Acceptance

**Execution:**
- [x] `tabrail-jump` (new, executable) -- the helper per Boundaries and Design Notes.
- [x] `tabrail.tmux` -- the M-0 binding -- ships with tabrail like prefix M. Its one-line comment names both sides of leaving out `-b`: quick presses queue in order, and each press holds that client's keys until the helper returns (so the helper stays one tmux read and at most one `select-window`).
- [x] `README.md` -- bullet after "Mouse:": Alt+0 goes to the Claude tab in this session that has waited longest for you (every `● waits` tab before every Claude `✓ 3m` done one, each oldest first by `@claude_waits`), again for the next; worded so it cannot read as "the session that waited longest", and so Claude's done mark is not confused with the plain `✓` of a finished command. Files table: `tabrail.tmux` names Alt+0, new `tabrail-jump` row, new `tabrail-jump-test` row among the test rows.
- [x] `tabrail-jump-test` (new, executable, committed, under 100 lines) -- `L=jump` on `tabrail-test-lib`, the reader bar dropped from its conf. Stand-in Claude tabs via `exec -a claude sleep 1d`, presses as real `M-0` keys sent to the outer server. Checks: one per I/O row; the two-presses-at-once case; the sidebar and `returned` acceptance check; a stand-in in a pane marked `@reader`, and one in a pane marked `@reader_slot`, are not counted; a second session whose waiting Claude has the same window index as `main`'s `shell` tab and an older `@claude_waits` is never selected (the press sequence from `shell` is unchanged, not merely the session name); `run-shell` of the helper is silent and exits 0. Exits `$((fails > 0))`.

**Acceptance Criteria:**
- Given the success-signal server on `shell`, when Alt+0 is pressed four times through the outer server, then the current window goes B, A, C, B.
- Given that server on `shell`, when `send-keys M-0 M-0` sends two presses at once, then it ends on A.
- Given any press, then sidebars change only where `❯` moves and no pane shows output or `returned`.
- Given the repo root, when `./tabrail-jump-test` runs, then every line reads `ok` and it exits 0.

## Implementation Notes

- `tabrail-jump` (34 lines) follows the KEEP design; the awk keys panes by window index as `model::tabs` does, and the unset sentinel is the string `"999999999999"` (a numeric literal can print as `1e+12` under mawk, which `sort -n` reads as 1).
- `tabrail-jump-test` (92 lines; `default-shell /bin/bash` in its conf, so `exec -a` does not depend on the user's shell): `list-clients` also lists every sidebar's control-mode client, so the "main kept" check filters `client_control_mode`. The `@reader_slot` pane is marked after every `ensure` has run (`ensure` kills slots while `@tabrail_reader` is unset), and the R/S/G checks assert their pane setup so none passes vacuously.
- Mutation check (each run alone): `-b` on the binding, `list-panes -a`, no sidebar skip, no reader/slot skip, first pane only, any non-empty `@claude_waits`, the index sorted as text, busy states in the order, no `claude` gate, selecting the current tab, printing: each fails at least one check.
- Back-to-back runs of a `tabrail-test-lib` test race: `kill-server` in `cleanup` returns before the server has exited, so a run started at once can attach to the dying server ("server exited unexpectedly"). Leave a few seconds between runs.

## Spec Change Log

### 2026-11-27 -- review pass 1 (bad_spec)

- **Trigger:** review found no committed regression test (verification-gap, blind hunter, intent audit; medium). Every helper in README's Files table has a `tabrail-*-test` on `tabrail-test-lib`. The plan's "Scratch harness (not committed)" left Alt+0's order, the queueing that depends on leaving out `-b`, and the parity with `model::tabs` unguarded once the scratch file is gone. The scratch run's "other session" check passed regardless of the code, and `@reader_slot` was never exercised.
- **Amended:**
  - Tasks: a committed `tabrail-jump-test` replaces the scratch harness. The README wording now says "in this session" and separates Claude's `✓ 3m` from a command's `✓`. The binding comment names the cost of leaving out `-b`.
  - Acceptance: the test passes.
  - Code Map: line numbers, `tabrail-test-lib`, `tabrail-colour-test`'s shape, the README status bullet.
  - Design Notes: the full content-pane rule and the numeric sort.
  - Verification: `./tabrail-jump-test` instead of the inline harness on `-L test`/`-L outer`.
- **Known-bad state avoided:** Alt+0 merged with only a throwaway scratch file ever checking it.
- **KEEP:**
  - The pass-1 helper's design:
    - one `list-panes -s -t SESSION_ID`;
    - awk with the exact `model::tabs` pane rule, `@reader` and `@reader_slot` included;
    - a numeric three-key sort (tier, waits, index);
    - `select-window -t "$session:$next"` only when `$next` differs from `WINDOW_ID`;
    - tmux stderr to /dev/null and `exit 0`;
    - 39 lines.
  - The binding line exactly as the frozen block gives it, under prefix M.
  - The README Alt+0 bullet right after the Mouse bullet.
  - In the test:
    - `status off` in the inner server;
    - an `after-select-window` hook that counts selects, for the "no `select-window`" checks;
    - the sidebar compared before and after with `❯` blanked and the clock line dropped.

## Review Triage Log

### 2026-11-27 -- Review pass

- verdicts: 24 findings -- high 0, medium 3, low 15, false 6, maybe-false 0
- findings:
  - `[false]` `[reject]` Intent audit: the content pane also skips `@reader`/`@reader_slot`, beyond the parenthetical "skip `@sidebar` 1 panes". The frozen rule is "picked as `model::tabs` does", and `src/model.rs:30` skips all three; the parenthetical summarises that rule rather than narrowing it.
  - `[low]` `[reject]` Intent audit: `@claude_waits` in milliseconds or with a fraction sorts after unset or as unset. The hook writes `$EPOCHSECONDS` (whole seconds, `tmux-claude-state:11`; story 2 uses it), so only a foreign hook could trigger it, and the fix restructures the sort keys.
  - `[false]` `[reject]` Intent audit: order is tier, then window index, until story 2 sets `@claude_waits`. SPEC Assumptions: a tab without it "sorts last in its tier". The frozen intent puts the hook in story 2 ("edit `~/Dev/vmConfig`" is a Never).
  - `[low]` `[reject]` Intent audit: the README bullet says the hook sets `@claude_waits`, but "Optional inputs" does not list it. Story 2's frozen intent owns that README line ("tabrail's README lists it among the hook's optional inputs"); out of scope by intent here.
  - `[low]` `[reject]` Intent audit: a window linked twice into the same session appears twice in the order. That needs a hand-made `link-window` into the same session, which is never met in everyday use, and the fix adds a dedupe stage.
  - `[medium]` `[bad_spec]` Intent audit R7: the diff carries no test; the evidence is a scratch harness. Grouped with the committed-test entry. Amended Tasks, Acceptance, Code Map and Verification (Spec Change Log, pass 1).
  - `[medium]` `[bad_spec]` Blind hunter: no committed regression test, while every helper in README's Files table has a `tabrail-*-test`. Same entry; same amendment.
  - `[low]` `[reject]` Blind hunter: README names `@claude_waits` as set by the hook, but it is undocumented and unset. Same as the intent-audit row: story 2's frozen intent owns the Optional-inputs line and the hook.
  - `[low]` `[reject]` Blind hunter: the `999999999999` sentinel breaks for millisecond values. Same as the intent-audit R4 row: the hook writes whole seconds.
  - `[false]` `[reject]` Blind hunter: a `done` tab already seen stays at the front. `@claude done` lasts until the next prompt (`tmux-claude-state:18`); the frozen intent orders every `done` tab by its wait start and forbids state files or polling, and SPEC counts a done Claude as waiting for the user.
  - `[false]` `[reject]` Blind hunter: unconditional root-table `M-0` with no opt-out. The frozen intent prescribes exactly `bind -n M-0 …`; SPEC is for the one user, whose `tmux.conf` leaves M-0 free (Alt+1..9 at lines 48-56).
  - `[low]` `[patch]` Blind hunter: the README bullet's "of the session that has waited longest" reads as the session waiting; "`✓` one" collides with the plain `✓` of a finished command (README:81). Moot under bad_spec; folded into the amended README task so the re-derivation carries it.
  - `[low]` `[reject]` Blind hunter: no guard for missing args. The binding always passes both ids (`tabrail.tmux:35`); only a call by hand reaches it, and the fix adds a guard branch.
  - `[false]` `[reject]` Blind hunter: no feedback when there is nothing to go to. The frozen intent says "prints nothing … Nothing to go to: no `select-window`", and SPEC Assumptions say Alt+0 "does nothing".
  - `[low]` `[reject]` Blind hunter: the story's Design Notes, Code Map and Verification contradict the code. The fix edits this build's spec (the bad_spec amendment refreshed those sections anyway).
  - `[low]` `[patch]` Blind hunter: the binding comment names only the upside of leaving out `-b`, not that each press holds the client's keys. Moot under bad_spec; folded into the amended `tabrail.tmux` task.
  - `[medium]` `[bad_spec]` Verification gap: the only check on the helper and binding is an uncommitted scratch harness, so a later `-b` or a `model::tabs` change breaks silently. Pre-verified; same entry; same amendment.
  - `[low]` `[bad_spec]` Verification gap: the scratch harness's "other session never visited" check passes regardless of the code. Grouped with the committed-test entry; the amended test task requires a same-index other-session Claude and an unchanged press sequence.
  - `[low]` `[bad_spec]` Verification gap: the `@reader_slot` skip is never exercised. Grouped with the committed-test entry; the amended test task requires a stand-in in a `@reader_slot` pane.
  - `[low]` `[reject]` Edge case: millisecond or fractional `@claude_waits`. Same as the intent-audit R4 row.
  - `[low]` `[reject]` Edge case: a window linked twice with a tab tied between them. Same as the intent-audit linked-window row.
  - `[low]` `[reject]` Edge case: two clients on one session press Alt+0 in the same instant; their queues are separate. One user and two attached clients pressing simultaneously is not met in everyday use, and the fix adds `flock` and a re-read.
  - `[low]` `[reject]` Edge case: called without SESSION_ID/WINDOW_ID. Same as the blind hunter's missing-args row.
  - `[false]` `[reject]` Edge case (claim): merged before story 2, Alt+0 cycles by window index. Same as the intent-audit row: the SPEC assumption and the story split make that intended.

### 2026-11-27 -- Review pass

- verdicts: 25 findings -- high 0, medium 2, low 15, false 8, maybe-false 0
- findings:
  - `[low]` `[reject]` Intent audit: the README presents `@claude_waits` as coming from the hook, which does not set it yet. carried: pass 1, README bullet vs Optional inputs; story 2's frozen intent owns the hook and that README line.
  - `[false]` `[reject]` Intent audit R1: the reader/slot skip goes beyond the parenthetical. carried: pass 1, `model.rs:30` is the rule.
  - `[low]` `[reject]` Intent audit R2: millisecond, decimal or negative `@claude_waits` are untested and sort as unset or after it. carried: pass 1, the hook writes `$EPOCHSECONDS`.
  - `[false]` `[reject]` Intent audit: the production layout is untested, with the reader bar on and `tabrail-reader follow` swapping panes between queued presses. A scratch run on the lib's default conf with the reader bar on gave `B/B A/A C/C B/B` (tab/reader tab) and two quick presses ended on A. Both pane kinds are skipped wherever they sit.
  - `[false]` `[reject]` Intent audit: the tests send `M-0` without `extended-keys` or a real terminal. The binding uses the same root-table key path as the user's Alt+1..9 (`~/Dev/vmConfig/tmux.conf:48-56`); nothing in the helper depends on key encoding.
  - `[false]` `[reject]` Intent audit R4: stderr from awk/sort/cut and the missing-args path. awk/sort/cut have no error path on any `list-panes` output (fixed program, fixed keys); missing args is carried from pass 1 (reject, unreachable from the binding).
  - `[false]` `[reject]` Intent audit: the sidebar check compares two different sidebar panes. Both draw the same session's rows from the same state, so a name, status or drawing change from the press would differ; only `❯` is per tab, and it is blanked.
  - `[false]` `[reject]` Intent audit: no `@finished`, `✗` or bell tab in the tests. The helper never reads `@finished`, exit codes or bells; a tab enters the order only with `@claude` `waiting`/`done` and `claude` running, and D covers leftover `@claude`.
  - `[low]` `[patch]` Verification gap: no tab has claude first while a marked pane is active, so dropping the sidebar skip passed. Added tab H (waiting stand-in, its sidebar active) expecting `H B A C`; deleting `$3 == "1" ||` now fails it.
  - `[medium]` `[patch]` Verification gap: only `idle` is tested as outside the order; busy states never appear. "Only B left" now sets A `thinking`, C `writing`; letting non-idle states in now fails "only B left" and "none left".
  - `[low]` `[patch]` Verification gap: the index tie-break is only tested with one-digit indices. E and F now sit at 9 and 10, expecting `B E F A`; `-k3,3` without `n` now fails it.
  - `[low]` `[patch]` Blind hunter: the "no `select-window`" checks only ever see `@selects == 0`. The first `jump shell 4` check now also requires `@selects` 4.
  - `[false]` `[reject]` Blind hunter: the test never runs with the reader bar. Same scratch run as the intent-audit production-layout row: B A C B with the reader following, two presses end on A.
  - `[medium]` `[defer]` Blind hunter: `tabrail-test-lib`'s `cleanup` returns before `kill-server` has finished, so a test started at once can hit the dying server ("server exited unexpectedly"). Pre-existing in the shared lib, untouched here; the implementer hit it in back-to-back runs. Deferred.
  - `[low]` `[defer]` Blind hunter: AGENTS.md's tmux pitfalls lack this build's lessons (`list-clients` lists the sidebars' control clients; a script named `claude` shows as `bash`; a key binding's `run-shell` without `-b` holds that client's keys). The fix edits agent-context files. Deferred.
  - `[low]` `[patch]` Blind hunter: the M-0 comment in `tabrail.tmux` is 137 columns, where the file's other comments wrap at 104 or less. Wrapped to two lines.
  - `[low]` `[reject]` Blind hunter: the story's KEEP and Design Notes carry stale facts (39 lines, a scratch run). The fix edits this build's spec.
  - `[low]` `[patch]` Blind hunter: "a tab without it last" reads as last of all and omits non-numeric values. Now "a tab without it, or with one that is not a number, is last among the waiting or the done ones".
  - `[false]` `[reject]` Blind hunter: README Setup does not say `tabrail.tmux` takes `M-0`. carried: pass 1, unconditional root-table `M-0` (frozen binding; Setup lists no bindings, prefix M included; Alt+0 is in the behaviour bullets and the Files table).
  - `[low]` `[defer]` Blind hunter: `~/Dev/vmConfig/tmux.conf:37-39`'s key list does not mention Alt+0. Story 1's frozen intent forbids editing vmConfig; story 2 commits there anyway. Deferred to story 2.
  - `[low]` `[reject]` Blind hunter: em-dashes in the triage log against the user's no-em-dash rule. The fix edits this build's spec (the log headings now use `--` anyway).
  - `[low]` `[reject]` Edge case: a window linked twice into the session. carried: pass 1, linked-window row.
  - `[low]` `[patch]` Edge case: the stand-ins need a default-shell with `exec -a` (dash/fish lack it). The test conf now sets `default-shell /bin/bash`.
  - `[low]` `[reject]` Edge case: a fixed `settle 0.6` per press can read `cur` before the select lands on a loaded host. Every sibling `tabrail-*-test` uses fixed settles; the fix adds bounded polling; not met in everyday runs (passed in every run here).
  - `[low]` `[reject]` Edge case: a late queued `ensure` could remove the hand-marked `@reader_slot` pane. The S check asserts the pane is there, so it fails loudly rather than passing; `ensure`'s lock (`/tmp/tabrail-$UID.lock`) is shared with the user's live sidebars, so waiting on it would also block on them, and hooks not yet fired would still be missed.

## Design Notes

One `list-panes -s -t SESSION_ID -F` (window id and index, `@sidebar`, `@reader`, `@reader_slot`, `pane_active`, `pane_current_command`, `@claude`, `@claude_waits`). The awk step:
- skips panes with `@sidebar`, `@reader` or `@reader_slot` 1;
- keeps one content pane per window (the first; the active one wins), as `model.rs:26-46` does;
- keeps live `claude` tabs whose `@claude` is `waiting` (tier 0) or `done` (tier 1);
- prints `<tier> <waits; unset or not all digits: 999999999999> <index> <window_id>`.

Then `sort -k1,1n -k2,2n -k3,3n`, which compares the raw digits numerically, so no padding is needed. The ids go into an array. The next one after `WINDOW_ID`, wrapping, else the first, is selected with `select-window -t SESSION_ID:ID`, and only when it differs from `WINDOW_ID`. tmux stderr goes to /dev/null, then `exit 0`. The pass-1 helper (39 lines) passed a scratch run of every I/O row.

Test notes: a plain `#!/bin/bash` script named `claude` shows as `bash`, because tmux reads argv0; hence `exec -a claude sleep 1d`. Give the inner server `status off` so screen captures see only the sidebars.

## Verification

**Commands:**
- `cargo build --release --offline && for f in tabrail-jump tabrail-jump-test; do bash -n $f; done && wc -l tabrail-jump tabrail-jump-test` -- expected: builds, both parse, each under 100 lines.
- `./tabrail-jump-test` from the repo root -- expected: every line `ok`, exit 0. Afterwards `tmux -L tabrail-jump ls` and `tmux -L tabrail-jump-o ls` report no server, and `git -C ~/Dev/vmConfig diff --stat tabrail-groups.conf` shows the same as before the run. Never run it twice at once (fixed sockets), never `pkill -f`.

## Auto Run Result

Status: done

**Summary:** Alt+0 goes to this session's Claude tab that has waited longest: every `● waits` tab, then every Claude `✓` done tab, each oldest first by `@claude_waits` (unset or not a number last in its tier, ties by window index). Pressing again goes to the next tab in that order, wrapping. Only tabs whose content pane (picked as `model::tabs` does) runs `claude` count. With nothing to go to, it does nothing, silently. Until story 2 sets `@claude_waits`, the order within a tier is by window index.

**Files changed:**
- `tabrail-jump` (new, 34 lines): the helper, one `list-panes -s` plus at most one `select-window`.
- `tabrail.tmux`: `bind -n M-0 run-shell …tabrail-jump` without `-b`, with a two-line comment.
- `README.md`: the Alt+0 bullet; Files table rows for `tabrail.tmux`, `tabrail-jump`, `tabrail-jump-test`.
- `tabrail-jump-test` (new, 92 lines): 14 checks on throwaway servers with real M-0 keys.
- This story file: the bad_spec amendment, triage logs, notes.

**Review findings:**
- **Pass 1** (24 findings): one bad_spec loop. A committed test replaced the scratch harness: 3 medium findings, plus 2 low ones grouped with them. The 2 low README/comment patches were folded into the amendment. All other findings were rejected (6 false, 13 low); the reasons are in the pass 1 triage log.
- **Pass 2** (25 findings):
  - 7 patches: 1 medium (busy states tested), 6 low (active-sidebar tab, 9-before-10 tie, select counter, `default-shell`, comment wrap, README tier wording).
  - 3 deferred: the test-lib cleanup race (medium), the AGENTS.md pitfalls (low), the vmConfig key comment, for story 2 (low).
  - 15 rejected (8 false, 7 low), each with its reason in the pass 2 triage log.
- **Follow-up review recommended:** false. Patched at entry verdict: 1 medium, 6 low, 0 high.

**Verification:**
- `cargo build --release --offline` builds; both scripts pass `bash -n`; they are 34 and 92 lines.
- `./tabrail-jump-test`, run alone after the patches: 14 of 14 `ok`, exit 0. No server is left on `-L tabrail-jump` or `-L tabrail-jump-o`.
- `~/Dev/vmConfig/tabrail-groups.conf` has md5 `02bb1e64…` before and after every run; its existing uncommitted diff predates this run.
- The spec's pass-1 inline harness (B A C B, two at once A) passed on `-L test`/`-L outer`.
- A scratch run with the reader bar on passed: B/B A/A C/C B/B, and two presses at once end on A.
- The implementer's mutation runs: each of 11 deliberate breaks fails at least one check.

**Residual risks:**
- Until story 2's hook sets `@claude_waits`, Alt+0 orders by window index within each tier.
- Each press holds that client's keys while the helper runs. It is fast, but this is by design.
- Back-to-back runs of tests that use `tabrail-test-lib` can fail spuriously (deferred).
