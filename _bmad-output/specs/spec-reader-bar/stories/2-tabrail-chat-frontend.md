---
title: 'tabrail-chat, a Claude Code frontend for a few rows'
type: 'feature'
created: '2026-11-23'
status: 'done'
baseline_commit: 'fcf80d85ab8216dd61811a76cb9e25b7f42e68c2'
route: 'dispatch'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/specs/spec-reader-bar/SPEC.md'
  - '{project-root}/_bmad-output/specs/spec-reader-bar/stack.md'
  - '{project-root}/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Claude Code's TUI spends a 4-row pane on its prompt box and mode line; the reader bar needs a frontend that spends every row on the conversation (SPEC CAP-3, CAP-5, CAP-6, CAP-4's options pass-through).

**Approach:** A second binary `tabrail-chat [claude options…]` in the tabrail crate runs `claude -p` over stream-json (stack.md) with the caller's folder, environment and options, and draws the tail of the conversation above a one-line input at the pane's full width.

## Boundaries & Constraints

**Always:**
- Std only, no crates; each file under 100 lines, split by job (stream, JSON, keys, view, main loop).
- Claude runs as `${TABRAIL_CHAT_CLAUDE:-claude} -p --input-format stream-json --output-format stream-json --verbose --include-partial-messages --permission-prompts none <options…>` (story 3 swaps in `--permission-prompt-tool stdio`); `initialize` control request first.
- Layout: the input sits at the bottom, `❯ ` + text wrapped at the pane's full width over as many rows as it needs, the cursor where it is in the text; empty, it shows a dim `type to ask`; the right edge of its last row dim `… 12s` while a turn runs. Every other row is conversation: the question (bold `❯ …`), answer text wrapped at full width, tool lines; no blank separator rows, no chrome. The input keeps at least one conversation row.
- Decided (user, 2026-11-23): no double-click anywhere; the bar's height follows the input and the answer up to a cap, with buttons to make it bigger and smaller: story 6, after this one. Here the pane's size is whatever tmux gives it.
- Answer text from `text_delta` stream events (thinking skipped); tool lines from the `assistant` message's `tool_use` blocks, dim, `<icon> <tool> <arg>` cut to width; icons as the sidebar's states: Bash ⚙, Read ◎, Grep/Glob ⌕, Edit/Write ✎, Task/Agent ⧉, else `·`; arg: `command`, `file_path` (`~` for $HOME), `pattern`, `skill`, else the first string field. Lines with a non-null `parent_tool_use_id` (subagents) are skipped.
- Scroll: wheel (SGR 64/65) 3 lines, PgUp/PgDn a page; a key typed or new answer text returns to the bottom.
- Keys: UTF-8 text, Backspace, Left/Right, Home/End, Ctrl-A/E/U; Enter sends a non-empty line; Ctrl-C clears it; Ctrl-D on an empty line quits; bracketed paste joins lines with spaces; clicks ignored.
- A `result` with `is_error` or claude exiting shows one dim line with the error (last stderr line on exit); the next Enter starts claude again with `--resume <last session_id>` in place of `--continue`/`-c`/`--resume`/`-r`.
- Redraw only changed lines; on exit restore the terminal (`stty sane`, main screen, mouse off).

**Never:** Render markdown; show tool output; send slash commands (story 3); touch `tabrail-reader*`, `src/` sidebar modules or `~/Dev/vmConfig`; run a test tmux on the default socket.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Stream | fake sends 5 text deltas of a 6-line answer | rows 1-3 show its last 3 wrapped lines while it streams | N/A |
| Tool | `tool_use` Bash `./journal-search -R tmux` | `⚙ Bash ./journal-search -R tmux`, dim, one row | cut at width |
| Escapes | text with `\n`, `\"`, `é`, `😀` | newline, quote, é, 😀 | bad JSON line skipped |
| Scroll | 3 turns, wheel up ×5 | first question visible; typing `x` → bottom | top clamps |
| Resize | pane 120x4 → 160x30 | rewrapped at 160, 29 conversation rows | N/A |
| Long input | 200 characters typed in a 120x4 pane | input on its last 2 rows, the cursor after the last character; 2 conversation rows | text longer than 3 rows: its tail, cursor visible |
| Empty input | nothing typed | dim `type to ask` after `❯ ` | N/A |
| Crash | fake exits 1 with stderr `boom` | dim `claude ended: boom`; Enter restarts with `--resume <sid>` | N/A |

</frozen-after-approval>

## Code Map

- `Cargo.toml` -- add `[[bin]] name = "tabrail"` (path `src/main.rs`) and `[[bin]] name = "tabrail-chat"` (path `src/chat/main.rs`); no `[dependencies]`.
- `src/main.rs:37,45`, `src/input.rs` -- `stty` via `Command` with inherited stdin, mouse escapes, SGR parsing; copy patterns, share no modules with the sidebar.
- `src/draw.rs:27-52` -- `fit` (chars as columns) and `changes` (redraw changed lines): same approach in the chat's view.
- `tabrail-reader-test` -- harness style for `tabrail-chat-test` (`check`, `settle`, throwaway `-L` servers, cleanup trap).

## Tasks & Acceptance

**Execution:**
- [x] `Cargo.toml` -- two `[[bin]]` entries.
- [x] `src/chat/json.rs` -- parse a line into a small value enum (object, array, string, number, bool, null; `\uXXXX` with surrogate pairs) and escape a string for output.
- [x] `src/chat/claude.rs` -- spawn with options and `TABRAIL_CHAT_CLAUDE`, write user/control lines, reader thread → events (text, tool, turn end with error, session id, exit with stderr tail).
- [x] `src/chat/keys.rs` -- stdin bytes → key and wheel events.
- [x] `src/chat/view.rs` -- conversation entries, wrap at width, scroll offset, rows for a height.
- [x] `src/chat/main.rs` -- terminal setup/restore, size each tick via `ioctl(TIOCGWINSZ)` (`extern "C"`, std links libc), event loop on an mpsc channel with a 200 ms timeout, draw.
- [x] `tabrail-chat-test` -- fake claude (bash, reads lines, prints canned stream-json per question) on a `-L` server, one check per I/O row.
- [x] `README.md` -- Files rows; a line under the reader pane: `tabrail-chat` is a small-pane Claude Code frontend the reader command can run.

**Acceptance Criteria:**
- Given `tabrail-chat --model claude-haiku-5-5 --no-session-persistence` in a 120x4 pane in a scratch folder, when "say hi in one word" is typed and Enter pressed, then within 15 s the pane shows the question line and the answer above `❯ ` with no other rows used.
- Given `tabrail-chat-test`, then every line is `ok`.

## Implementation Notes

- Split by job to stay under 100 lines: besides the listed files, `stream.rs` (a stream-json line into events), `session.rs` (claude across processes: `--resume` restarts, events of a replaced process dropped), `chat.rs` (keys and events into state, the frame), `input.rs` (the input rows and cursor), `tool.rs` (a tool call's line), `term.rs` (stty, escapes, `ioctl`); the fake claude is its own file, `tabrail-chat-fake`. The test is split by job: `tabrail-chat-test` (screen and keys) and `tabrail-chat-session-test` (claude's options, restarts, the way out), sharing `tabrail-chat-test-lib`, which sources `tabrail-test-lib` (unchanged) and writes its own conf.
- Blank lines in an answer are not drawn (no row spent on nothing); tabs show as four spaces, other control characters are dropped.
- Autowrap is off while running, so a wide character (😀) cannot push a row onto the next; rows are cleared (`ESC[2K`) before they are written, since an erase after a full row would take its last character with autowrap off.
- Enter while a turn runs sends nothing; the text stays in the input. Restarts: `--resume <id>` with the caller's `-c`/`-r`/`--resume`/`--session-id` dropped; the caller's options as given under `--no-session-persistence`; a new conversation (those options dropped, no `--resume`) after a process ended before it named its session.
- Claude ends with SIGTERM, SIGKILL after 1 s, a replaced one on its own thread; a `result` with `is_error` shows `claude failed: <result or subtype>`, an exit `claude ended: <last stderr line>`, once.
- Real run (claude 2.1.323, Haiku 5.5, `tmux -L chat-try -f /dev/null`, 120x4): answer `Hi` about 1 s after Enter, rows: question, answer, input; the stream matched stack.md (`control_response` for `initialize`, `system` hook lines, `stream_event`, `assistant`, `result`, `rate_limit_event`).

## Spec Change Log

## Review Triage Log

### 2026-11-23 — Review pass
- verdicts: 44 findings — high 0, medium 10, low 32, false 1, maybe-false 1
- findings:
  - `[medium]` `[patch]` (blind-hunter) a question sent during a turn takes the rest of the running answer and its result clears the timer — Enter sends nothing while a turn runs, the text stays; check added.
  - `[medium]` `[patch]` (blind-hunter) a resume that cannot work (`--no-session-persistence`, a caller's `--session-id`) fails on every Enter — no resume under `--no-session-persistence`, `--session-id` dropped, an id forgotten when a resumed claude ends before naming a session (then the caller's -c/-r/--resume/--session-id are dropped too); checks added.
  - `[low]` `[reject]` (blind-hunter) a write to a dead claude's stdin is dropped, the question not resent — only within the second before the exit event lands; "claude ended" then shows and the user asks again; resending needs a retry path.
  - `[maybe-false]` `[defer]` (blind-hunter) claude shares tabrail-chat's process group and terminal: a tool opening /dev/tty could read the bar's keys; SIGKILL leaves tool processes — unverified whether Claude Code detaches its tools; deferred-work.md.
  - `[low]` `[reject]` (blind-hunter) a lone Esc waits for the next byte and the reader has no timeout — Esc is story 3's (interrupt), which needs the same timeout; story 2 binds nothing to it.
  - `[low]` `[patch]` (blind-hunter) mouse reporting takes tmux's selection from the bar — README: Shift-drag selects.
  - `[low]` `[reject]` (blind-hunter) the wrapped input lacks Delete, Up/Down, word moves, Ctrl-K — the story lists the keys; more keys are a new request.
  - `[low]` `[reject]` (blind-hunter) the whole conversation is rewrapped every 200 ms tick — a day of reader chat is a few thousand lines, well under a millisecond's work per tick.
  - `[low]` `[reject]` (blind-hunter) the timer is not drawn when the input's last row is full — only while a long follow-up fills that row exactly; reserving room adds layout rules.
  - `[low]` `[reject]` (blind-hunter) characters counted as columns (wide and combining characters) — the SPEC assumes one character is one column.
  - `[low]` `[reject]` (blind-hunter) SPEC.md CAP-2, stack.md, story 6's file not updated — story 6 amends CAP-2 and writes its story file; fixing it here edits the spec.
  - `[low]` `[reject]` (blind-hunter) the frozen block changed without a Spec Change Log entry, Intent still says one-line input — the user renegotiated it on 2026-11-23 (the Decided bullet); the fix edits this build's spec.
  - `[low]` `[patch]` (blind-hunter) tests missing for resume options, did-not-start, bold/dim, a question during a turn, redraw of changed rows — covered with the verification-gap checks.
  - `[low]` `[patch]` (blind-hunter) docs: claude CLI needed, how a rebuilt tabrail-chat goes live, AGENTS.md pitfall — README done; AGENTS.md deferred (agent-context file).
  - `[medium]` `[patch]` (edge-case-hunter) resume under `--no-session-persistence` — same fix as the blind-hunter resume row.
  - `[medium]` `[patch]` (edge-case-hunter) `--session-id` kept beside `--resume` — same fix.
  - `[medium]` `[patch]` (edge-case-hunter) timer cleared while a queued question runs — same fix as the blind-hunter turn row.
  - `[medium]` `[patch]` (edge-case-hunter) answer drawn under the next question — same fix.
  - `[low]` `[reject]` (edge-case-hunter) write to a dead pipe ignored — same as the blind-hunter row.
  - `[low]` `[reject]` (edge-case-hunter) scrolled back, appended tool lines shift the view by a row each — the spec returns to the bottom on new answer text anyway; tracking rows per entry adds state.
  - `[low]` `[reject]` (edge-case-hunter) wheel past the top and back down in one event batch does not move — needs both within one 200 ms batch; clamping in key() needs the last frame's top.
  - `[low]` `[reject]` (edge-case-hunter) wide characters — SPEC assumption.
  - `[low]` `[reject]` (edge-case-hunter) lone Esc then `[`/`O` swallows keys — story 3's Esc handling.
  - `[low]` `[patch]` (edge-case-hunter) an invalid UTF-8 lead byte swallows the keys after it — one byte consumed; check added.
  - `[false]` `[reject]` (edge-case-hunter) Enter on spaces clears them unsent — deliberate (Implementation Notes): nothing to send, the input cleared.
  - `[low]` `[reject]` (edge-case-hunter) a panic or SIGTERM to tabrail-chat skips claude's Drop — a closing pane sends SIGHUP to claude too (same terminal); claude's stdin closes and it ends after its turn.
  - `[low]` `[patch]` (edge-case-hunter) dropping the old claude blocks input for up to 1 s — dropped on its own thread.
  - `[low]` `[reject]` (edge-case-hunter) rewrap every tick — same as the blind-hunter row.
  - `[low]` `[reject]` (edge-case-hunter) HOME with a trailing slash never gets `~` — not how HOME is set here; cosmetic.
  - `[low]` `[reject]` (edge-case-hunter) Intent says one-line input — same as the blind-hunter frozen-block row.
  - `[medium]` `[patch]` (verification-gap) a replaced process's events pass unfiltered undetected — `row 1 == "❯ again"` asserted; fails with the filter off.
  - `[low]` `[patch]` (verification-gap) blank lines in an answer never produced — `a\n\nb` on adjacent rows.
  - `[medium]` `[patch]` (verification-gap) tabs and control characters in an answer untested (escape injection) — tab as four spaces, `\u001b[2J` not passed on.
  - `[medium]` `[patch]` (verification-gap) a question with `"` and `\` never sent — the logged user line checked as exact JSON.
  - `[low]` `[patch]` (verification-gap) `-r`, `--resume <v>`, `--resume=` replacement never exercised — restart from `-r old --resume=x --session-id s1 --session-id=s2 -c`.
  - `[low]` `[patch]` (verification-gap) typed non-ASCII untested — `é😀` typed.
  - `[low]` `[patch]` (verification-gap) hard break of a long word untested — a 130-character word at 120.
  - `[low]` `[patch]` (verification-gap) one dim line when claude exits after a failed result untested — failexit case.
  - `[low]` `[patch]` (verification-gap) the subtype fallback untested — error result without `result`.
  - `[low]` `[patch]` (verification-gap) redraw of changed rows only unverified — idle with no turn writes nothing (pipe-pane).
  - `[low]` `[patch]` (verification-gap) autowrap restore on exit unverified — `#{wrap_flag}` 1 after exit.
  - `[low]` `[patch]` (verification-gap) "claude did not start" untested — `TABRAIL_CHAT_CLAUDE` pointing nowhere.
  - `[low]` `[patch]` (verification-gap) tool icons and argument fallbacks beyond Bash and Read untested — Grep, Edit, Agent, Skill, unknown.
  - `[medium]` `[patch]` (verification-gap) the acceptance command's `--no-session-persistence` cannot resume — same fix as the blind-hunter resume row.

## Design Notes

A stream line is one object; only `type`, `subtype`, `event.type`, `event.delta.type/text`, `message.content[]` (`type`, `name`, `input`), `parent_tool_use_id`, `session_id`, `is_error`, `result` are read. Wrap: split the answer on `\n`, break each paragraph at the last space before the width, hard-break longer words.

## Verification

**Commands:**
- `cargo build --release --offline && ls target/release/tabrail target/release/tabrail-chat && wc -l src/chat/*.rs tabrail-chat-test` -- expected: both binaries, every file under 100 lines.
- `./tabrail-chat-test` -- expected: all `ok`, exit 0, its `-L` server gone.

**Manual checks (if no CLI):**
- The real-claude acceptance run once, in `tmux -L chat-try` with `-f /dev/null`, then `kill-server`.
