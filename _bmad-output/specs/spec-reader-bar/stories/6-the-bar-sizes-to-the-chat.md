---
title: 'The bar sizes to the chat, buttons instead of double-click'
type: 'feature'
created: '2026-11-23'
status: 'done'
baseline_commit: 'a8f7a907b083ff0c7f9b44ab8e40430711e9b410'
route: 'dispatch'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/specs/spec-reader-bar/SPEC.md'
  - '{project-root}/_bmad-output/specs/spec-reader-bar/stack.md'
  - '{project-root}/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** The bar grows only on a double-click, which tmux counts only within about 300 ms (a slower double-click does nothing) and which nothing on screen hints at (user feedback, 2026-11-23: "nothing doubleclick").

**Approach:** With tabrail-chat as the reader, the bar sizes to what it shows: it grows a row per input line and with the answer as it streams, up to a cap; two buttons in the bar make it bigger and smaller. The double-click binding goes everywhere.

## Boundaries & Constraints

**Always:**
- Auto height: rows of the input plus the latest question and its answer (tool lines included), rounded up to a multiple of 4, between 4 rows and a third of the window height. Taken only when the chat's content changes (a key edits the input, answer or tool lines arrive) while the bar has the keyboard; never on a click, a focus change or a tab switch alone. At most one resize every 500 ms while an answer streams.
- The tab's own pane taking the keyboard shrinks the bar to 4 rows (as today); it stays 4 rows while answers arrive, until its content changes again while it has the keyboard.
- Buttons: `▲` and `▼` at the right end of the input's last row (after the timer), shown only when tabrail-chat's pane is the reader. A click on `▲` grows the bar by a sixth of the window height, at most two thirds; on `▼` shrinks it by a sixth, at least 4 rows. A size set by a button holds (no auto height) until the next question is sent.
- tabrail-chat finds its own pane by its terminal (`pane_tty`), since the reader command unsets `TMUX_PANE`; it resizes nothing when that pane is not the reader or the tab is zoomed. The resizing itself goes through tabrail-reader (its lock, zoom and place rules).
- No double-click binding: tabrail.tmux binds no `DoubleClick1Pane` (tmux's default applies), `tabrail-reader grow` goes, and a vmConfig patch restores `tmux-mouse.conf`'s own `DoubleClick1Pane` line (Lynn applied story 5's patch).
- Kept from stories 1 and 5: heights only (no pane's width changes, `@sidebar_width` never written); a tab switch resizes no other pane (the bar leaves a tab at 4 rows and arrives at 4); zoomed tabs stay zoomed; option names, `follow`/`fit`/`away`/`slots`.

**Never:** Grow the bar on a click, a double-click or focus alone; touch the claude stream handling of story 2 beyond what sizing needs; edit `~/Dev/vmConfig` files in place; test on the default tmux socket.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Typing | bar with the keyboard, 160 wide, 46-row window, 500 characters typed, no conversation | input on 4 rows, bar 8 rows (4 input + 1 conversation, rounded up) | N/A |
| Streaming | question sent, 20-line answer streams, bar has the keyboard | bar grows in steps of 4 to 15 rows (a third of 46) and stays; the answer scrolls within | at most one resize per 500 ms |
| Away | bar 12 rows, own pane clicked, more answer text arrives | bar 4 rows, stays 4 | N/A |
| Back | bar 4 rows, clicked into, nothing typed | stays 4 rows | N/A |
| ▲ / ▼ | bar 4 rows in a 46-row window, ▲ clicked 5 times, then ▼ once | 11, 18, 25, 30, 30, then 23 rows | ▼ at 4 rows: stays 4 |
| Manual holds | after ▲, answer text arrives | size kept; the next question: auto again | N/A |
| Double-click | double-click into the bar, or a word in an own shell pane | bar unchanged; the word copied (tmux's default) | N/A |
| Tab switch | bar 12 rows in tab 1, tab 3 selected | bar 4 rows across tab 3's bottom; no pane of tab 3 resized | N/A |
| Not the reader | tabrail-chat in an ordinary pane | no buttons, no resize | N/A |
| Zoomed | content changes in a zoomed tab | nothing resized, the tab stays zoomed | N/A |

</frozen-after-approval>

## Code Map

- `tabrail-reader` (99 lines) -- `grow WINDOW` (line 13, dispatch line 96-97) becomes `size PANE ROWS|up|down`: acts only when PANE is the reader; ROWS is the auto height (clamped to 4..window/3), up/down step a sixth of the window within 4..2/3; resize only while the reader is its window's active pane and not zoomed; `place` first. Move helpers into `tabrail-reader-size` to stay under 100 lines.
- `tabrail-reader-size` (55 lines) -- `big`/`small`/`tall`/`fit`/`grow`. `@reader_big` (kept name, AGENTS.md) holds the bar's height when above 4 rows, so `fit` re-applies it after a terminal resize while the bar or sidebar has the keyboard; the own pane active: 4 rows and `@reader_big` unset. `grow` goes.
- `tabrail.tmux:45,67-72` -- the `DoubleClick1Pane` binding and its comments go.
- `src/chat/chat.rs` (99 lines) -- `frame()` gives rows; add the auto height (the latest question and answer rows plus input rows), a content-changed flag (set by input edits and claude text/tool events), manual mode (set by a button, cleared by `send`), button clicks. New module for the bar's size (e.g. `src/chat/bar.rs`): find its pane once by tty (`/proc/self/fd/0` → `tmux list-panes -a -F '#{pane_tty} #{pane_id} #{@reader}'`), rate limit, run `$(tmux show -gv @tabrail_dir)/tabrail-reader size <pane> <rows|up|down>` detached.
- `src/chat/keys.rs` (98 lines) -- clicks are dropped (`csi`); a left press (`<0;col;rowM`) becomes a key with its row and column. Split if it would pass 100 lines.
- `src/chat/input.rs` -- `rows()` draws `▲ ▼` at the last row's right edge after the timer when asked, and says where they are.
- `src/chat/view.rs` -- rows of the latest question and answer (for the auto height).
- `tabrail-reader-test`, `tabrail-slot-test` -- double-click checks (reader-test 19-32, 44, 55-57; slot-test 51-58 with the vmConfig patch) become `size`/button checks or go; the word-copy double-click check stays (tmux's default now). The slot test applies `5-vmconfig.patch` to its copies; with `6-vmconfig.patch` it must end at the restored binding.
- New test (e.g. `tabrail-bar-test`) -- tabrail-chat as `@tabrail_reader` with `TABRAIL_CHAT_CLAUDE` = `tabrail-chat-fake`, on `-L` servers, one check per matrix row; reuse `tabrail-test-lib`, `tabrail-chat-test-lib`.
- `_bmad-output/specs/spec-reader-bar/stories/6-vmconfig.patch` -- against the live `~/Dev/vmConfig/tmux-mouse.conf` (story 5's patch applied): its `DoubleClick1Pane` line back to `bind -n DoubleClick1Pane select-pane -t = \; if -F "#{||:#{pane_in_mode},#{mouse_any_flag}}" { send -M } { copy-mode -H ; send -X select-word ; send -X copy-pipe-no-clear }` and its comment; applies with `patch -p0 -d ~/Dev/vmConfig`; made and tested on copies.
- `README.md` (reader-pane bullet, tabrail-chat bullet, the DoubleClick note at 102), `SPEC.md` (CAP-2 intent and success, the double-click Assumption, Success signal), `stack.md` (sizes line).

## Tasks & Acceptance

**Execution:**
- [x] `tabrail-reader`, `tabrail-reader-size` -- `size`, `fit`, no `grow`.
- [x] `tabrail.tmux` -- no double-click binding.
- [x] `src/chat/` -- auto height, buttons, clicks, the size module.
- [x] tests -- double-click checks replaced; a bar test with one check per matrix row; every file under 100 lines.
- [x] `6-vmconfig.patch` -- double-click line restored.
- [x] `README.md`, `SPEC.md`, `stack.md`.

**Acceptance Criteria:**
- Given the bar test, then every line is `ok`, and `tabrail-reader-test`, `tabrail-slot-test`, `tabrail-chat-test`, `tabrail-chat-session-test` still pass.
- Given tabrail-chat as the reader on a test server, when tabs are switched while it is 12 rows, then no pane of either tab but the bar changes size.

## Implementation Notes

- tabrail-chat: `bar.rs` decides when and how high (the input's and the view's `changed` flags, set by edits, a sent question, answer text, tool and error lines; the 500 ms limit holds a change for a later frame, not drops it; `manual` set by a button, cleared by `sent()`); `tmux.rs` finds the pane and runs the asks in order on one thread; `mouse.rs` parses SGR reports (split off `keys.rs` for its line budget). The timer's format moved into `input.rs` and the wheel/PgUp/PgDn arithmetic into `view.rs` (`scroll`) to keep `chat.rs` under 100 lines.
- The pane is found by `/proc/self/fd/0` against `#{pane_tty}` and looked up again a few times over 3 s until `@reader` is set: `start()` sets it right after `respawn-pane`, so the chat can look before it. An ordinary pane gives up after that; a pane never becomes the reader later (the reader is always a fresh respawn).
- An auto height is first checked with one `tmux display` (bar active, tab unzoomed, the clamped height differs from the pane's): `tabrail-reader size` and its lock (which tab-switch `follow` also takes) run only for a real resize. `tabrail-reader size` checks the same again under the lock; the buttons go to it directly (the click selected the bar).
- Buttons sit where they fit on the input's last row, like the timer (the timer is dropped first): hidden while a long input's last row is not on screen or the text fills that row to the edge. Hit zones: `▲` or the space before it, `▼` or the space before it.
- `@reader_big` holds the height; `fit` caps it at 2/3 of the window and treats values of 4 or less as small, so a live bar still carrying story 5's flag value `1` shrinks to 4 rather than 1 row on the first `fit` after the upgrade.
- Tab switch acceptance: a 12-row bar leaves its tab at 4 rows (Always rule), so that tab's own panes get those 8 rows back by design. `tabrail-bar-test` checks tab 3's panes unchanged throughout, tab 1's at their sizes with a 4-row slot, and both unchanged on the way back, with answer text arriving in between.
- `tabrail-slot-test` applies `5-vmconfig.patch` only when neither part is applied (forward dry-run), then `6-vmconfig.patch` unless it reverses cleanly; checked on copies in the three states (neither, 5, 5 and 6), all ending at story 5's resurrect entry plus the restored `DoubleClick1Pane` line.
- Review fixes: `tabrail-reader grow` stays as a no-op exiting 0, for old double-click bindings (an unpatched tmux.conf, a server not reloaded). The button hold also ends when the bar is back at 4 rows: `tmux.rs` reports tmux's height after each button's ask (a refused `▼` at 4 clears it), and `bar.rs` clears it when the screen shrinks to 4 rows with no button ask pending (own pane clicked, tab switch). `tmux.rs` drains its queue before each ask (buttons in order, only the newest own height) and `find` retries a failed `tmux list-panes`. `place` joins at SMALL to 2/3 rows.
- Decision from Lynn's words (2026-11-23, "a prompt that scales with the amount of text, per line bigger, then grows with the output"): while a new question is typed (no turn running, the input not empty) the bar's own height is the input's rows plus one conversation row, rounded up to 4; the latest question and answer count again from Enter on, while that turn streams and after it until the next typing starts. Without it, typing after a long answer made the bar jump to the cap. `Bar::fit` gets whether a turn runs from `Chat::frame`.
- Not changed: AGENTS.md still describes `@reader_big` as "whether it is grown" (now it holds the height; still set only when grown).

## Spec Change Log

## Review Triage Log

### 2026-11-23 — Review pass
- verdicts: 31 findings — high 0, medium 9, low 21, false 1, maybe-false 0
- findings:
  - `[medium]` `[patch]` (verification-gap) `fit`'s 2/3 cap after a terminal shrink untested — 30-row bar, terminal 46→30 rows: 20.
  - `[low]` `[patch]` (verification-gap) story 5's live `@reader_big 1` untested — set by hand, `fit`: 4 rows, unset.
  - `[medium]` `[patch]` (verification-gap) tool lines in the auto height untested — `ask tools`: 8 rows.
  - `[medium]` `[patch]` (verification-gap) slot-test's double-click check cannot tell a word copy from a failing `grow` — reads `#{pane_mode}` (copy-mode) and asserts no `tabrail-reader` in DoubleClick1Pane.
  - `[low]` `[patch]` (verification-gap) buttons on a crowded last input row untested — timer and ▲ ▼; 152 characters: ▲ ▼ only; 156: none, a press there does nothing.
  - `[medium]` `[patch]` (verification-gap) the live tmux-mouse.conf still calls `grow` — `grow` kept as a no-op (exit 0).
  - `[medium]` `[patch]` (verification-gap) `manual` set before the size is taken — cleared whenever the bar is back at 4 rows.
  - `[medium]` `[patch]` (blind-hunter) removing `grow` breaks old bindings (live config, a running server's old binding) — same no-op.
  - `[low]` `[patch]` (blind-hunter) a reader other than tabrail-chat can no longer grow the bar — README: the bar sizes itself only with tabrail-chat; another reader command stays at 4 rows.
  - `[medium]` `[patch]` (blind-hunter) `manual` outlives its size (▲, away, back, typing never auto-grows) — cleared at 4 rows; bar-test check.
  - `[low]` `[reject]` (blind-hunter) window events leave a wrong auto height (fit caps at 2/3 not 1/3; zoom spends the change flags) — only after a terminal shrink or unzoom while grown; the next content change sizes it again; telling auto from button heights in fit needs another option.
  - `[low]` `[reject]` (blind-hunter) pane discovery is a one-shot ~3 s poll — `@reader` is set by the same tabrail-reader run that respawns the pane, milliseconds later; a failed tmux call now retries (edge-case row).
  - `[low]` `[patch]` (blind-hunter) `place` joins with the unclamped `@reader_big` — clamped to 4..2/3.
  - `[low]` `[patch]` (blind-hunter) asks queue behind the flock and run late, unmerged — drained, newest Rows kept, Up/Down in order.
  - `[low]` `[reject]` (blind-hunter) the rate-limit check accepts 400 ms gaps — asks are spaced 500 ms; the tolerance absorbs scheduling jitter.
  - `[low]` `[defer]` (blind-hunter) AGENTS.md: `@reader_big` now rows, the `tabrail-reader size` contract, pane by `pane_tty` — agent-context file; deferred-work.md.
  - `[low]` `[patch]` (blind-hunter) the chat test lib's `C` mode half done, its doc claims more — `HELPERS_ONLY=1`, doc and README row say only what it does.
  - `[low]` `[patch]` (blind-hunter) documented behaviour with no test — ▲ → away → back → type and the crowded row added; vmConfig mouse ▲/▼, hit zones and resize/unzoom corners left (rejected as covered by the size and button checks).
  - `[low]` `[reject]` (blind-hunter) wasted tmux calls at the cap or while away — one `tmux display` twice a second during an answer.
  - `[low]` `[patch]` (blind-hunter) `hs` used before set — `hs=` first.
  - `[low]` `[patch]` (edge-case-hunter) asks back to back — same queue fix.
  - `[medium]` `[patch]` (edge-case-hunter) `manual` set when the step is refused — same hold fix.
  - `[low]` `[reject]` (edge-case-hunter) fit caps an auto height at 2/3 — same as the window-events row.
  - `[low]` `[reject]` (edge-case-hunter) a shrink clamp rewrites `@reader_big`, a grown terminal does not restore it — rare; the next content change or ▲ sets it.
  - `[low]` `[patch]` (edge-case-hunter) a failed `list-panes` in `find` skips the retries — it retries.
  - `[low]` `[patch]` (edge-case-hunter) `hs` uninitialised — same fix.
  - `[low]` `[patch]` (edge-case-hunter) an inherited `C` switches the chat tests to helpers-only — dedicated flag.
  - `[medium]` `[patch]` (edge-case-hunter) removed `grow` dispatch — same no-op.
  - `[low]` `[patch]` (edge-case-hunter) `place` without the old 2/3 clamp — same clamp.
  - `[low]` `[reject]` (edge-case-hunter) the acceptance says no pane of either tab changes, but a 12-row bar leaving tab 1 gives its panes 8 rows back — the frozen Always rule has the bar leave a tab at 4 rows; the test checks tab 3 untouched and tab 1 back to its sizes.
  - `[false]` `[reject]` (edge-case-hunter) the intent says a row per input line, the code grows in 4-row steps — the frozen Always rule rounds up to a multiple of 4.
- after the review, from Lynn's words: while a new question is typed only the input counts (plus one row), so a short question after a long answer keeps the bar at 4; the answer counts again from Enter (bar-test checks).

## Verification

**Commands:**
- `cargo build --release --offline && bash -n tabrail-reader tabrail-reader-size tabrail-*-test && wc -l tabrail-* src/chat/*.rs` -- expected: builds without warnings, parses, every file under 100 lines.
- each `tabrail-*-test` -- expected: all `ok`, exit 0, its `-L` servers gone.
