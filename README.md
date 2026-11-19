# tabrail

A vertical tab rail for tmux: a sidebar on the left of every tab that lists all tabs of the session,
grouped by project, with what each one is doing.

```
▾ ◆ Website
 ❯ Fix login redire ✳ ⚙ 2m         Claude's topic, cut; Claude running a command for 2 min
   website·release-2 ✳ ◇           folder and git branch; Claude idle
   deploy · vim                    named by hand; vim runs in it
▾ ▲ Infra
   lde@app01 in · ssh ⏱ 4m         ssh: remote user, host, folder, cut; 4 min
   infra ✗ 2                       the last command failed
            14:41  Wed 18.11.
```

- Groups by folder (groups.conf), each with a faint tint and an icon; fold them with a click.
- Tab names: a name given by hand, else Claude Code's topic, the running command (without `VAR=value`
  in front), or folder and branch. An ssh tab shows the remote's `user@host folder`, vim's file
  (`vim nginx.conf`), else what the remote titles, or the host until the remote sends a title.
- After the name, dimmed, the tab's foreground program as tmux reports it (`deploy · vim`,
  `x.py · python3`, `deploy · sudo` while `sudo vim` runs), `✳` for Claude Code; nothing while a shell
  is the foreground program (an idle prompt or a shell script) or when the name's first word is the
  program (`htop`, `make test`). A long name is cut first; the program is left out where it would leave
  fewer than 8 of the name's characters (or cut a shorter name), so it can come and go as a timer grows.
  The status is never cut.
- Status: Claude's state with its time while `claude` is the tab's foreground program (`… ✎ ⚙ ◎ ⌕ ⧉`
  working, `● waits`, `✓ 3m` done, `◇` idle), `⏱` for a command running 3 s or more, `✗` for a failed
  one, `✓` when something finished out of sight.
- Mouse: click a tab to go there, drag it onto another tab or a group, right-click for menus (new tab,
  new group, rename, duplicate, move to group, close, fold).
- One width: every tab's sidebar keeps the same width through terminal resizes and layout commands;
  drag any sidebar's border (or `resize-pane` it) to change it for all tabs until tmux stops.
- Fast: each sidebar keeps one tmux control-mode connection and redraws when tmux reports a change;
  nothing is spawned on a tab switch (~15 ms until the sidebar shows it). Rust, standard library only.

## Setup

Needs tmux (tested with 3.8), `fzf` for the pickers and name fields, and Rust to build.

```bash
cargo build --release --offline
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

After a rebuild, `tabrail-tmux restart` restarts every sidebar.

Optional inputs, shown when present:

- Claude Code's state: a hook that sets the window options `@claude` (idle, thinking, writing, running,
  reading, searching, delegating, waiting, done), `@claude_since` (epoch of the prompt) and
  `@claude_took` (e.g. `3m12s`), then runs `tmux set-buffer -b sidebar-refresh x`. Shown only while
  `claude` is the foreground program of the tab's active pane: after a crash the options stay set, and
  the sidebar ignores them until `claude` runs there again.
- Command timers: the shell's OSC 133 marks (`\e]133;C\a` before a command, `\e]133;D;$?\a` after).

## Files

| File | Job |
|---|---|
| `src/` | The sidebar: `main.rs` loop, `control.rs` tmux connection, `model.rs` tabs and status, `name.rs` tab names, `ssh.rs` ssh tab names, `program.rs` the program after a name, `draw.rs` screen, `input.rs` mouse, `drag.rs` left button, `menu.rs` right-click menus, `width.rs` one width for all |
| `tabrail.tmux` | What tmux.conf loads: mouse bindings, hooks, prefix M |
| `tabrail-tmux` | Housekeeping from the hooks: a sidebar for every tab, restart, close tabs with only a sidebar |
| `tabrail-act` | What the menus do: new tab, duplicate, fold, new group, rename group |
| `tabrail-move`, `tabrail-pick` | Move a tab to a group in a picker; the picker (Claude Code's option-list style, fzf), or a name field with `-q` |
| `tabrail-name` | The menus' Rename, Rename group and New group…: a name field in a popup, Esc cancels whatever `status-keys` is |
| `groups.conf.example` | The groups file format |
