# Stack: the bar and the frontend

## The bar (tmux)

- `tabrail-reader` places the pane with full-size splits: `split-window -fv` / `join-pane -fv` against any pane of the target window put it across the bottom at full width.
- Sizes stay in `tabrail-reader-size` (`SMALL`, `big`, `small`, `fit`); the hooks in `tabrail.tmux` and the calls from `tabrail-tmux` keep their names and arguments.

## The frontend (`tabrail-chat`)

- A second binary in the tabrail crate (`[[bin]]` in `Cargo.toml`), built with `cargo build --release --offline` beside `tabrail`; its own modules, files under 100 lines.
- Invoked as `tabrail-chat [claude options…]`; the folder and environment are the caller's. It runs:
  `claude -p --input-format stream-json --output-format stream-json --verbose --include-partial-messages --permission-prompt-tool stdio <claude options…>`
- Input: one JSON object per line on claude's stdin. A question is
  `{"type":"user","message":{"role":"user","content":"<text>"}}`.
- Output: one JSON object per line. Text arrives as `stream_event` → `content_block_delta` with `text_delta`; a tool call as `content_block_start` with a `tool_use` block, its input completed by the `assistant` message; a turn ends with a `{"type":"result",…}` line. `system` lines (hooks, init) are skipped.
- Control protocol, verified on claude 2.1.323 (`.memlog.md` event):

  | Frontend sends `control_request` subtype | Reply |
  |---|---|
  | `initialize` (once, first) | `control_response` success |
  | `interrupt` (Esc) | success, `still_queued` |
  | `set_model` with `model` (`/model`) | success, `model` |

  Claude sends `control_request` subtype `can_use_tool` (`tool_name`, `input`, `description`) and blocks the turn until the frontend answers with a `control_response` carrying the same `request_id` and `{"behavior":"allow","updatedInput":<input>}` or `{"behavior":"deny","message":"…"}`.
  Replies arrive out of order: match by `request_id`.
- `/clear`: end the claude process and start it again without `--continue`, `-c`, `--resume <id>` or `-r <id>`.
- Terminal: raw input with `stty` as the sidebar does (`src/main.rs`), alternate screen, SGR mouse reporting (`?1000h ?1006h`) for the wheel; redraw only changed lines (`draw::changes` pattern).
