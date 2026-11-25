---
title: 'Speed test at 5 and 50 tabs'
type: 'chore'
created: '2026-11-24'
status: 'done'
baseline_commit: '55a2917b84a853fff57260be0d0cc2d701a5f313'
route: 'dispatch'
review_loop_iteration: 1
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
- Decision (2026-11-24, review loop 1): the 10/s load starts a call every 100 ms unless 4 are still running (like 4 busy Claudes, each blocked on its hook), and the line reports the rate reached as `hooks=N/s`.
- Output per N, one line: `tabs=50 switch=..ms max=..ms switch@hooks=..ms hooks=../s reader=..ms idle=..% hook=..ms`. Exit 0 unless the setup times out.
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

- `tabrail-test-lib` -- servers `-L tabrail-$L`/`-$L-o`, `inner` (sets `spid`, the server pid), `t`/`o`, `rd`, `own` (a tab's own panes), `settle`, `cleanup` (EXIT trap; menu files by server pid; removes `$T`).
- `tabrail-reader-test` -- style of a test script on top of the lib.
- `tabrail.tmux` -- `after-new-window` ensure hook gives each new tab its sidebar and slot (one `tabrail-tmux ensure` at a time: ~50 s for 50 tabs).
- `tabrail-reader:53` -- the bar moves by `swap-pane`: the reader pane keeps its pane id, so it can be polled by id.
- `src/main.rs` -- today every sidebar, hidden ones too, redraws on each notification and each second; story 2 stops hidden ones, so a tab shown again first shows its last frame.
- `~/Dev/vmConfig/tmux-claude-state` -- read only; called with the inner server's socket.
- `README.md` -- Files table: add the script.

## Tasks & Acceptance

**Execution:**
- [x] `tabrail-perf-test` (new, executable) -- per Boundaries and Design Notes.
- [x] `README.md` -- Files table row for `tabrail-perf-test`: what it measures, `IDLE=`, `SETTLE=`, `HOOK=`, how long `5 50` takes.
- [x] Implementation Notes -- the baseline output of `tabrail-perf-test 5 50` on today's main.

**Acceptance Criteria:**
- Given no tabrail test servers, when `tabrail-perf-test 5 50` runs, then it prints two complete lines and exits 0, and afterwards `tmux -L tabrail-pt ls` and `tmux -L tabrail-pt-o ls` fail and no `$XDG_RUNTIME_DIR/tabrail/<its pids>-*` file exists.
- Given the same run twice, then switch medians differ by less than a third.
- Given the target tab's sidebar stopped (`kill -STOP`) before one switch, when that switch is timed, then it is reported as timed out on stderr, not matched on the tab's old frame.
- Given a 50-tab run on today's main, when it measures idle after the settle, then the two halves agree within 25% (no stderr warning).

## Design Notes

- **Settle before measuring.** After `ready`, wait `SETTLE` s (default 60 + 2 per tab) before idle, switches or hooks. Measured at 50 tabs on main, 15 s windows from ready: 70 72 72 60 61 61 44 34 35 34 35 35 35 36 35 37 %; steady from ~105 s, matching the live server's 35%. Loop 1 measured idle and the plain pass inside this transient. Order per N: settle, idle (two halves; a half more than 25% off the other gets a stderr warning), plain pass, 100 hook calls, busy pass.
- **No stale frames.** Before each switch, while tab K is hidden, rename it to a name unique to that switch (`tabK-i`), then the pause between switches, then select and wait for `❯ tabK-i `. An old frame cannot show that name.
- **Switch order.** Over each pass every tab is a target when N ≤ 20, and targets spread over 1..N otherwise; never the current tab.
- **Reader wait.** Take the reader's pane id once per pass and poll `t display -p -t <id> '#{window_index}'` with a few ms pause, not an unpaced `list-panes -a` on the server being timed.
- **Timeouts.** Each wait ends after 10 s. Count timed-out waits per N; any → one stderr line naming N and the count. The stdout line and the exit code stay as the Boundaries set them.
- **Hook load (decision).** Start a call every 100 ms unless 4 are running (`wait -n`); `hooks=` is calls finished over the pass's duration, `n/a` without a hook. On stop, wait for the calls in flight.
- **Input.** Counts must match `^[1-9][0-9]*$` and be ≥ 2 (else exit 2; `010` is not octal 8). INT exits 130, TERM 143.

## Implementation Notes

Loop 2 (current baseline; supersedes loop 1's numbers and its notes on the hook load):

Baseline on main 55a2917 (release build), f4l002, 4 cores, tmux 3.8, bash 5.3, 2026-11-24, two runs
of `tabrail-perf-test 5 50` (default SETTLE and IDLE), 709 and 712 s, nothing on stderr:

```
tabs=5 switch=16ms max=24ms switch@hooks=14ms hooks=9.8/s reader=123ms idle=0.5% hook=17.3ms
tabs=50 switch=443ms max=487ms switch@hooks=663ms hooks=4.7/s reader=1335ms idle=44.3% hook=390.0ms
tabs=5 switch=16ms max=34ms switch@hooks=16ms hooks=9.8/s reader=124ms idle=0.6% hook=17.8ms
tabs=50 switch=425ms max=470ms switch@hooks=621ms hooks=4.5/s reader=1339ms idle=43.0% hook=384.9ms
```

- AC 1: both runs exit 0 with two lines; afterwards `tmux -L tabrail-pt ls` and `-pt-o ls` fail, no
  `tmux-claude-state` call or script left, no new menu-file prefix in `$XDG_RUNTIME_DIR/tabrail`.
- AC 2: switch medians 16/16 ms and 443/425 ms (4%).
- AC 4: no idle-halves warning at 50 tabs in either run (idle 44.3%, 43.0% after 160 s of settle;
  above the 35% of the Design Notes' 15 s windows, but steady).
- AC 3: `kill -STOP` of a sidebar does not hold under tmux: the stopped sidebar was running again
  within 0.5 s (tmux continues a pane process that stops), and stopping its `tmux -C` child changed
  nothing (the sidebar kept getting replies and drawing). Checked instead by freezing the sidebar's
  output, which leaves its old frame on screen: a temporary copy sent `t send-keys -t <tab K's
  sidebar> C-s` (XOFF) before switch 3 of a 3-tab run and `C-q` after: `max=10003ms` and
  `tabs=3: 1 waits timed out (10 s after the switch)` on stderr, exit 0.
- At 50 tabs the hook load reaches ~4.6/s with 4 calls in flight (a call takes ~390 ms there).
- Checked: `010`, `1`, `x 5`: usage, exit 2. `SETTLE=5 IDLE=4 … 5`: one line, exit 0.
  `HOOK=/nonexistent SETTLE=5 IDLE=2 … 2`: `switch@hooks=n/a hooks=n/a … hook=n/a`, exit 0. Ctrl-C
  during the busy pass (inside `tmux -L ptctl`, under `bash -c`): exit 130, no server, hook call,
  `$T` or menu file left. TERM during setup: exit 143. A copy without `target/`: `tabs=2: setup
  timed out…`, exit 1 after 64 s, nothing left.
- Review fixes after this baseline: idle now prints two decimals (the lines above have one); the
  halves warn only when also more than 5 ticks (100 Hz) apart, as 5 tabs give ~15 ticks a half;
  `ready` also needs a reader pane (checked: `@tabrail_reader_started` in the future, every tab a
  sidebar and a slot, no reader: setup timed out, exit 1); counts are 2-9999 (`10000` and 20
  digits: usage, exit 2). `SETTLE=5 IDLE=4 … 5` afterwards: `idle=0.50%`, no warning, exit 0.
- After the review fixes, one more `tabrail-perf-test 5 50` (16:50, load average 1.94, 640 s,
  nothing on stderr, nothing left):
  ```
  tabs=5 switch=13ms max=17ms switch@hooks=11ms hooks=9.8/s reader=95ms idle=0.43% hook=12.9ms
  tabs=50 switch=306ms max=325ms switch@hooks=470ms hooks=6.1/s reader=982ms idle=31.22% hook=286.1ms
  ```
  The 50-tab values are ~30% below the two runs above (switch 443/425 ms, idle 44/43%): the runs
  share 4 cores with the live tmux and its Claudes, so a story's before and after belong in
  back-to-back runs (main, then the branch), not against these lines.

Loop 1 (code reverted in review; its numbers were taken during the post-setup transient, see Design Notes):

Baseline on main 55a2917 (release build), f4l002, 4 cores, tmux 3.8, 2026-11-24, two runs of
`tabrail-perf-test 5 50`, 384 s each:

```
tabs=5 switch=13ms max=32ms switch@hooks=13ms reader=100ms idle=0.5% hook=14.1ms
tabs=50 switch=451ms max=606ms switch@hooks=521ms reader=1177ms idle=68.7% hook=308.0ms
tabs=5 switch=12ms max=20ms switch@hooks=12ms reader=93ms idle=0.5% hook=13.1ms
tabs=50 switch=396ms max=465ms switch@hooks=553ms reader=1146ms idle=68.5% hook=311.7ms
```

Switch medians between the runs: 13/12 ms and 451/396 ms (12%). After each run neither test server
was left and no new menu-file prefix was left in `$XDG_RUNTIME_DIR/tabrail`.

- The 10/s load runs one hook call at a time (a call every 100 ms, or when the one before is done).
  A fixed 10/s in background calls piled up at 50 tabs: a call takes ~300 ms there, so 557 calls
  were in flight and the server sat at 87% CPU. At 5 tabs the load reaches ~9.8/s; at 50 tabs on
  today's main it reaches only ~2.7/s, so this baseline's `switch@hooks` at 50 understates CAP-3.
  Once a call is under 100 ms (stories 2 and 4) the load is 10/s.
- The outer server is `N + 10` rows high (at least 46): the sidebar does not scroll, and 50 tabs plus
  the 4-row reader bar do not fit in 46 rows.
- Every wait (sidebar, reader) ends after 10 s; a value of ~10000 ms means it never arrived.
- `HOOK=<path>` points at another hook (missing: `hook=n/a switch@hooks=n/a`); `IDLE=<s>` shortens
  the idle measurement for a quick run. Counts under 2 are refused (exit 2): no tab to switch to.
- Setup at 50 tabs takes ~50 s (one `tabrail-tmux ensure` per new tab, one at a time); the timeout
  is 60 s + 2 s per tab.
- Checked: `IDLE=2` with Ctrl-C during the hook load (servers, load loop, temp dir and menu files
  gone); `HOOK=/nonexistent` (n/a fields, exit 0); the sidebar binary moved away (`tabs=2: setup
  timed out…`, exit 1, nothing left).
- The test servers share `/tmp/tabrail-$UID.lock` and `/tmp/tabrail-reader-$UID.lock` with the
  user's live tmux: a run can delay a live `tabrail-reader follow` (it gives up after 5 s), and a
  live tab switch during a run shows up in that switch's reader time.

## Spec Change Log

- **Loop 1 (2026-11-24).** Triggers: BH1+EC11 (intent_gap: one hook call at a time reached ~2.7/s at 50 tabs; resolved by the human: cap of 4 in flight, `hooks=N/s`), BH7 (bad_spec: idle taken in the post-setup transient), plus the patch findings (VG1 stale frame, reader poll, timeouts, switch order, octal count, TERM, README knobs). Amended: Boundaries decision and output line (human), Code Map, Tasks, ACs 3-4, Design Notes, Verification. Avoids: baselines read 2× high (idle 68.7% vs steady 35%), `switch@hooks` read as 10/s, and story 2 passing on old frames. KEEP from loop 1: the lib's `inner`/`t`/`o`/`rd`/`own`/`cleanup` with `L=pt`; outer `N + 10` rows high (at least 46: the sidebar does not scroll); `ready` = one `list-panes -a` counted per window (a sidebar and the reader or a slot) plus the outer showing `❯ tab1 ` and ` tabN `; setup timeout 60 s + 2 s per tab, message naming N, exit 1; µs from `EPOCHREALTIME` without forks; `cpu` reads `/proc/<spid>/stat` after the `) `; median via `sort -n` + awk; EXIT trap stops the load loop then `cleanup`; `HOOK=`/`IDLE=` knobs; `t kill-server; o kill-server` between counts. A full `5 50` took 384 s.

## Review Triage Log

Loop 1 (blind-hunter BH, edge-case-hunter EC, verification-gap VG):

| Finding | Verdict | Route | Evidence |
|---|---|---|---|
| BH1+EC11 `switch@hooks` load is one call at a time, not calls arriving 10/s; rate not printed (`tabrail-perf-test:77`) | medium | intent_gap | ~2.7/s at 50 tabs on main; open-loop 10/s piled up 557 calls. What a slow hook should do (pile up, throttle, cap in flight) is not settled by the intent. |
| VG1 switch timer can stop on a stale frame: `❯ tabK` from the tab's last visit (`:45`) | high | patch | Pre-verified: a never-redrawn pane matched in 8 ms after switching back; the busy pass revisits the plain pass's 20 tabs, N=5 revisits 17 of 20. Valid today only because hidden sidebars still redraw; story 2 removes that. Rename tab K uniquely before each switch. |
| BH7 50-tab idle taken while still settling (`:67`) | medium | bad_spec | Same setup, four 30 s windows from `settle 3`: 71.9, 65.1, 60.1, 41.5%. CAP-2 compares against it; the spec gives no settle rule. |
| BH2+EC6+VG-o2 reader wait polls `list-panes -a` unpaced on the server it times (`:46`) | medium | patch | Each poll forks a client queued in the inner server's single thread with `tabrail-reader follow`'s ~10 calls; size of the effect unmeasured. Poll the reader's pane id with `display`, with a pause. |
| BH4+EC7 timed-out waits count as ~10000 ms samples, uncounted (`:45-47`) | medium | patch | A median hides up to 9 of 20; reader and switch@hooks have no max. Report timed-out waits on stderr; line and exit code stay as the Boundaries set them. |
| BH5 test servers share `/tmp/tabrail-$UID.lock` and the reader lock with the live tmux | medium | defer | Pre-existing: lock paths in tabrail-tmux/tabrail-reader, shared by every tabrail-*-test. |
| EC4+VG-o1 switch sequence skips tabs: N=5 is `2 3 4 1` repeated, N=20 odd tabs only (`:43`) | low | patch | Computed for N=5, 20, 50. Formula change is a direct correction. |
| BH8+EC1 count `010` passes as octal 8, setup then times out (`:11`) | low | patch | `[[ $n =~ ^[0-9]+$ ]]` then `(( n ))`; regex `^[1-9][0-9]*$` fixes it. |
| EC10+BH9b TERM exits 130 (`:16`) | low | patch | One trap for INT and TERM; split it. |
| BH10 README row omits `IDLE=`, `HOOK=` and the ~6.5 min run | low | patch | Neighbouring row documents `HELPERS_ONLY=1`; direct edit. Progress lines on stderr rejected (adds code, no defect). |
| BH3 switch poll's fork cost and precision floor not recorded | low | reject | The frozen Boundaries define the interval from `t select-window` to the outer's capture, forks included; recording a floor edits this spec. |
| BH6 baseline does not name the vmConfig hook commit | low | reject | Fix edits this build's spec. |
| BH9a in-flight hook call outlives `kill "$loop"` | low | reject | It ends once the server is gone: a Ctrl-C run had no process left 4 s later; a fix adds a trap in the loop. |
| EC2 `IDLE=0` or non-numeric breaks the idle field | low | reject | A developer knob; a guard adds code. |
| EC3 a running `tabrail-pt` server is reused and killed | low | reject | Same fixed `-L tabrail-$L` in every lib test; a guard adds code. |
| EC5 reader timed only after the switch mark, so reader ≥ switch | low | reject | The reader follows after the hooks by design (1177 vs 451 ms); interleaving adds code. |
| EC8 a no-op hook (no jq, `HOOK` a directory) is timed | low | reject | Unlikely on this machine; guards add code. |

Loop 2 (BH, EC, VG on the re-derived script):

| Finding | Verdict | Route | Evidence |
|---|---|---|---|
| VG2-1+BH2-1+EC2-10 AC 3 and its manual check use `kill -STOP`, which tmux undoes (SIGCONT); the working XOFF check is recorded only in Implementation Notes | medium | defer | Pre-verified: a stopped pane process ran again within 50 ms on tmux 3.8. The fix edits this spec's AC/Verification and AGENTS.md (pitfall), so it goes to the ledger. |
| VG2-o1 `tabrail-perf-test` matches story 2's `for t in ./tabrail-*-test` sweep: +~12 min, always exit 0 | medium | defer | `2-hidden-sidebars-stay-idle.md:84`, SPEC.md:44. The name is set by the frozen intent; the fix is in stories 2-4's verification. |
| EC2-1+BH2-5 no `@reader` pane: `r` empty, `display -t ''` reads the current window, reader waits match at once (`:37`, `:43`) | medium | patch | Checked: `display -p -t ''` returns the active window's index. `ready` counts a slot as `r`, so setup passes without a reader. Require the reader in `ready`. |
| BH2-8 idle printed with one decimal: ±0.05 of a 0.5% 5-tab idle is ±10% of CAP-2's 2× comparison (`:68`) | low | patch | `printf "%.1f"`; two decimals is a direct correction. |
| EC2-4 idle-halves warning fires on tick noise at 5 tabs (~15 ticks a half) (`:69`) | low | patch | 25% of 15 ticks is 4 ticks; add a floor of a few ticks to the difference. |
| EC2-7 counts have no upper bound; 20 digits wrap (`:10`) | low | patch | Regex `^[1-9][0-9]{0,3}$`: a direct correction. |
| BH2-16 README row lacks the failure signals (stderr timeouts and idle warning, ~10000 ms, exit 1) | low | patch | Direct README edit. |
| BH2-9 50-tab idle reads 43-44% after the 160 s settle against ~35% in the Design Notes' profile | low | reject | Re-profiled with this script's setup, 15 s windows from ready: 70 72 73 64 60 60 61 44 37 38 39 38 37 38 39 38 %: steady from ~120 s, so the 160 s settle clears the transient. The rest is run-to-run variance (37-44%); each run's halves agree. Rewording the Design Notes edits this spec. |
| BH2-6+EC2-2 empty `own :1` pane makes the hook a no-op that is timed | low | reject | carried: EC8 (no-op hook timed); `:84` still has no pane check. |
| BH2-10 baseline does not name the vmConfig hook commit | low | reject | carried: BH6. |
| BH2-13 switch poll forks `o capture-pane` back to back (`:42`) | maybe-false | reject | Each iteration already waits on a forked client (a few ms); a pause would coarsen the timing. If true, low: it inflates 5 and 50 tabs alike. Settle by comparing medians with a 2 ms pause once. |
| BH2-3 the 1 s pause after the rename lines up with the sidebars' 1 s tick (`:40`) | maybe-false | reject | If true, low: it biases today's baseline only; story 2's targets are measured after hidden ticks are gone. Settle by comparing medians at 1.0 and 1.5 s. |
| EC2-9+BH2-7 the Intent says 10/s while the capped load reaches ~4.6/s at 50 tabs; no warning | low | reject | The rate is printed next to `switch@hooks` by the human's decision; the frozen Intent is the human's to change. |
| EC2-3+BH2-4 TERM during the settle or an idle half waits for `sleep` to end (up to 160 s) | low | reject | INT (Ctrl-C) reaches the sleep too and exits at once; only a TERM to the script alone waits; a fix adds `& wait` per sleep. |
| EC2-5 a wedged hook call hangs the run | low | reject | Unlikely; a timeout per call adds code. |
| EC2-6 at N > 50 the rename's redraw wave may outlast the 1 s pause | low | reject | Targets are 5 and 50 tabs; a wait loop adds code. |
| EC2-8 `SETTLE` non-numeric or `IDLE=0` unchecked | low | reject | carried for `IDLE`: EC2; `SETTLE` is the same developer knob. |
| BH2-2 the loop 1 lock deferral duplicates spec-reader-bar story 5's entry | low | reject | The ledger is appended without a duplicate search by the workflow's rule. |
| BH2-11 baseline does not record machine load | low | reject | Recording it edits this spec or adds code; the idle halves already flag unsteady idle. |
| BH2-12 no check that `target/release/tabrail` exists or is newer than `src/` | low | reject | Lib-wide convention (`tabrail-test-lib` header: "Needs target/release/tabrail"); stories' verification runs `cargo build` first. |
| BH2-14 98 of 100 lines with fixes pending | low | reject | No defect; the patches must fit. |
| BH2-15 Implementation Notes keep loop 1's reverted numbers | low | reject | Fix edits this build's spec; the block is labelled reverted. |

## Verification

**Commands:**
- `bash -n tabrail-perf-test && wc -l tabrail-perf-test` -- expected: parses, under 100 lines.
- `cargo build --release --offline && ./tabrail-perf-test 5 50` -- expected: two lines; the baseline (run twice: AC 2).
- `SETTLE=5 IDLE=4 ./tabrail-perf-test 5` -- expected: one line, exit 0 (matrix: one count).
- `HOOK=/nonexistent SETTLE=5 IDLE=2 ./tabrail-perf-test 2` -- expected: `switch@hooks=n/a hooks=n/a … hook=n/a`, exit 0.
- `./tabrail-perf-test 010; echo $?` -- expected: usage, 2.

**Manual checks (throwaway servers only):**
- Ctrl-C during the busy pass, run inside `tmux -L ptctl`: no `tabrail-pt*` server, hook call, `$T` or menu file of its pids left.
- A copy of the repo without `target/`: `tabs=2: setup timed out…`, exit 1, nothing left.
- `kill -STOP` the target tab's sidebar before one switch (a temporary copy of the script): a timeout on stderr, not a fast sample.
