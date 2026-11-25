---
title: 'Claude hook in one tmux call'
type: 'refactor'
created: '2026-11-24'
status: 'done'
baseline_commit: '1a8f5b7575a48323a559ae2a56a0fec1d92fe55f'
vmconfig_baseline_commit: '7e5c6f52fe2602c196138bf2432c3b4bc6a03685'
route: 'dispatch'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/specs/spec-performance/SPEC.md'
  - '{project-root}/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `~/Dev/vmConfig/tmux-claude-state` runs on every event of every Claude (two per tool use) and starts `jq`, `date` and up to four `tmux` clients each time (SPEC CAP-4, cost-today.md).

**Approach:** Each event builds one tmux command list (its `set -w`/`set -wu` and `set-buffer -b sidebar-refresh x`, chained with `\;`) and runs one `tmux` client; `$EPOCHSECONDS` replaces `date`. `Stop` may read `@claude_since` first: at most two calls.

## Boundaries & Constraints

**Always:**
- After each event the window holds exactly the options and values today's hook leaves (I/O matrix), and the refresh buffer is touched.
- `jq` stays the one JSON parse; it prints nothing; exits 0 on every path; `TMUX_PANE` unset: exit 0 before any work.
- vmConfig is a separate repo: commit on `lde/claude-state-fast` there, push, open its PR. spec-claude-jump story 2 adds `@claude_waits` (Notification-waiting and Stop, unset on SessionEnd): if it has landed, keep it in the chained commands; if not, leave the structure so it is one more element.

**Never:** Change states, `@claude_took`'s format, `~/.claude/settings.json` or tabrail files; run the hook against the default socket (every running Claude calls it; this shell's `TMUX` is the user's server).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| SessionStart | | `@claude` idle | N/A |
| UserPromptSubmit | | thinking, `@claude_since` now, no `@claude_took` | N/A |
| PreToolUse Edit/Bash/Read/WebFetch/Agent/other | | writing/running/reading/searching/delegating/thinking | N/A |
| PostToolUse | | thinking | N/A |
| Notification permission | | waiting | N/A |
| Notification idle_prompt | | nothing but the buffer | N/A |
| Stop | since 125 s ago | done, `@claude_took` 2m5s | since unset: 0s |
| SessionEnd | | no `@claude*` options | N/A |
| Outside tmux | no `TMUX_PANE` | nothing | exit 0 |

</frozen-after-approval>

## Code Map

- `~/Dev/vmConfig/tmux-claude-state` -- the hook: `setw`/`unsetw` helpers, one case per event, `set-buffer` last.
- `~/.claude/settings.json` -- which events call it; read only.
- `tabrail-perf-test` (story 1) -- prints hook ms per call.
- `_bmad-output/specs/spec-claude-jump/stories/2-record-when-claude-starts-waiting.md` -- the other change to this file.

## Tasks & Acceptance

**Execution:**
- [x] `~/Dev/vmConfig/tmux-claude-state` -- one chained `tmux` call per event (two for Stop).
- [x] Scratch check (not committed) -- feed every matrix event to the old hook (`git -C ~/Dev/vmConfig show HEAD:tmux-claude-state`) and the new one on two panes of a throwaway server; `show -w` of both match.

**Acceptance Criteria:**
- Given the scratch check, then every event leaves identical window options in both panes.
- Given 100 calls against a throwaway server, then together under 1 s (SPEC CAP-4).

## Implementation Notes

vmConfig `lde/claude-state-fast`, commits 511aa27 and 0762713, PR LynnDelpy/vmConfig#7. `setw`/`unsetw`
append `; set -w(u) -t PANE ...` to an array; one `tmux set-buffer -b sidebar-refresh x "${c[@]}"` at
the end; `now=$EPOCHSECONDS`. `set-buffer` leads the list: a failing command (pane gone) skips the rest
of its `;` list, so last it could be lost; tmux runs the whole list before a sidebar reads, so first is
safe. After review Stop is one call too: `set -wF @claude_took` with format arithmetic on `$now` and
`#{@claude_since}` (unset: 0s), the same strings as the bash formula for 0-90061 s and -5 s.
`@claude_waits` (spec-claude-jump story 2) has not landed: one `setw` in Notification and Stop, one
name in SessionEnd's `unsetw` when it does.

