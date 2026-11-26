# tabrail

A vertical tab rail for tmux: a sidebar on the left of every tab that lists all tabs of the session,
grouped by project, with what each one is doing.

```
▾ Website             ▷ +          the group has no timer: a click on ▷ starts one; + adds a subgroup
 ❯ Fix login redire ✳ ⚙ 2m         Claude's topic, cut; Claude running a command for 2 min
   website·release-2 ✳   ◇         folder and git branch; Claude idle
   deploy · vim                    named by hand; vim runs in it
▾ Infra               ▷ +
   lde@app01 in · ssh ⏱ 4m         ssh: remote user, host, folder, cut; 4 min
   infra               ✗ 2         the last command failed
    14:41  Wed 18.11.
```

- Groups by folder (groups.conf), each with a faint tint; the groups file's icon column is required
  but not shown. Fold them with a click, reorder or nest them by dragging, nest them or take them
  back to the top level with their menu's Move into group…. A group's menu (not Default's) has
  Colour…: a picker of the 25 named colours in `palette`, each a swatch with the group's name on it,
  the group's own marked `· current`; typing part of a colour's name narrows the list, Enter writes
  the pick into column 2 of the group's line in the groups file (nothing else in it changes) and
  every sidebar shows it, Esc changes nothing. New group… takes the first palette colour no group
  uses. The order is the groups file's line order; Default is always last. A new tab joins the group
  whose folder is the longest one holding the tab's folder, at any level and whatever the line order
  (equal ones go to the first line; `~` expanded, symlinks not resolved), else Default; only a tab
  without a group gets one.
- Groups in groups: a line indented under a group's line makes it a subgroup, any depth deep (its
  parent is the nearest group line above it with less leading whitespace, a space or a tab one each;
  an indented line with none above is top-level). Every sidebar shows the tree: each level's header
  and tabs one column further right, a parent's own tabs above its subgroups, one blank line after
  each top-level tree and none inside one. A group shows while it or a subgroup holds a tab or has a
  timer (a parent without tabs of its own shows its header, then its subgroups). Folding a parent
  hides its whole tree, its `N tabs` counting every tab in it; a subgroup keeps its own fold. Move to
  group… lists the tree in order, indented one column per level; Rename group changes only the name,
  the line's indent and spacing stay. New tab in a group without folders opens in the first folder
  of its nearest ancestor that has folders, else home.
- New subgroups: a group's menu has New subgroup… (not Default's), and every header but Default's a
  dim `+` one column in from its right edge whose left click (press and release both on it) does the
  same. Both open a name field titled `New subgroup of GROUP` (one word: spaces become `-`; never
  Default). Enter adds the name's line right after the last line of the group's subtree, one level
  deeper, 2 spaces per level (in a file indented by hand, no deeper than the group's last subgroup,
  so the line still lands under the group), with a palette tint and an icon no group uses yet
  (not shown), written through a symlink, mode kept, renamed into place; it opens a new tab in the
  subgroup, and every sidebar of every session shows it on its next redraw. A name that exists
  anywhere adds no line and opens the new tab in that group; Esc, or an empty or blank name, changes
  nothing.
- Moving a group in or out: a group's menu has Move into group… (not Default's), a picker that lists
  Top level, then every group but this one and its subgroups in tree order, indented one column per
  level, its parent (or Top level) marked `· current`; no taller than the terminal, the rest scrolls.
  A pick moves the group with its whole tree in the groups file (as a drag nests, through
  `tabrail-groups nest`): a group makes it that group's last subgroup (its own parent too), Top level
  puts it at depth 0 after the last top-level tree, above Default (a top-level group already last
  stays). Esc changes nothing; a group renamed while the picker is open is not moved.
