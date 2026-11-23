---
title: 'Slot in every tab, grow on double-click'
type: 'feature'
created: '2026-11-23'
status: 'done'
baseline_commit: 'c03485f4c65b1851d831b281c0f0cccb9d541693'
route: 'dispatch'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/specs/spec-reader-bar/SPEC.md'
  - '{project-root}/_bmad-output/specs/spec-reader-bar/stack.md'
  - '{project-root}/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Joined into each tab (story 1), the bar takes 5 rows from the new tab's own panes on every tab switch and gives them back in the old one, so every Claude Code in them redraws: tab switches lag. Any click into the full-width bar, right under the work pane's prompt, grows it to 2/3: it pops up too often (user feedback, 2026-11-23).

**Approach:** Every tab keeps an idle 4-row slot pane across the bottom. Following a tab, the reader swaps places with the new tab's slot (same size, so no other pane resizes); a new reader takes the place of the shown tab's slot. The bar grows to 2/3 only on a double-click into it; a single click gives it the keyboard at 4 rows; a click into the tab's own pane shrinks it. SPEC.md CAP-1/CAP-2 and stack.md change to match.

## Boundaries & Constraints

**Always:**
- A tab switch, session switch or `away` changes no pane's size in either tab, except: a big bar shrinks in the tab it leaves; a zoomed tab unzooms and rezooms as today (`rezoom`).
- One slot per tab, pane option `@reader_slot`; a tab holds its slot or the reader, never both, never two slots. Slots only while `@tabrail_reader` is set, created only under tabrail-reader's lock.
- The sidebar treats a slot like the reader: never a tab's content pane, and a tab with a slot expects the sidebar flags 01100 (src/width.rs).
- A tab left with only sidebar, slot and/or reader closes; `tabrail-tmux restart` kills neither reader nor slots; Duplicate takes the own pane's folder, never a slot's.
- A layout command or terminal resize puts a tab's slot back across the bottom at 4 rows, as `fit` does for the bar.
- Option names `@reader`, `@reader_big`, `@tabrail_reader`, `@tabrail_reader_started` and subcommands `follow`, `fit`, `away` stay.
- Decided: a double-click into a big bar toggles it back to 4 rows, the keyboard staying in the bar.
- Decided: after a resurrect restore every tab has exactly one slot again: the slot runs a child named `tabrail-slot`, and `~/Dev/vmConfig/tmux.conf`'s `@resurrect-processes` gets `"~tabrail-slot->~/Dev/tabrail/tabrail-slot"`; README notes the entry.

**Never:** Grow the bar on a single click or on keyboard focus; join-pane the reader into a tab that has a slot; test on the default tmux socket; touch `journal-reader/start`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Tab switch | bar small in tab 1, select tab 3 | bar at tab 3's bottom, full width, 4 rows, same pid; every other pane of tabs 1 and 3 keeps its size; tab 1 holds a slot | tab 3 has no slot yet: make it, then swap |
| New tab | new-window | it gets a sidebar and a slot; the bar swaps in | N/A |
| Single click | click into the small bar | keyboard in the bar, still 4 rows | N/A |
| Double-click | double-click into the bar | 2/3 of the window height; every pane keeps its width | zoomed: untouched |
| Shrink | big bar, click into own pane, or double-click into the bar | 4 rows | N/A |
| Leave while big | big in tab 3, select tab 1 | shrinks in tab 3, swaps; tab 1's panes keep their sizes | N/A |
| Start | no reader (first, or it ended) | the shown tab's slot becomes the reader, no pane resizes | 10 s guard; no slot there: split as today |
| Layout / resize | prefix Space or terminal resize, tab with a slot | slot back across the bottom at 4 rows | N/A |
| Unset | `@tabrail_reader` empty | no slots made; a tab with only sidebar and slot closes | N/A |
| Restore | resurrect restore of saved slot panes | each restored slot pane is a marked slot again; no tab gets a second one | N/A |

</frozen-after-approval>

## Code Map