Scratch check (old hook from 7e5c6f5 on window @0, new on @1, a `tmux` shim refusing any other socket):
SessionStart, UserPromptSubmit, PreToolUse for 16 tools (unknown and empty included), PostToolUse,
Notification permission / idle_prompt / "waiting for your input" with a newline, Stop since 125 s
(2m5s), 3725 s (1h2m), unset (0s), SessionEnd twice, an unknown event, bad JSON, a dead pane, no
`TMUX_PANE`: `show -w` identical, stdout empty, exit 0, buffer touched, one call, no `date`. 0 fails.
Committed after review: `~/Dev/vmConfig/tmux-claude-state-test`, fixed expected values per matrix row,
one tmux call per event counted by a shim: 19 ok.

100 calls, bare throwaway server, f4l002, 2026-11-25:

| Event | Old | 511aa27 | Now (3 rounds) |
|---|---|---|---|
| PreToolUse / PostToolUse | 1230-1337 ms | 869-899 ms | 823-978 ms |
| UserPromptSubmit | 1770-1800 ms | 852-872 ms | 808-885 ms |
| Stop | 1794-1859 ms | 1225-1297 ms (two calls) | 819-856 ms |

- AC 2 met for every event, with little margin (978 ms worst): bash, jq and one tmux client are most
  of the ~8.5 ms a call costs, and the intent keeps all three.
- `tabrail-perf-test 5 50`, before (main after story 2) and now:
  ```
  before tabs=5 switch=8ms max=13ms switch@hooks=9ms hooks=9.8/s reader=99ms idle=0.08% hook=13.9ms
  before tabs=50 switch=18ms max=35ms switch@hooks=16ms hooks=9.8/s reader=132ms idle=0.38% hook=15.5ms
  now    tabs=5 switch=8ms max=27ms switch@hooks=8ms hooks=9.8/s reader=104ms idle=0.07% hook=8.6ms
  now    tabs=50 switch=18ms max=39ms switch@hooks=15ms hooks=9.9/s reader=128ms idle=0.35% hook=10.1ms
  ```
  The 5-tab `hook=` ran 8.9, 9.4, 10.6 ms on 511aa27 and 8.6 ms now: at the 10 ms line, run to run.
- `~/.claude/settings.json` has no PreToolUse entry although the hook's header says it is wired for it:
  the writing/running/reading states never show live. Deferred (settings are outside this story).

## Spec Change Log

## Review Triage Log

| Finding | Verdict | Route | Evidence |
|---|---|---|---|
| No committed test of the hook's options, refresh and exit per event; the scratch comparison is lost (verification-gap, blind) | medium | patch | Pre-verified: only `tabrail-perf-test` runs the hook and checks time only; a dropped `';'` or `set-buffer` moved last passes everything; spec-claude-jump story 2 edits this file next. A fixed-expectation test is not the uncommitted old/new comparison the task names. |
| AC 2 / CAP-4 missed for Stop (1225-1297 ms per 100) (blind, edge) | medium | patch | Measured. Intent allows two calls, not requires them; `set -wF @claude_took` with format arithmetic gives the same strings in one call (reviewer checked on tmux 3.8). |
| The comment does not say why `set-buffer` first is race-free (blind) | low | patch | One comment line: tmux runs the client's whole list before any sidebar's query. |
| `~/.claude/settings.json` has no PreToolUse entry; header and cost-today.md say it is wired (blind) | medium | defer | Checked: settings call the hook for 6 events, not PreToolUse. Pre-existing; settings are outside this story. |
| Empty `c` (idle_prompt, unknown event, bad JSON) still runs tmux (edge) | false | reject | The intent requires the buffer touched on every event; matrix row idle_prompt: "nothing but the buffer". |
| `EPOCHSECONDS` undefined on bash < 5 (edge) | false | reject | vmConfig runs on f4l002 only, bash 5.2. |
| `TMUX_PANE` without `TMUX` writes on the default server (edge) | false | reject | Same as the old hook; a Claude in a tmux pane always has both. |
| CAP-3 lines (`switch@hooks`, `hooks=`) not recorded (blind) | low | reject | Fix edits this build's spec; recorded at verification. |
| `hook=` over 10 ms in 1 of 3 runs, cause unproven; cost breakdown short by ~2 ms (blind) | low | reject | Fix edits this build's spec; re-measured at verification. |
| Code Map still says "`set-buffer` last" (blind, verification-gap) | low | reject | Fix edits this build's spec; the hook's comment states the order and why. |
| vmConfig checkout on the feature branch, the hook live before review (blind) | low | reject | Process: PR #7 merges at finish (vmConfig rule) and main is pulled. |
| Stray line breaks in the notes (blind) | low | reject | Fix edits this build's spec. |

## Verification

**Commands:**
- `bash -n ~/Dev/vmConfig/tmux-claude-state` -- expected: parses.
- `./tabrail-perf-test 5` -- expected: `hook=` under 10 ms.
