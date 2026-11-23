## tabrail (tmux sidebar)

A sidebar in the left pane of every tmux tab: all tabs grouped by project, Claude Code's state, timers,
mouse menus and dragging. Rust binary (`src/`) plus bash helpers and `tabrail.tmux`, which the user's
tmux.conf loads. `README.md` is the reference for behaviour and setup. BMAD planning output goes to
`_bmad-output/`.

## Building and testing

- `cargo build --release --offline`: standard library only, no crates (crates.io is not reachable
  without the proxy, and none are needed). After a build, `tabrail-tmux restart` restarts the sidebars.
- Never test against the user's tmux (the default socket): it runs their work, Claude chats included.
  Use a throwaway server, `tmux -L test -f <conf>`, and show it inside a second one to see and drive it:
  `tmux -L outer -f /dev/null new -d "TMUX= tmux -L test attach"` (`-f /dev/null`: not the user's
  tmux.conf with its plugins), then `tmux -L outer capture-pane -p` reads the screen and
  `tmux -L outer send-keys -l $'\e[<0;5;3M'` sends a real mouse click (SGR: button 0 left, 2 right,
  32 drag; `M` press, `m` release) through tmux's own bindings. `tabrail-reader-test` does it so.
  Menu files of a test server start with its pid: delete only those, never another server's.
- Helper scripts called from a shell outside that server need `TMUX=<its socket>,0,0`, or they act on
  the user's tmux. When a test server runs tabrail, kill its sidebars with it and never `pkill -f` by a
  pattern that also matches your own command line.

## Conventions

- Files under 100 lines; split by job (see README's Files table).
- tmux options the sidebar uses: pane `@sidebar` (this pane is a sidebar), `@reader` and `@reader_big`
  (the reader pane and whether it is grown; never `@sidebar`), window
  `@group`, `@claude*`, `@finished`, `@sidebar_relayout`, global `@sidebar_fold`, `@sidebar_width`,
  `@tabrail_groups`, `@tabrail_dir`, `@tabrail_reader` (the reader pane's command),
  `@tabrail_reader_started`. The buffer `sidebar-refresh` is the "redraw now" signal. Keep these
  names: running sidebars find each other by them.

## tmux pitfalls (each cost a debugging round)

- Control mode: every command in a `;` list gets its own `%begin/%end` reply; send one command per line
  and wait for its reply. Replies to our commands are flagged 1, the attach's own block 0.
- A menu (`display-menu`) only takes mouse clicks when tmux opens it from a mouse key binding; opened by
  a command it ignores clicks and closes on the button release (`-O` makes it not show at all). Hence
  menus are files per row that the `MouseUp3Pane` binding sources. A numeric `-y` is the menu's bottom.
- tmux redraws a pane over an open menu: draw only lines that changed (`draw::changes`) and pause
  drawing while a menu is open.
- `command-prompt` replaces `%1`…`%9` in its template, so a pane id like `%12` becomes "answer1" + "2":
  pass pane ids through prompts without the `%`.
- Pane ids repeat across tmux servers: files keyed by pane carry the server pid.
- Releasing a drag on another tab's row must not switch tabs on the press: the release then reaches
  the other tab's sidebar, which knows nothing of the drag. Switch on release.
- tmux.conf `$VAR` and `~` expand only outside single quotes; hooks run `run-shell` with formats
  expanded (`#{@tabrail_dir}`), not with tmux.conf variables.
