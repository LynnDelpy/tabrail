---
title: 'Claude hook in one tmux call'
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
- [ ] `~/Dev/vmConfig/tmux-claude-state` -- one chained `tmux` call per event (two for Stop).
- [ ] Scratch check (not committed) -- feed every matrix event to the old hook (`git -C ~/Dev/vmConfig show HEAD:tmux-claude-state`) and the new one on two panes of a throwaway server; `show -w` of both match.

**Acceptance Criteria:**
- Given the scratch check, then every event leaves identical window options in both panes.
- Given 100 calls against a throwaway server, then together under 1 s (SPEC CAP-4).

## Implementation Notes

## Spec Change Log

## Review Triage Log

## Verification

**Commands:**
- `bash -n ~/Dev/vmConfig/tmux-claude-state` -- expected: parses.
- `./tabrail-perf-test 5` -- expected: `hook=` under 10 ms.