- A timer per group, Default too, started and stopped by hand from the group's right-click menu or a
  click on its time or `▷`: the time right after the group's name (before a folded group's `N tabs`),
  `▶ 12m` while it runs, `⏸ 12m` dimmed while stopped, in every sidebar of every session. Start timer
  (or a click on a stopped time) counts on from the stopped value; never started or reset, the row
  shows no time but a dim `▷` two columns left of the `+`, three in from its right edge (Default's
  too, its `+` column blank; Nerd Font nf-cod-play, U+EB2C: the terminal needs a Nerd Font such as
  MesloLGS NF), which starts it from zero (no `▷` and no `+` for a name with quotes or other
  characters the menus drop: it has no timer items either). Reset timer
  drops it, Rename group takes it along. Wall-clock (seconds before plus the time since its start),
  kept in `$XDG_STATE_HOME/tabrail/timers`, else `~/.local/state/tabrail/timers`: it survives
  `tabrail-tmux restart`, a tmux restart and a reboot, and a running one counts the time tmux was
  down. Where the row is too narrow the count goes first (whole or not at all), then the name is cut;
  the time, the `▷` and the `+` are never cut. A group without tabs shows only while it has a timer (its
  header, no tab rows), so its time and menu stay in reach. A group removed from groups.conf keeps
  its timer unseen, shown again if the name comes back; `tabrail-timer reset NAME` drops it.
- Tab names: a name given by hand, else Claude Code's topic, the running command (without `VAR=value`
  in front), or folder and branch. An ssh tab shows the remote's `user@host folder`, vim's file
  (`vim nginx.conf`), else what the remote titles, or the host until the remote sends a title.
- After the name, dimmed, the tab's foreground program as tmux reports it (`deploy · vim`,
  `x.py · python3`, `deploy · sudo` while `sudo vim` runs), `✳` for Claude Code; nothing while a shell
  is the foreground program (an idle prompt or a shell script) or when the name's first word is the
  program (`htop`, `make test`). A long name is cut first; the program is left out where it would leave
  fewer than 8 of the name's characters (or cut a shorter name), so it can come and go as a timer grows.
  The status is never cut.
- Status, at the right edge so the times of all rows end in one column: Claude's state with its time
  while `claude` is the tab's foreground program (`… ✎ ⚙ ◎ ⌕ ⧉` working, `● waits`, `✓ 3m` done, `◇`
  idle), `⏱` for a command running 3 s or more, `✗` for a failed one, `✓` when something finished out
  of sight.