- `tabrail-reader` (75 lines) -- `move()`: swap with the target's slot, `swap-pane -d -s <bar> -t <slot>`, keep `rezoom`. `start()`: respawn the shown tab's slot as the reader (`respawn-pane -k`, unset `@reader_slot`, set `@reader`). New `slots [WINDOW]` (missing slots: `split-window -fv -d -l $SMALL -t <sidebar> <dir>/tabrail-slot`) and `grow WINDOW`. The lock (line 14) covers every subcommand; the slot process itself never runs under it.
- `tabrail-reader-size` -- `fit()`: the `reader)` branch keeps the size (no `big`); `place()` also places a slot in a window without the bar; `big`/`small`/`tall` serve `grow`.
- new `tabrail-slot` -- marks its pane (`$TMUX_PANE`) `@reader_slot 1`, then blocks in a child whose command line contains `tabrail-slot`: resurrect's ps strategy saves the pane pid's children (sidebars are saved as `:tmux -C attach-session …`).
- `tabrail-tmux` -- `ensure`: `reader slots` before `reader follow`; `reap`: filter `#{@sidebar}#{@reader}#{@reader_slot}`; `restart` unchanged.
- `tabrail-act:28` -- own-pane filter also skips `@reader_slot`.
- `tabrail.tmux` -- `after-select-layout` and `window-resized` guards also fire for `#{P:#{@reader_slot}}`; `bind -n DoubleClick1Pane if -F -t = "#{@reader}" <grow> <tmux's default>` (default from `tmux -f /dev/null list-keys -T root DoubleClick1Pane`).
- `src/model.rs:6-30` -- FIELDS gains `@reader_slot`; `tabs()` skips it as it skips `@reader` (`f[21]`).
- `src/width.rs:18,33-35` -- FORMAT's `#{P:#{?@reader,1,}}` counts slots too.
- `tabrail-reader-test` (99 lines) -- split by job (e.g. a slot/swap/layout test beside it, shared helpers in a sourced file); each under 100 lines.
- `README.md` -- reader-pane bullet, Files rows, resurrect note.
- `_bmad-output/specs/spec-reader-bar/SPEC.md` (CAP-1, CAP-2 success, Assumptions) and `stack.md` ("The bar").
- `~/Dev/vmConfig/tmux-mouse.conf:21` -- binds `DoubleClick1Pane` after tabrail.tmux, so it overrides tabrail's: put the reader branch first, else its current command. `~/Dev/vmConfig/tmux.conf:88` -- the `@resurrect-processes` entry. Both files are untracked in vmConfig (its checkout is dirty, on another branch) and are the user's live config: never edit them in place and never touch vmConfig's git. Write both changes as one unified diff, `_bmad-output/specs/spec-reader-bar/stories/5-vmconfig.patch`, made against copies in the scratchpad and applying with `patch -p0 -d ~/Dev/vmConfig`; tests source the patched copies. Restore test on a `-L` server with its own `@resurrect-dir` in the scratchpad, never the user's resurrect files.

## Tasks & Acceptance

**Execution:**
- [x] `tabrail-slot` -- new: mark, block -- the slot's process.
- [x] `tabrail-reader`, `tabrail-reader-size` -- swap, respawn, `slots`, `grow`, `fit` without grow on focus -- CAP-1, CAP-2.
- [x] `tabrail-tmux`, `tabrail-act` -- slots in `ensure`, filters -- housekeeping.
- [x] `tabrail.tmux` -- hooks for slot tabs, double-click binding.
- [x] `src/model.rs`, `src/width.rs` -- slot skipped and counted like the reader.
- [x] `tabrail-reader-test` + split -- every matrix row; keep every current check that still applies; each file under 100 lines.
- [x] `README.md`, `SPEC.md`, `stack.md` -- behaviour reference and spec.
- [x] `_bmad-output/specs/spec-reader-bar/stories/5-vmconfig.patch` -- double-click reader branch for `tmux-mouse.conf`, resurrect entry for `tmux.conf`; the user applies it.

**Acceptance Criteria:**
- Given tabs 1 and 3 with own panes, when tabs are switched 1, 3, 1, 3, then `#{pane_width}x#{pane_height}` of every pane but the bar and slots in both tabs never changes.
- Given a sidebar on a test server with slots, then no tab row shows the slot as its program, and `@sidebar_width` is never written.
- Given the vmConfig mouse config sourced after tabrail.tmux on a test server, when the bar is double-clicked, then it grows.
- Given the test files, then every line is `ok`.

## Implementation Notes

