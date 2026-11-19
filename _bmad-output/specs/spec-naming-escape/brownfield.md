# Brownfield: why Escape does not leave naming today

Measured 2026-11-19, tmux 3.8, throwaway server started with `EDITOR=/usr/bin/vim`.

## Naming sites (`src/menu.rs`)

| Menu item | Row | Today |
|---|---|---|
| Rename | tab | `command-prompt -I '<name>' -p 'Rename tab:' { rename-window -t <@id> '%%' }` |
| Rename group | group (not Default) | `command-prompt -I '<name>'`, then `tabrail-act rename-group <old> '%%'` |
| New group… | tab | `command-prompt`, then `tabrail-act new-group '%%' <@id>`: the tab moves in |
| New group… | group, empty space | same with `p<NN>` for pane `%NN`: a new tab opens in it (`%` would read as `%1`) |

## Cause

- tmux sets `status-keys` to vi when `VISUAL` or `EDITOR` contains `vi`; the user's `EDITOR` is `/usr/bin/vim`
  and tmux.conf does not set `status-keys`.
- vi prompt: Escape switches to normal mode, further Escapes do nothing, `x` then deletes; only `q` or `C-c`
  cancel. emacs prompt: one Escape cancels. tmux has no per-prompt key setting.
- `confirm-before` (Close) under vi keys: Escape leaves the `(y/n)` prompt open.

## Precedent

`tabrail-move` opens `tabrail-pick` in `display-popup -B -E` from the tab menu (`run-shell -b` with the
client name, then `display-popup -c`). fzf exits 130 on Esc; the pickers' footer reads
`Enter to select · ↑/↓ to navigate · Esc to cancel`.

## Test notes

`command-prompt` and `confirm-before` run from a shell block until the prompt is dismissed: pass `-b`.
Keys reach a prompt or popup through the viewer: `tmux -L outer send-keys Escape`.
