# Brownfield: how resizes break the sidebar today

Measured 2026-11-19, tmux 3.8, throwaway server, 3 tabs, sidebar 26 and content 93 at 120 columns.

## Causes

- tmux spreads a window resize evenly over its panes. 120 to 73 columns took the sidebar 26 to 2 and the
  content 93 to 70; 73 to 93 gave 12 and 80. Widening never returns the sidebar to 26.
- `window-size` latest (tmux default): only a window a client shows is resized. Other tabs keep their size
  until visited, then drift on their own; three tabs ended at 21, 26 and 12.
- Border drag is tmux's default `MouseDrag1Border` → `resize-pane -M`: one pane, nothing records the result.
  tmux offers `MouseDragEnd1Border`, unbound by default.
- New sidebars: `tabrail-tmux ensure` splits with `-l "$WIDTH"`, `WIDTH=26`.
- Stale rows: on a width change tmux reflows the pane's grid. `draw::changes` rewrites only lines whose
  text changed; blank rows (`String::new()`: row 1, gaps between groups, padding above the clock) never
  change, so leftovers stay. After a 42 to 26 drag, row 1 read `            2 tmp`. `\e[2J` clears only
  when the line count (height) changes.
- The sidebar already hears of resizes: control mode sends `%layout-change`, which wakes the loop
  (`Event::Changed`), and the next frame reads `pane_width`.
- Not affected: closing a content split gives its columns to the right neighbour, not the sidebar; a
  content split divides only the content pane. `select-layout even-horizontal` does equalise all panes
  (sidebar 40 at 120 columns).

## Repro

```bash
tmux -L rsz -f test.conf new -d -s t -x 120 -y 40    # test.conf: mouse on, source-file tabrail.tmux
tmux -L rsz new-window -d; tmux -L rsz new-window -d
tmux -L outer -f /dev/null new -d -x 120 -y 40 "TMUX= tmux -L rsz attach -t t"
tmux -L outer resize-window -x 73; tmux -L outer resize-window -x 200
tmux -L rsz list-panes -a -F '#{window_index} #{?@sidebar,sb,  } #{pane_width}'
# border drag, sidebar 42 wide (border column 43): press, drag, release
tmux -L outer send-keys -l $'\e[<0;43;10M' $'\e[<32;27;10M' $'\e[<0;27;10m'
```

`tmux -L outer` without `-f /dev/null` loads `~/.tmux.conf`: the viewer then runs its own tabrail sidebar
plus resurrect and continuum.
