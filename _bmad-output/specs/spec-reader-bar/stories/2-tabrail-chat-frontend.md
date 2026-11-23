---
title: 'tabrail-chat, a Claude Code frontend for a few rows'
type: 'feature'
created: '2026-11-23'
status: 'ready-for-dev'
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
- Layout: last row is the input (`❯ ` + text, scrolled horizontally to keep the cursor visible; right edge dim `… 12s` while a turn runs); every other row is conversation: the question (bold `❯ …`), answer text wrapped at full width, tool lines; no blank separator rows, no chrome.
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
| Crash | fake exits 1 with stderr `boom` | dim `claude ended: boom`; Enter restarts with `--resume <sid>` | N/A |

</frozen-after-approval>

## Code Map

- `Cargo.toml` -- add `[[bin]] name = "tabrail"` (path `src/main.rs`) and `[[bin]] name = "tabrail-chat"` (path `src/chat/main.rs`); no `[dependencies]`.
- `src/main.rs:37,45`, `src/input.rs` -- `stty` via `Command` with inherited stdin, mouse escapes, SGR parsing; copy patterns, share no modules with the sidebar.
- `src/draw.rs:27-52` -- `fit` (chars as columns) and `changes` (redraw changed lines): same approach in the chat's view.
- `tabrail-reader-test` -- harness style for `tabrail-chat-test` (`check`, `settle`, throwaway `-L` servers, cleanup trap).

## Tasks & Acceptance

**Execution:**
- [ ] `Cargo.toml` -- two `[[bin]]` entries.
- [ ] `src/chat/json.rs` -- parse a line into a small value enum (object, array, string, number, bool, null; `\uXXXX` with surrogate pairs) and escape a string for output.
- [ ] `src/chat/claude.rs` -- spawn with options and `TABRAIL_CHAT_CLAUDE`, write user/control lines, reader thread → events (text, tool, turn end with error, session id, exit with stderr tail).
- [ ] `src/chat/keys.rs` -- stdin bytes → key and wheel events.
- [ ] `src/chat/view.rs` -- conversation entries, wrap at width, scroll offset, rows for a height.
- [ ] `src/chat/main.rs` -- terminal setup/restore, size each tick via `ioctl(TIOCGWINSZ)` (`extern "C"`, std links libc), event loop on an mpsc channel with a 200 ms timeout, draw.
- [ ] `tabrail-chat-test` -- fake claude (bash, reads lines, prints canned stream-json per question) on a `-L` server, one check per I/O row.
- [ ] `README.md` -- Files rows; a line under the reader pane: `tabrail-chat` is a small-pane Claude Code frontend the reader command can run.

**Acceptance Criteria:**
- Given `tabrail-chat --model claude-haiku-5-5 --no-session-persistence` in a 120x4 pane in a scratch folder, when "say hi in one word" is typed and Enter pressed, then within 15 s the pane shows the question line and the answer above `❯ ` with no other rows used.
- Given `tabrail-chat-test`, then every line is `ok`.

## Implementation Notes

## Spec Change Log

## Review Triage Log

## Design Notes

A stream line is one object; only `type`, `subtype`, `event.type`, `event.delta.type/text`, `message.content[]` (`type`, `name`, `input`), `parent_tool_use_id`, `session_id`, `is_error`, `result` are read. Wrap: split the answer on `\n`, break each paragraph at the last space before the width, hard-break longer words.

## Verification

**Commands:**
- `cargo build --release --offline && ls target/release/tabrail target/release/tabrail-chat && wc -l src/chat/*.rs tabrail-chat-test` -- expected: both binaries, every file under 100 lines.
- `./tabrail-chat-test` -- expected: all `ok`, exit 0, its `-L` server gone.

**Manual checks (if no CLI):**
- The real-claude acceptance run once, in `tmux -L chat-try` with `-f /dev/null`, then `kill-server`.
