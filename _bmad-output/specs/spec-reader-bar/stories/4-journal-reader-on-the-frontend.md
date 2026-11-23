---
title: 'Journal reader on the frontend'
type: 'chore'
created: '2026-11-23'
status: 'ready-for-dev'
route: 'dispatch'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/specs/spec-reader-bar/SPEC.md'
  - '{project-root}/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** The journal reader chat still runs Claude Code's TUI and is told it sits 26 columns wide under the sidebar (SPEC CAP-4).

**Approach:** `~/Dev/vmConfig/journal-reader/start` execs `tabrail-chat` with the options it gives `claude` today, falling back to the TUI when the binary is missing; its CLAUDE.md describes the bar and asks for plain-text answers that fit it.

## Boundaries & Constraints

**Always:**
- `start`: same `cd`, proxy, `CLAUDE_JOURNAL=off`, `unset TMUX_PANE CLAUDECODE` and `--continue` logic; then `exec "$chat" --model claude-haiku-5-5 "${again[@]}"` with `chat=$(tmux show -gv @tabrail_dir 2>/dev/null)/target/release/tabrail-chat` when executable, else `exec claude …` as today. Header comment names both.
- `CLAUDE.md`: "small tmux pane under the sidebar … lines under 80 columns, no wide tables" becomes: a 4-row bar across the bottom of the tab that shows the last 3 lines of an answer; answer in plain text (no markdown tables or headings), in at most 3 lines when the question allows, citations included. Rules 1-6 unchanged.
- `.claude/settings.json` (journal-reader) unchanged: `claude -p` ignores its TUI keys, the fallback still uses them, the permissions apply to both.
- Commit in `~/Dev/vmConfig` on `lde/reader-bar`, push, PR.

**Never:** Change the journal, its hooks, `journal-search` or permissions; edit tabrail; restart or kill the user's running reader pane (Lynn restarts it).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Built | `tabrail-chat` present | `start` runs it with `--model claude-haiku-5-5 --continue` | N/A |
| Not built | binary missing or no tmux | `start` runs the claude TUI as today | N/A |
| First run | no transcripts in the folder's project dir | no `--continue` | N/A |

</frozen-after-approval>

## Code Map

- `~/Dev/vmConfig/journal-reader/start` -- 18 lines; the `again` array and final `exec claude`.
- `~/Dev/vmConfig/journal-reader/CLAUDE.md:3-5` -- the "You sit in a small tmux pane under the sidebar" paragraph.
- `~/Dev/vmConfig/journal-reader/.claude/settings.json` -- read only; its allow/deny lists are what `tabrail-chat` runs under.
- `~/Dev/vmConfig/journal-reader/journal-search-test` -- unaffected; run it to confirm.

## Tasks & Acceptance

**Execution:**
- [ ] `journal-reader/start` -- pick `tabrail-chat`, fall back to `claude`.
- [ ] `journal-reader/CLAUDE.md` -- the bar paragraph.

**Acceptance Criteria:**
- Given `tmux -L jr -f /dev/null` with `@tabrail_dir` set to the tabrail checkout holding a built `tabrail-chat`, when `start` runs in a 160x4 pane and "which tmux is on this VM?" is asked, then the answer streams in plain text and cites a journal entry or raw record, or gives `rpm -q tmux`.
- Given `@tabrail_dir` unset, when `start` runs, then Claude Code's TUI starts.

## Implementation Notes

## Spec Change Log

## Review Triage Log

## Verification

**Commands:**
- `bash -n ~/Dev/vmConfig/journal-reader/start && ~/Dev/vmConfig/journal-reader/journal-search-test` -- expected: parses; journal-search tests pass.

**Manual checks (if no CLI):**
- The two acceptance runs on `tmux -L jr`, then `kill-server`. Tell Lynn: `tabrail-tmux restart` keeps the old reader, so kill the reader pane once (it restarts on the next tab switch) to pick up the new `start`.
