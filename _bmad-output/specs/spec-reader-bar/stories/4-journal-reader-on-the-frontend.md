---
title: 'Journal reader on the frontend'
type: 'chore'
created: '2026-11-23'
status: 'done'
baseline_commit: 'a8f7a907b083ff0c7f9b44ab8e40430711e9b410'
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

- Where to work: `/tmp/claude-1002/-home-lde-Dev-tabrail-build/79a18e00-3106-4c66-87f8-81f5c07429be/scratchpad/vmconfig-reader-bar`, a git worktree of ~/Dev/vmConfig already on `lde/reader-bar` (from `lde/agent-journal`, vmConfig's open PR #1, where journal-reader lives). Edit and commit there only. ~/Dev/vmConfig itself is the user's live config (on `lde/agent-journal`, `.zshrc` modified, untracked files): never switch its branch, never edit, add or commit anything in it. No push and no PR during implementation: they come at the end, with this story's tabrail PR (vmConfig PR base `lde/agent-journal`).
- The acceptance runs use the worktree's `journal-reader/start` and `@tabrail_dir` = /home/lde/Dev/tabrail-build (built `tabrail-chat`), on `tmux -L jr -f /dev/null`, killed afterwards.
- `~/Dev/vmConfig/journal-reader/start` -- 18 lines; the `again` array and final `exec claude`.
- `~/Dev/vmConfig/journal-reader/CLAUDE.md:3-5` -- the "You sit in a small tmux pane under the sidebar" paragraph.
- `~/Dev/vmConfig/journal-reader/.claude/settings.json` -- read only; its allow/deny lists are what `tabrail-chat` runs under.
- `~/Dev/vmConfig/journal-reader/journal-search-test` -- unaffected; run it to confirm.

## Tasks & Acceptance

**Execution:**
- [x] `journal-reader/start` -- pick `tabrail-chat`, fall back to `claude`.
- [x] `journal-reader/CLAUDE.md` -- the bar paragraph.

**Acceptance Criteria:**
- Given `tmux -L jr -f /dev/null` with `@tabrail_dir` set to the tabrail checkout holding a built `tabrail-chat`, when `start` runs in a 160x4 pane and "which tmux is on this VM?" is asked, then the answer streams in plain text and cites a journal entry or raw record, or gives `rpm -q tmux`.
- Given `@tabrail_dir` unset, when `start` runs, then Claude Code's TUI starts.

## Implementation Notes

- vmConfig `lde/reader-bar` (from `lde/agent-journal`, PR #1): 1e4d4ae `start` picks tabrail-chat, CLAUDE.md for the bar; 3f078f8 review fixes: `$TMUX` guard, `shopt -s execfail`, `TABRAIL_CHAT_CLAUDE` unset, `journal-reader/start-test` (9 checks, in claude-journal-test's loop), CLAUDE.md bar rules (end with the answer, no markdown, short lines and citations, tools outside the allowlist refused until story 3). Built in a worktree; the live ~/Dev/vmConfig checkout was not touched.
- Acceptance: on `tmux -L jr`, a real Haiku turn in a 160x4 pane answered "which tmux is on this VM?" in plain text with a raw-record citation and `rpm -q tmux`; a second start continued the conversation; with `@tabrail_dir` unset the TUI started.
- Found: Claude Code's TUI `--continue` skips transcripts written by `claude -p`, so the TUI fallback ends with "No conversation found to continue" once the folder has only tabrail-chat transcripts (deferred-work.md).

## Spec Change Log

## Review Triage Log

### 2026-11-23 — Review pass
- verdicts: 16 findings — high 0, medium 4, low 11, false 1, maybe-false 0
- findings:
  - `[medium]` `[patch]` (verification-gap) nothing tests which program `start` runs with which options — `journal-reader/start-test`, 9 checks; 8 mutations each fail it.
  - `[low]` `[defer]` (verification-gap) tabrail-chat's tests drop `-c` on restart but never `--continue`, the flag `start` passes — tabrail is out of this story; deferred-work.md.
  - `[low]` `[patch]` (verification-gap) outside tmux `start` asks the user's server — `$TMUX` guard.
  - `[medium]` `[patch]` (blind-hunter) until story 3 tabrail-chat refuses tools outside the allowlist, so Rule 3's transcript drill-down is denied silently — CLAUDE.md says so and gives the pointer instead.
  - `[low]` `[patch]` (blind-hunter) the bar shows the end of an answer — CLAUDE.md: end with the answer and its citation.
  - `[low]` `[patch]` (blind-hunter) "no tables or headings" too narrow, headers start with `###` — no markdown at all, headers cited without `###`.
  - `[low]` `[patch]` (blind-hunter) "3 lines" but long lines wrap into more rows — short lines.
  - `[low]` `[patch]` (blind-hunter) full header citations do not fit 3 lines — short form date, time, session.
  - `[false]` `[reject]` (blind-hunter) `@tabrail_dir` unset gives `/target/release/tabrail-chat` — not executable, so the fallback runs; now also guarded by `$TMUX`.
  - `[low]` `[patch]` (blind-hunter) outside tmux tabrail-chat runs full-screen — same `$TMUX` guard.
  - `[low]` `[reject]` (blind-hunter) the fallback to the TUI is silent when tabrail-chat is not built — the deploy steps build it; a message costs a tmux call on every start.
  - `[low]` `[patch]` (blind-hunter) `TABRAIL_CHAT_CLAUDE` could leak into the reader — unset in `start`.
  - `[medium]` `[patch]` (blind-hunter) no test runs `start` — same start-test.
  - `[low]` `[patch]` (edge-case-hunter) a tabrail-chat that fails to exec ends bash with 126, no fallback — `shopt -s execfail`; start-test covers a corrupt binary.
  - `[low]` `[patch]` (edge-case-hunter) outside tmux — same `$TMUX` guard.
  - `[medium]` `[defer]` (edge-case-hunter) after a resurrect restore the reader's `claude -p` child matches `~claude` and comes back as a full TUI — needs a narrower entry plus tabrail-chat starting claude by path, and a restore test; deferred-work.md.

## Verification

**Commands:**
- `bash -n ~/Dev/vmConfig/journal-reader/start && ~/Dev/vmConfig/journal-reader/journal-search-test` -- expected: parses; journal-search tests pass.

**Manual checks (if no CLI):**
- The two acceptance runs on `tmux -L jr`, then `kill-server`. Tell Lynn: `tabrail-tmux restart` keeps the old reader, so kill the reader pane once (it restarts on the next tab switch) to pick up the new `start`.