- Mouse: click a tab to go there, drag it onto another tab or a group; click a group's row to fold it,
  its time (`▶ 12m` or `⏸ 12m`, the marker included; press and release both on it) to stop or resume
  its timer, its `+` (press and release both on it) for a new subgroup; drag the row (from its time or
  `+` too) to move the group with its whole tree in the groups file: let go on another group's header
  at or right of its name (folded or not), it becomes that group's last subgroup (its lines re-indented
  2 spaces per level; not into Default, nor into itself or one of its subgroups); let go left of a
  header's name or on a tab's row, it stays at its own level, next to that row's group or the ancestor
  of it that is the dragged group's sibling (top-level groups and Default are siblings): above it
  dragged up, below its tree dragged down; nothing where there is no sibling, so a drag never takes a
  group out of its parent (its menu's Move into group… does). Default never moves. Right-click for
  menus (new tab, new group, rename, duplicate, move to group, close, fold; a group's New subgroup…,
  Move into group…, Colour…, Start timer or Stop timer, and Reset timer once it has a time).
- One width: every tab's sidebar keeps the same width through terminal resizes and layout commands;
  drag any sidebar's border (or `resize-pane` it) to change it for all tabs until tmux stops.
- Reader pane (optional, `@tabrail_reader`): one chat in a bar across the full width of the tab
  shown, under the sidebar and the tab's own panes, 4 rows high; it moves along from tab to tab, the
  same process and conversation. Every other tab keeps an idle slot of the same size in its place
  (`tabrail-slot`), which the bar swaps places with: a tab switch resizes no other pane, so nothing
  in the tabs redraws, except in a zoomed tab, which unzooms for the swap and zooms again. A click
  into the bar gives it the keyboard at 4 rows. Only with `tabrail-chat` as the reader command does
  the bar size itself to what it shows while it has the keyboard, with buttons to make it bigger and
  smaller (below), only in height (every pane keeps its width); another reader command (Claude Code's
  own TUI as a fallback) stays at 4 rows. Dragging its border changes nothing (a hook puts the
  height back). A click into the tab's own pane shrinks it to 4 rows, where it stays while answers
  arrive, and it leaves a tab at 4 rows and arrives at 4. It follows
  into another session too (switch-client, the tree); it and the slots get their size back after a
  terminal resize and go back across the bottom after a layout command (prefix Space) or a sidebar
  added beside them. A tab left with only the sidebar and the reader pane or its slot closes, the
  pane moving on first, as does a tab closed with the menus' Close or prefix `&`; `tabrail-tmux
  restart` leaves the reader and the slots running.
- `tabrail-chat [claude options…]` is a small-pane Claude Code frontend the reader command can run. It
  runs `claude -p` over stream-json (Claude Code's `claude` on PATH, or `$TABRAIL_CHAT_CLAUDE`) in its
  caller's folder and environment with those options, and spends every row on the conversation: the
  question bold, the answer as plain text wrapped at the full width while it streams in (no tool
  calls: the text before a tool call is narration and goes when the call comes), above the input (`❯ `,
  wrapped over the rows it needs, a dim `… 12s` while a turn runs; Enter sends nothing until the turn
  is over). Wheel and PgUp/PgDn scroll back; a key or new answer text goes back to the bottom. Ctrl-C
  clears the input, Ctrl-D on an empty one quits. When claude ends or a turn fails, a dim line says
  why and the next Enter starts claude again with `--resume` on the same conversation (as it was
  started under `--no-session-persistence`; on a new one when claude could not resume it). A drag
  in the bar copies its text as in any pane (copy mode). Permission prompts are denied
  (`--permission-prompts none`). As the reader (it finds its pane by its terminal) it sizes the bar
  while the bar has the keyboard: when the input or the conversation changes, to the rows of the
  input and the latest question and answer, rounded up to a multiple of 4, from 4
  rows to a third of the window, at most once every 500 ms. While a new question is typed (no turn
  running) only the input counts, with one row above it, so a short question after a long answer
  stays at 4 rows; from Enter on the answer counts again. `▲` at the end of the input's last row
  (after the timer) grows the bar by a sixth of the window, up to two thirds, `▼` shrinks it by a
  sixth, down to 4 rows; that size holds until the next question or until the bar is back at 4
  rows. A double-click does nothing to it.
- Fast: only the sidebar shown works. Each keeps one tmux control-mode connection; the one of the tab
  shown (current in its session, a terminal attached) redraws when tmux reports a change and every
  second, listing only the tabs' own panes. The others neither draw nor tick until their tab is
  shown (a tab switch, an attach, `switch-client`), then draw at once; meanwhile a terminal attaching,
  detaching or switching sessions costs each one `list-clients`, and a layout change of its own tab
  one `display` (and a `resize-pane` if its width is off). Nothing is spawned on a tab switch but,
  with the reader pane, the helper that moves it, in two tmux calls (one to look, one to move).
  `tabrail-perf-test`, reader bar on: a tab switch shows its sidebar current after ~8 ms at 5 tabs and
  ~18 ms at 50 (~16 ms with Claude hook calls 10 times a second), the bar is in the new tab after ~70
  and ~95 ms (most of it `pgrep` looking for a resurrect restore, before the first call), and tmux
  idles at 0.1% and 0.4% of a core.
  A `swap-window` onto the tab shown tells the control connections nothing: that tab's sidebar shows
  its last frame until the next tab switch or a click on it (the reader bar does not follow either). Rust, standard
  library only.

## Setup

Needs tmux (tested with 3.8), `fzf` for the pickers and name fields, and Rust to build.

```bash
cargo build --release --offline           # target/release/tabrail and tabrail-chat
cp groups.conf.example groups.conf        # or set @tabrail_groups to a file of your own
```

In `~/.tmux.conf`:

```tmux
set -g mouse on
set -g status off                          # optional: the sidebar shows tabs and the time
set -g @tabrail_groups "$HOME/.config/tabrail/groups.conf"   # optional
set -goq @sidebar_width 26                 # optional: the starting width (-o: a reload keeps a dragged one)
source-file ~/src/tabrail/tabrail.tmux
```

After a rebuild, `tabrail-tmux restart` restarts every sidebar. A rebuilt `tabrail-chat` goes live when
the reader pane restarts: kill it (`tmux kill-pane -t <the reader>`) and tabrail starts it again.

Optional inputs, shown when present:

- Claude Code's state: a hook that sets the window options `@claude` (idle, thinking, writing, running,
  reading, searching, delegating, waiting, done), `@claude_since` (epoch of the prompt) and
  `@claude_took` (e.g. `3m12s`), then runs `tmux set-buffer -b sidebar-refresh x`. Shown only while
  `claude` is the foreground program of the tab's active pane: after a crash the options stay set, and
  the sidebar ignores them until `claude` runs there again.
- Command timers: the shell's OSC 133 marks (`\e]133;C\a` before a command, `\e]133;D;$?\a` after).
- The reader pane: `set -g @tabrail_reader '~/Dev/vmConfig/journal-reader/start'`, the command it
  runs (here the agent journal's reader chat). Unset, tabrail starts no pane or slot and spawns
  nothing for it, and `tabrail-tmux ensure` removes the slots left. If its program ends, the next
  tab switch or closed pane starts it again in the shown tab's slot, at most every 10 s.
  tabrail.tmux binds no `DoubleClick1Pane`; drop a `tabrail-reader grow` branch from a binding of
  your own (that command is gone). With tmux-resurrect, add
  `"~tabrail-slot->/path/to/tabrail/tabrail-slot"` to `@resurrect-processes` so that restored slot
  panes are slots again: the slot waits as a plain `sleep` whose child carries the name
  `tabrail-slot` for that, and a restored one respawns its pane as itself, so no shell stays behind
  it. The reader
  pane is saved as its `claude -p --input-format stream-json …`: make it a slot too
  (`"~--input-format stream-json->/path/to/tabrail/tabrail-slot"`) and keep an entry for Claude
  Code's own TUI off it (`"~^claude( --(continue|resume)|$)->claude --continue"`, not `~claude`), as
  resurrect types the command of every entry that matches. While resurrect restores this tmux server,
  tabrail-reader does nothing; `tabrail-tmux ensure` starts the reader once it is done. Load tmux-continuum with
  `run-shell -b`: without it, its restore at tmux's start holds tmux's command queue until it ends
  (no hook runs: the bar follows no tab switch), and for good once resurrect's spinner hangs.

## Files

| File | Job |
|---|---|
| `src/*.rs` | The sidebar: `main.rs` loop, `control.rs` tmux connection, `model.rs` tabs and status, `groups.rs` the group tree from the groups file (`groups/tests.rs` its tests), `name.rs` tab names, `ssh.rs` ssh tab names, `program.rs` the program after a name, `draw.rs` screen, `head.rs` a group's header row (`head/tests.rs` its tests), `row.rs` a tab's row (`row/tests.rs` its tests), `input.rs` mouse, `drag.rs` left button, `drag/group.rs` a dragged group's drop (`drag/tests.rs`, `drag/tests/plus.rs`, `drag/tests/tree.rs` their tests), `menu.rs` right-click menus, `menu/group.rs` a group's menu, `timer.rs` group timers (`timer/tests.rs` its tests), `width.rs` one width for all, `shown.rs` whether its tab is shown (`shown/tests.rs` its tests) |
| `src/chat/` | `tabrail-chat`: `main.rs` loop and drawing, `chat.rs` keys and claude's events, `session.rs` claude across restarts (`--resume`), `claude.rs` the claude process, `stream.rs` its stream-json lines, `json.rs` JSON, `view.rs` the conversation, `input.rs` the input, `keys.rs` keys, `mouse.rs` wheel and clicks, `term.rs` the terminal, `bar.rs` the reader bar's height and buttons, `tmux.rs` its pane and `tabrail-reader size` |
| `tabrail.tmux` | What tmux.conf loads: mouse bindings, hooks, prefix M and & |
| `tabrail-tmux` | Housekeeping from the hooks: a sidebar (and a reader slot) for every tab, a new tab's group (the longest folder holding it), restart, close tabs with only a sidebar (and the reader pane or its slot), the menus' Close |
| `tabrail-reader`, `tabrail-reader-size`, `tabrail-reader-move` | The reader pane, a bar across the bottom: a slot in every tab, swap places with the shown tab's slot, start in it, move away before its tab closes; its height as tabrail-chat asks (`size`); back across the bottom when tmux moved it or a slot; `-move`: what a tab switch decides on in one tmux call, the move in one more |
| `tabrail-slot` | A tab's slot: marks its pane, then waits as a plain `sleep`, no shell, with a child named `tabrail-slot` (for resurrect); restored into a shell, it respawns its pane as itself |
| `tabrail-reader-test`, `tabrail-slot-test` | The reader bar's tests and its slots' (with ~/Dev/vmConfig's mouse config and a resurrect save and restore), on throwaway tmux servers |
| `tabrail-move-test` | The reader bar's move in tmux calls, counted by a logging `tmux` first in PATH: a tab switch at most 4, `fit` with the bar or slot in place 1; a slot out of place and a zoomed tab without a slot switched to, on throwaway tmux servers |
| `tabrail-chat-test`, `tabrail-chat-session-test` | `tabrail-chat`'s tests on a throwaway tmux server: what it shows and its keys; its claude's options, restarts and the way out |
| `tabrail-bar-test` | The reader bar sized by `tabrail-chat` in it (its height, buttons, away, tab switch, zoom), against the fake claude on throwaway tmux servers |
| `tabrail-timer-test` | The group timers in the sidebar on throwaway tmux servers, state in a temp `XDG_STATE_HOME`: the menu's items (Default's, none for a name with quotes), start, stop, resume and reset by right-click, the time's colour, two groups, a second session, rename, folded, kill-server, a narrow sidebar, a garbled file, a group's last tab closed |
| `tabrail-colour-test` | A group's colour on throwaway tmux servers, the groups file a symlink to a temp copy: Colour… in the menus (none for Default or a name `clean()` changes), New group…'s palette tint (the first unused, in any case), the picker's swatches, current mark and search on colour names only, the pick in column 2 alone (link and mode kept), every sidebar, Esc, a hand-written tint, the group renamed meanwhile, the same tint again, a 50x20 client |
| `tabrail-timer-click-test` | A click on a group's time on throwaway tmux servers, real SGR presses: stop and resume (both sessions, fold state kept, folded too); beside it, the name, off the time, no time, a drag from it and a tab row as before |
| `tabrail-tree-test` | Nested groups on throwaway tmux servers, the groups file a symlink to a temp file: a flat file drawn as before (a drag still moves its line), the tree one column in per level, a click on a nested tab row, a folded parent counting its tree, a subgroup's fold kept, an empty leaf, an indented first line, `tabrail-groups tree`, Move to group…'s tree and a tab moved into a subgroup in every sidebar, a parent without tabs, rename keeping the indent (link and mode kept) |
| `tabrail-subgroup-test` | Creating a subgroup on throwaway tmux servers with real SGR clicks, the groups file a symlink to a temp file: the `+` on every header but Default's, New subgroup… and its field, the line after the parent's subtree with an unused tint and icon (link and mode kept), a new tab in it, a second session's sidebar showing it, the `+` opening the field in the terminal clicked, one level deeper, Esc and a blank name, a name that exists, no item on Default, a tab or empty space, a click on the name, off the `+` and a drag from it, a file indented by hand |
| `tabrail-nest-test` | Move into group… on throwaway tmux servers with real SGR clicks and keys, the groups file a symlink to a temp file: the item after Rename group and none on Default, the picker's rows (Top level, the tree without the group's own, indented) and its current mark at depth 0, 1 and 2, a nest (the file, link and mode kept, and main's sidebar), a move to the top level (the file and every sidebar of both sessions), the current parent, a top-level group to the top level, `nest` without PARENT, Esc, the group renamed while the picker is open, a last top-level tree of two lines, a nested last tree to the top level, a hand-written Default line left out, a group without a line, a 50x10 client |
| `tabrail-tree-drag-test` | Dragging a group with its tree on throwaway tmux servers with real SGR drags, the groups file a symlink to a temp file: up onto a tab row, down onto a header left of its name and inside a group, via a subgroup's header, nested at or right of a header's name (link and mode kept, every sidebar of both sessions), no sibling there, into its own tree, into its own parent, onto Default's tab row and header, Default dragged, into a folded group, a click still folding; the writer alone refusing what a drag never sends, a comment and a blank line moving with a tree, a file indented by hand |
| `tabrail-folder-test` | The folder a new tab joins a group by and opens in, on a throwaway tmux server, the groups file a temp file: the longest folder winning at any level whatever the line order, a name prefix, a tie after `~` expansion, a hand-written Default line never matched, a `new-window` in a subgroup's folder joining it, New tab in a group's own folder, in the first of its nearest ancestor with folders (one and two levels up) and at home, each in the group asked for |
| `tabrail-timer-state-test` | `tabrail-timer` and its state file alone, no tmux server: toggle, start twice, reset, rename, no file, usage, garbage dropped, racing writes |
| `tabrail-shown-test` | Only the sidebar shown works: which sidebars write per tab switch, hook burst, detach, attach, width set while hidden, new tab, second session and `switch-client`, on throwaway tmux servers |
| `tabrail-perf-test [N…]` | Speed at N tabs (default 5 and 50) on throwaway tmux servers, the reader bar on: per N one line with the tab switch median and max (until the new tab's sidebar marks it), the same under Claude hook calls (one every 100 ms unless 4 run) and the rate they reached, the reader's median (until it is in the new tab), idle tmux CPU and ms per hook call. `SETTLE=` seconds to wait before measuring (60 + 2 per tab), `IDLE=` seconds of idle measured (60), `HOOK=` the hook (`~/Dev/vmConfig/tmux-claude-state`; missing: n/a); `5 50` takes ~12 min. On stderr: waits timed out after 10 s (their values near 10000 ms) and idle halves too far apart to be steady; exit 1 when setup times out |
| `tabrail-chat-test-lib`, `tabrail-chat-fake` | A 120x4 pane for `tabrail-chat` and helpers for rows, keys and the wheel on pane `C` (`main`; only the helpers with `HELPERS_ONLY=1`); the fake claude, canned stream-json per question |
| `tabrail-test-lib` | What the tests share: the throwaway servers, clicks, checks, slots and their processes |
| `tabrail-act` | What the menus do outside the groups file: new tab, duplicate, fold (the groups file: tabrail-groups) |
| `tabrail-groups`, `tabrail-groups-new`, `tabrail-groups-move` | The groups file: its tree (`tree`: depth, name and icon per group, the pickers showing depth and name; `folder`: where a group's new tab opens, its first folder, else the first of its nearest ancestor that has folders) and its writers, for the menus and group drags: rename group (only the name: indent and spacing kept), new group, move group, nest; rename and move write through a symlink, keep the mode and rename into place; `-new` (sourced): new group, at the end or with a parent as its subgroup after the parent's subtree (written as rename and move are, if exactly that line was added); `-move` (sourced): `move-group` (a group's tree next to a sibling's: above it or below its tree) and `nest` (a group's tree after another's, one level deeper; no parent: top-level after the last group line), only the moved group lines re-indented, 2 spaces per level, written if the lines but for leading whitespace stayed the same |
| `tabrail-timer` | The group timers' state file, its only writer: start, stop, toggle, reset, rename (under flock, renamed into place) |
| `tabrail-move`, `tabrail-pick` | Move a tab to a group in a picker (the tree, indented); the picker (Claude Code's option-list style, fzf), or a name field with `-q` |
| `tabrail-nest` | The group menu's Move into group…: Top level and the tree but the group's own in a picker, the pick through `tabrail-groups nest` |
| `tabrail-colour` | The group menu's Colour…: `palette` as swatches in a picker, the pick written into column 2 of the group's line (through a symlink, mode kept, renamed into place) |
| `palette` | The group colours, `#rrggbb Name` per line: Colour…'s swatches and New group…'s tint (the first no group uses) |
| `tabrail-name` | The menus' Rename, Rename group, New group… and New subgroup…, and a header's `+`: a name field in a popup, Esc cancels whatever `status-keys` is; for the `+` (no client given) on the terminal showing the sidebar's tab |
| `groups.conf.example` | The groups file format, subgroups by indentation |
