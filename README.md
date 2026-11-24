# tabrail

A vertical tab rail for tmux: a sidebar on the left of every tab that lists all tabs of the session,
grouped by project, with what each one is doing.

```
▾ ◆ Website
 ❯ Fix login redire ✳ ⚙ 2m         Claude's topic, cut; Claude running a command for 2 min
   website·release-2 ✳   ◇         folder and git branch; Claude idle
   deploy · vim                    named by hand; vim runs in it
▾ ▲ Infra
   lde@app01 in · ssh ⏱ 4m         ssh: remote user, host, folder, cut; 4 min
   infra               ✗ 2         the last command failed
    14:41  Wed 18.11.
```

- Groups by folder (groups.conf), each with a faint tint and an icon; fold them with a click, reorder
  them by dragging. The order is the groups file's line order, which is also the order a new tab's
  folder is matched in (the first group whose folder holds it); Default is always last.
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
  drag it onto another group's row or one of its tabs to move its line in the groups file there (above
  that group moving up, below it moving down); right-click for menus (new tab, new group, rename,
  duplicate, move to group, close, fold).
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
  in the bar copies its text as in any pane (copy mode). Permission prompts are denied for now
  (`--permission-prompts none`). As the reader (it finds its pane by its terminal) it sizes the bar
  while the bar has the keyboard: when the input or the conversation changes, to the rows of the
  input and the latest question and answer, rounded up to a multiple of 4, from 4
  rows to a third of the window, at most once every 500 ms. While a new question is typed (no turn
  running) only the input counts, with one row above it, so a short question after a long answer
  stays at 4 rows; from Enter on the answer counts again. `▲` at the end of the input's last row
  (after the timer) grows the bar by a sixth of the window, up to two thirds, `▼` shrinks it by a
  sixth, down to 4 rows; that size holds until the next question or until the bar is back at 4
  rows. A double-click does nothing to it.
- Fast: each sidebar keeps one tmux control-mode connection and redraws when tmux reports a change;
  nothing is spawned on a tab switch (~15 ms until the sidebar shows it) but, with the reader pane, the
  helper that moves it. Rust, standard library only.

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
  panes are slots again: the slot waits in a child process named `tabrail-slot` for that. The reader
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
| `src/*.rs` | The sidebar: `main.rs` loop, `control.rs` tmux connection, `model.rs` tabs and status, `name.rs` tab names, `ssh.rs` ssh tab names, `program.rs` the program after a name, `draw.rs` screen, `input.rs` mouse, `drag.rs` left button, `menu.rs` right-click menus, `width.rs` one width for all |
| `src/chat/` | `tabrail-chat`: `main.rs` loop and drawing, `chat.rs` keys and claude's events, `session.rs` claude across restarts (`--resume`), `claude.rs` the claude process, `stream.rs` its stream-json lines, `json.rs` JSON, `view.rs` the conversation, `input.rs` the input, `keys.rs` keys, `mouse.rs` wheel and clicks, `term.rs` the terminal, `bar.rs` the reader bar's height and buttons, `tmux.rs` its pane and `tabrail-reader size` |
| `tabrail.tmux` | What tmux.conf loads: mouse bindings, hooks, prefix M and & |
| `tabrail-tmux` | Housekeeping from the hooks: a sidebar (and a reader slot) for every tab, restart, close tabs with only a sidebar (and the reader pane or its slot), the menus' Close |
| `tabrail-reader`, `tabrail-reader-size` | The reader pane, a bar across the bottom: a slot in every tab, swap places with the shown tab's slot, start in it, move away before its tab closes; its height as tabrail-chat asks (`size`); back across the bottom when tmux moved it or a slot |
| `tabrail-slot` | A tab's slot: marks its pane, then waits in a child named `tabrail-slot` (for resurrect) |
| `tabrail-reader-test`, `tabrail-slot-test` | The reader bar's tests and its slots' (with ~/Dev/vmConfig's mouse config and a resurrect save and restore), on throwaway tmux servers |
| `tabrail-chat-test`, `tabrail-chat-session-test` | `tabrail-chat`'s tests on a throwaway tmux server: what it shows and its keys; its claude's options, restarts and the way out |
| `tabrail-bar-test` | The reader bar sized by `tabrail-chat` in it (its height, buttons, away, tab switch, zoom), against the fake claude on throwaway tmux servers |
| `tabrail-chat-test-lib`, `tabrail-chat-fake` | A 120x4 pane for `tabrail-chat` and helpers for rows, keys and the wheel on pane `C` (`main`; only the helpers with `HELPERS_ONLY=1`); the fake claude, canned stream-json per question |
| `tabrail-test-lib` | What the tests share: the throwaway servers, clicks, checks |
| `tabrail-act` | What the menus and group drags do: new tab, duplicate, fold, new group, rename group, move group |
| `tabrail-move`, `tabrail-pick` | Move a tab to a group in a picker; the picker (Claude Code's option-list style, fzf), or a name field with `-q` |
| `tabrail-name` | The menus' Rename, Rename group and New group…: a name field in a popup, Esc cancels whatever `status-keys` is |
| `groups.conf.example` | The groups file format |
