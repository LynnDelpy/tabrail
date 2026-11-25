# Stack: the bar and the frontend

## The bar (tmux)

- Every tab holds one slot (pane option `@reader_slot`, process `tabrail-slot`) or the bar, never both: `tabrail-reader slots` makes missing ones with a full-size split, `split-window -fv -d -l 4` against the tab's sidebar, across the bottom at full width.
- Following a tab, the bar swaps places with that tab's slot, `swap-pane -d -s <bar> -t <slot>`: both 4 rows, so no other pane in either tab resizes (verified on tmux 3.8): first the bar goes to 4 rows in the tab it leaves (big or dragged), and the slot to 4 rows across the bottom. A new reader takes the shown tab's slot, `respawn-pane -k` with the reader command.
- `join-pane -fv` against the tab's sidebar only puts a bar or slot that tmux moved (layout command, sidebar added beside it) back across the bottom of its own tab; never into another tab.
- Sizes stay in `tabrail-reader-size` (`SMALL`, `height`, `small`, `fit`, `size`). `tabrail-chat` in the bar finds its pane by its terminal (`/proc/self/fd/0` against `#{pane_tty}`; the reader command unsets `TMUX_PANE`) and runs `tabrail-reader size <pane> <rows|up|down>`: rows its own height (4 up to a third of the window), up and down a sixth of the window (4 up to 2/3), taken only while the pane is the reader and its tab's active pane, the tab not zoomed. `@reader_big` holds a height above 4 rows, which `fit` applies again after a terminal resize unless the tab's own pane is active. tabrail binds no `DoubleClick1Pane`. The hooks in `tabrail.tmux` and the calls from `tabrail-tmux` keep their names and arguments.

## The frontend (`tabrail-chat`)

- A second binary in the tabrail crate (`[[bin]]` in `Cargo.toml`), built with `cargo build --release --offline` beside `tabrail`; its own modules, files under 100 lines.
- Invoked as `tabrail-chat [claude options…]`; the folder and environment are the caller's. It runs:
  `claude -p --input-format stream-json --output-format stream-json --verbose --include-partial-messages --permission-prompts none <claude options…>`
- Input: one JSON object per line on claude's stdin. A question is
  `{"type":"user","message":{"role":"user","content":"<text>"}}`.
- Output: one JSON object per line. Text arrives as `stream_event` → `content_block_delta` with `text_delta`; a tool call as `content_block_start` with a `tool_use` block, its input completed by the `assistant` message; a turn ends with a `{"type":"result",…}` line. `system` lines (hooks, init) are skipped.
- Control protocol, verified on claude 2.1.323 (`.memlog.md` event): the frontend sends one `control_request` subtype `initialize` first; claude replies with a `control_response` success.
- Terminal: raw input with `stty` as the sidebar does (`src/main.rs`), alternate screen, SGR mouse reporting (`?1000h ?1006h`) for the wheel and the bar's buttons (left presses); redraw only changed lines (`draw::changes` pattern).
