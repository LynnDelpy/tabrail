# What tabrail costs today

Measured on the live server, 2026-11-24, 37 minutes after boot, 4 cores. Evidence only; the targets are in SPEC.md.

| What | Measured |
|---|---|
| Tabs, sidebars | 53 tabs, 53 `tabrail` processes, 56 tmux clients (sidebars are control clients) |
| tmux server | 745 s CPU in 2137 s: 35% of one core on average; load average 7 shortly before |
| Sidebars | ~0.1% CPU each, 151 MiB together |
| Reader slots | per tab `bash tabrail-slot` (3.9 MB RSS) waiting on `tabrail-slot infinity` (sleep, 2.4 MB) |
| Claude | 20 `claude` processes, each calling the state hook on every event |

## Per sidebar, per wake

`src/main.rs` loop: `display -p` (clock, fold, size, groups file, width fields), then `list-panes -s -t <session>` with the 23 `model::FIELDS` for every pane of the session: sidebars, slots and the reader included (at least 3 panes per tab). `pane_current_command` and `pane_current_path` make tmux read `/proc` per pane. It wakes on every `%` notification (`control.rs` maps all of them to `Changed`) and after 1 s without one. So the work per second grows with tabs × panes, all in tmux's single thread.

## One tab switch

1. `after-select-window` hook: `set -wu @finished ; set-buffer -b sidebar-refresh x` → `%paste-buffer-changed` wakes every sidebar.
2. `%session-window-changed` wakes every sidebar again.
3. `session-window-changed` hook: `run-shell -b tabrail-reader follow`: bash, `flock`, sources `tabrail-reader-size`, about ten `tmux` client calls (`display`, `list-panes` for reader, slot and sidebar, zoom flags, `resize-pane`, `swap-pane`, `last-pane`).
4. The swap fires `window-layout-changed` and `window-pane-changed` (`tabrail-reader fit`, waits on the same lock) and `%layout-change` notifications → every sidebar wakes again.

The new tab's sidebar waits in tmux's queue behind the other sidebars' `list-panes`.

## One Claude hook call

`~/Dev/vmConfig/tmux-claude-state`: bash, `jq`, `date`, one to three `tmux set`/`show` client calls, then `tmux set-buffer -b sidebar-refresh x`, which wakes every sidebar. PreToolUse and PostToolUse each call it, for every tool use of every Claude.