- `start()` with no slot in the shown tab makes one (`mkslot`) and respawns it: the matrix's "split as today" in one code path.
- `slots` also puts an existing slot that is out of place back across the bottom (`place`): after a resurrect restore, `ensure`'s new sidebar (`split-window -hbf`, full height) narrows every restored slot.
- `move`: `swap-pane -d` makes the slot the active pane of the tab the bar leaves when the bar had the keyboard; `last-pane` then gives it back to the pane it had before the bar, as the join-pane did. Without it the bar comes back into that tab with the keyboard, and `rezoom` would zoom the slot.
- `tabrail-slot` checks and marks in one tmux command (one client's command list runs without another command in between): its pane already the reader (start respawned it) leaves it alone; another slot or the reader in its tab (a restored slot starting after `slots` made one) closes its pane. It `cd /`s: it keeps no folder busy, and the tests tell a slot's folder from the tab's.
- `place` returns in a zoomed tab: `slots` calls it outside `fit`'s zoom check.
- `slots` reads every tab's sidebar, slot, reader and slot place from one `list-panes`, so `reap` (every closed pane) can run it for all tabs: a tab whose reader ended gets its slot back at once.
- `move` puts the target slot to 4 rows across the bottom and the bar to 4 rows (big or dragged) before the swap: the swap hands each cell's size and place to the other pane.
- `tabrail-slot` exits unless its pane spans the window's width at the bottom and `@tabrail_reader` is set: resurrect's `~tabrail-slot` also matches a work pane running e.g. `less tabrail-slot-test`. With `@tabrail_reader` unset, `ensure` kills every slot.
- tmux 3.8 resizes a tab only when it is shown: a terminal resize reaches a slot tab when it is selected (`window-resized` fits the slot, `follow` swaps; either order ends at 4 rows). The slot test drives `resize-window -t` on a slot tab for a tab shown elsewhere, which also proves width.rs counting the slot (flags 01100).
- Double-click: tmux 3.8 fires `DoubleClick1Pane` after the second release (probed: down, up, SecondClick, up, DoubleClick); the first press has selected the bar already, and `fit` keeps its size. tabrail.tmux's other branch is tmux 3.8's default binding verbatim.
- Tests: `tabrail-reader-test` (bar), `tabrail-slot-test` (slots, vmConfig mouse config, resurrect save and restore with `@resurrect-dir` in its own temp dir), `tabrail-test-lib` (shared). The slot test copies `tmux.conf` and `tmux-mouse.conf` from ~/Dev/vmConfig and applies `5-vmconfig.patch` unless it is applied already; the resurrect entry's `~/Dev/tabrail/` becomes this checkout. Mutation runs: width.rs or model.rs without the slot, the slot without its named child, a join-pane `move`, `fit` growing on focus, no `last-pane`, an unconditional mark in `tabrail-slot`: each fails its check.
- Found, not changed: the resurrect save files list sidebars as `:tmux -C attach-session -f ignore-size,no-output -t $N`, which `~release/tabrail` never matches, so after a restore each tab keeps a plain shell where its sidebar was and `ensure` adds a new sidebar; the reader pane (claude) is saved with no command and comes back as a plain shell too.

## Spec Change Log

## Review Triage Log

### 2026-11-23 — Review pass
- verdicts: 29 findings — high 0, medium 9, low 19, false 1, maybe-false 0
- findings:
  - `[medium]` `[patch]` (verification-gap) no check switches tabs while a tab is zoomed; swap-pane unzooms source and target — zoom check added (tabs 1 and 3 zoomed, a slot made in a zoomed tab); fails without either rezoom.
  - `[medium]` `[patch]` (verification-gap) no double-click lands outside the bar, so the binding's else branch is untested — a double-click on a word in an own pane copies it and leaves the bar small; fails with the branch broken.
  - `[low]` `[defer]` (verification-gap) AGENTS.md does not list `@reader_slot` — agent-context file; deferred-work.md.
  - `[low]` `[reject]` (verification-gap) Design Notes say only the source tab unzooms — the code rezooms both; fixing it edits this build's spec.
  - `[low]` `[patch]` (verification-gap) usage line `fit | grow` without WINDOW — now `fit WINDOW | grow WINDOW`.
  - `[medium]` `[patch]` (blind-hunter) `~tabrail-slot` is a substring regex: a work pane saved with e.g. `less tabrail-slot-test` restores as tabrail-slot, which kills it or marks it — tabrail-slot now exits unless its pane spans the full width at the bottom (resurrect applies layouts before processes, restore.sh main); the close-path test splits with `-fv`.
  - `[medium]` `[patch]` (blind-hunter) unsetting `@tabrail_reader` leaves a slot in every tab — `ensure` kills `@reader_slot` panes when it is empty, tabrail-slot exits when it is empty; check "ensure removes every slot".
  - `[medium]` `[patch]` (blind-hunter) a bar dragged to N rows hands its cell to the tab it leaves — `move` shrinks the bar whenever it is not SMALL rows and puts the target slot back at SMALL across the bottom first; 8-row check.
  - `[low]` `[reject]` (blind-hunter) double-click cannot grow a bar in copy mode — the reader apps take the mouse (Claude fullscreen per vmConfig, tabrail-chat's SGR mouse), so copy mode there needs prefix [; bindings in two mode tables are more than a direct fix.
  - `[low]` `[patch]` (blind-hunter) a reader that exits leaves its tab with neither slot nor reader — `reap` runs `reader slots` first (rewritten to one list-panes for all tabs, as reap now calls it on every closed pane).
  - `[false]` `[reject]` (blind-hunter) no key grows the bar — the intent picks the double-click; a key is a new request.
  - `[low]` `[defer]` (blind-hunter) AGENTS.md lacks `@reader_slot` and story 5's pitfalls — same entry as the verification-gap row.
  - `[medium]` `[defer]` (blind-hunter) sidebars and the reader pane do not come back through resurrect (saved as `:tmux -C attach-session …`, which `~release/tabrail` never matches) — pre-existing; deferred-work.md, unverified by a live restore.
  - `[low]` `[reject]` (blind-hunter) deployment order (merge, pull, apply the vmConfig patch, reload) not written down — goes into the PR and the hand-off, not the spec or README.
  - `[low]` `[defer]` (blind-hunter) reader and ensure locks and the restore wait are per user, so tests and the live server serialize — pre-existing; deferred-work.md.
  - `[low]` `[reject]` (blind-hunter) tabrail-slot-test needs ~/Dev/vmConfig and the resurrect plugin; the Verification glob misses tabrail-test-lib — only this machine runs the tests; tabrail-test-lib was parsed with `bash -n` by hand; the glob is spec text.
  - `[low]` `[patch]` (blind-hunter) usage line — same fix as the verification-gap row.
  - `[low]` `[patch]` (blind-hunter) tabrail.tmux replaces a `DoubleClick1Pane` bound before it — README says so.
  - `[low]` `[patch]` (blind-hunter) zoomed tabs still unzoom and zoom again on a switch — allowed by the Always rule; README now names the exception.
  - `[medium]` `[patch]` (edge-case-hunter) resurrect substring match — same fix as the blind-hunter row.
  - `[medium]` `[patch]` (edge-case-hunter) a target slot not SMALL rows or out of place passes its size or place to the bar — same fix as the blind-hunter N-row row.
  - `[low]` `[reject]` (edge-case-hunter) a slot that is its tab's active pane makes the bar active there after the swap — a slot is active only if another client or a command selects it; tracking it adds state.
  - `[low]` `[patch]` (edge-case-hunter) a reader that exits leaves its tab slotless — same fix as the blind-hunter row.
  - `[medium]` `[patch]` (edge-case-hunter) slots with `@tabrail_reader` unset or restored while unset — same fix as the blind-hunter row.
  - `[low]` `[reject]` (edge-case-hunter) copy-mode double-click — same as the blind-hunter row.
  - `[low]` `[reject]` (edge-case-hunter) triple-click into the bar grows it, then copy mode selects a line — the reader apps take the mouse, so the triple-click goes to them; rare.
  - `[low]` `[defer]` (edge-case-hunter) a double-click on a sidebar row selects the sidebar — pre-existing (tmux's default and vmConfig's binding did it); deferred-work.md.
  - `[low]` `[patch]` (edge-case-hunter) tabrail-slot-test's forward patch has no `-N -t` and an unchecked status — `patch -s -N -t`, FAIL and stop if it does not apply.
  - `[low]` `[patch]` (edge-case-hunter) story 1's "@tabrail_reader unset: no reader pane" was dropped — restored, with the reader killed and the start guard cleared.

## Design Notes

Probed on tmux 3.8 (`-L probe-swap`, 236x84): with sidebar, two own panes and a 236x4 bar in one window and a 236x4 slot in another, `swap-pane -d -s <bar> -t <slot>` moves the bar to `0,80 236x4` in the other window, and every other pane in both windows keeps its size. The swap unzooms a zoomed source window, `-Z` included, so `rezoom` stays.

## Verification

**Commands:**
- `cargo build --release --offline && bash -n tabrail-reader tabrail-reader-size tabrail-slot tabrail-tmux tabrail-act tabrail-*-test && wc -l tabrail-* src/*.rs` -- expected: builds, parses, each file under 100 lines.
- each `tabrail-*-test` -- expected: all `ok`, exit 0, its `-L` servers gone afterwards.
