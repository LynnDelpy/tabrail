### DW-1: Row width is measured in chars, so a double-width character (emoji or CJK in a tab name, group icon or group name) makes a row wider than the pane.
origin: spec-deferred 935f8e567796
location: src/draw.rs:27-30, src/draw.rs:76-78
source_spec: `1-tighter-rows-without-numbers.md`
severity: low
reason: draw.rs fit() and the tab row's room/rest math use chars().count(); a 2-column character overflows the 26-column pane and wraps into the next row, and diff-only redraw can leave it corrupted. Predates this story.
status: open

### DW-2: The README example is simplified against real output: no leading blank line, no blank line after each group, no folded-group row, clock indented 12 instead of centred in 26 columns.
origin: spec-deferred 47a2fe0887f9
location: README.md:6-14
source_spec: `1-tighter-rows-without-numbers.md`
severity: low
reason: draw.rs:56 and :83 push blank lines and {clock:^width$} centres the clock at columns 5-21; the README block shows neither. The simplification predates this story, which only moved the columns and kept the clock line as the spec asked.
status: open

### DW-3: No automated test pins the sidebar layout; only the throwaway-tmux capture-pane harness checks it, and it does not run on later builds.
origin: spec-deferred c6fb5efd834d
location: src/draw.rs:54-89
source_spec: `1-tighter-rows-without-numbers.md`
severity: low
reason: The repo has no #[test] or tests/ dir; cargo test runs 0 tests. Restoring the index in draw.rs:77 still builds and passes. A std-only src/draw_test.rs (via #[cfg(test)] #[path]) calling draw::screen and stripping ANSI would pin it; it would be the repo's first test, a convention call for the owner.
status: open

### DW-4: When content overflows the pane, rows is not truncated with lines, so the clock line maps to a hidden tab or group row for clicks and right-click menus.
origin: spec-deferred 23deaa3ab39d
location: src/draw.rs:86-87
source_spec: `1-tighter-rows-without-numbers.md`
severity: low
reason: draw.rs:86-87 resizes lines to height-1 and pushes the clock but leaves rows as built, so rows.get(height) (main.rs:61, :75) returns the first hidden row's entry. A rows.truncate(height) would fix it. Predates this story; the test pane (30 rows, 11 content lines) never overflows.
status: open

### DW-5: No automated test pins tab naming (`name::bare`, `ssh::label`); only a scratch `rustc --test` crate checks it, and it does not run on later builds.
origin: bmad-build review, loop 1
location: src/name.rs, src/ssh.rs
source_spec: `2-readable-command-and-ssh-names.md`
severity: low
reason: cargo test runs 0 tests; reverting `bare(title)` to `title` in name.rs, breaking the `-vp 2222` bundle rule or the IPv4 check in ssh.rs all still build and pass. Same owner call as DW-3; when decided, the scratch crate's cases (I/O matrix plus captured vim/nvim titles) fit a std-only src/name_test.rs via #[cfg(test)] #[path].
status: open

### DW-6: The status column still follows a stale `@claude` when ssh runs in a tab whose Claude crashed or is suspended, so the row shows Claude's state instead of the `⏱` timer.
origin: bmad-build review, loop 1
location: src/model.rs:70-82
source_spec: `2-readable-command-and-ssh-names.md`
severity: low
reason: `model::status` returns on any non-empty f[7] (@claude) before the running-command timer. Story 2 made the name prefer foreground ssh over @claude, not the status. @claude is unset on SessionEnd (vmConfig/tmux-claude-state:16), so it is stale only after a crash or with Claude suspended. Predates this story.
status: open

### DW-7: A local vim or nvim tab shows its raw title (`nginx.conf (/etc/nginx) - VIM`), while over ssh the same title reads `vim nginx.conf`; a local `a=b.txt (/etc) - VIM` even reads `(/etc) - VIM` because `bare` takes `a=b.txt` for an assignment.
origin: bmad-build review, loop 2
location: src/name.rs:37-38, src/ssh.rs (editor)
source_spec: `2-readable-command-and-ssh-names.md`
severity: low
reason: The running-command branch shows `bare(title)` and has no editor parse; story 2 scoped `vim file` naming to ssh tabs. Trying `ssh::editor(title)` before `bare(title)` there would give one name per app everywhere and settle the local half of R22.
status: open

### DW-8: No automated test pins the menu item strings (`menu::for_row`): the namer quoting, `%NN` written `pNN` for `tabrail-act new-group`, and the `{{`/`\` escapes.
origin: bmad-build review, loop 1
location: src/menu.rs
source_spec: `1-name-field-where-esc-cancels.md`
severity: low
reason: cargo test runs 0 tests; only a scratch throwaway-tmux harness checks the naming items. `for_row` is pure, so a std-only #[cfg(test)] in menu.rs could assert the three namer items. Same owner call as DW-3/DW-5.
status: open

### DW-9: AGENTS.md's tmux pitfalls miss two lessons from naming-escape: tmux defaults `status-keys` to vi when EDITOR or VISUAL contains `vi` (Esc then does not cancel `command-prompt` or `confirm-before`), and `display-popup` takes argv, so pass values as arguments (bash `printf %q` gives `$'…'` under a C locale).
origin: bmad-build review, loop 1
location: AGENTS.md
source_spec: `1-name-field-where-esc-cancels.md`
severity: low
reason: Both cost a planning round in this story; editing agent-context files is the owner's call.
status: open
