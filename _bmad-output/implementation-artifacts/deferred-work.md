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
status: resolved
resolution: spec-tab-program CAP-4, `model::status` reads @claude only while pane_current_command is claude

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

### DW-10: No automated test pins this story's rules: the live-Claude gate (`model::status`, `Facts.claude`), `program::mark` and the row's drop, cut and trim math; only a scratch `rustc --test` crate checks them.
origin: bmad-build review, loops 1-3
location: src/model.rs, src/program.rs, src/draw.rs
source_spec: `spec-tab-program.md`
severity: low
reason: cargo test runs 0 tests; mutants (ungated `@claude`, `shell(name)`, no basename, no `least` check, no `trim_end`, `min(7)`) all build and pass. Same owner call as DW-3/DW-5/DW-8. When decided, the scratch cases fit a std-only src/rows_test.rs via #[cfg(test)] #[path] feeding `model::tabs` FIELDS lines into `draw::screen` at width 26: the I/O matrix of spec-tab-program.md plus `./target/release/x` -> `· x`, a cut at a space (`lde@app01 · ssh ⏱ 1h6m`), `deploy`+`terraform` showing at `⏱ 9s`, dropped at `⏱ 10s`, back at `⏱ 1m`, and every row exactly 26 wide with ` ❯ ` plus min(8) name characters.
status: open

### DW-11: AGENTS.md's tmux pitfalls miss how `pane_current_command` is made: the argv0 of the foreground process group's leader, basenamed only when absolute.
origin: bmad-build review, loop 2
location: AGENTS.md
source_spec: `spec-tab-program.md`
severity: low
reason: Cost two review findings: `sudo vim` reports `sudo`, npm retitles itself so `npm run build` reports `npm` (not `node`), and `./target/release/x` stays whole. Editing agent-context files is the owner's call.
status: open

### DW-12: A program suspended with Ctrl-Z shows `✗ 148` in the fail colour, now a suspended Claude too.
origin: bmad-build review, loop 3
location: src/model.rs (status), ~/Dev/vmConfig/tmux.zsh
source_spec: `spec-tab-program.md`
severity: low
reason: zsh reports 148 after a suspend and tmux.zsh sends it as the OSC 133 exit status, so `status` takes the fail arm. Before spec-tab-program a stale `@claude` masked it for Claude; vim and others already showed it. Treating 148 as no failure would fix it.
status: open

### DW-13: SPEC.md and stories.yaml of spec-sidebar-width disagree with what story 2 built.
origin: bmad-build review, oneshot
location: _bmad-output/specs/spec-sidebar-width/SPEC.md, stories.yaml
source_spec: `specs/spec-sidebar-width/stories/2-one-sidebar-width-set-by-dragging.md`
severity: low
reason: SPEC assumes a keyboard `resize-pane` is undone (story 2 adopts it like a drag); open question 2 is half answered (a layout command restores the width; a layout that moves the sidebar off the left column is left alone); stories.yaml calls `MouseDragEnd1Border` free, but ~/Dev/vmConfig/tmux-mouse.conf binds it after tabrail.tmux. Spec edits go through bmad-spec.
status: open

### DW-14: No automated test pins `width::Width::keep`, a state machine with zoomed, off-column, first-frame, clamp re-baseline, relayout, moved and adopt branches.
origin: bmad-build review, oneshot
location: src/width.rs
source_spec: `specs/spec-sidebar-width/stories/2-one-sidebar-width-set-by-dragging.md`
severity: low
reason: Verified only by hand on throwaway tmux servers. Same owner call as DW-3/DW-5/DW-8/DW-10.
status: open

### DW-15: No automated test pins group drag: fold on release and `reorder` in `drag::left`, and `tabrail-act move-group`'s line move, Default target, symlink write-through and failure cleanup; only a scratch `rustc --test` crate and a scratch bash script check them.
origin: bmad-build review, dispatch
location: src/drag.rs, tabrail-act
source_spec: `implementation-artifacts/spec-group-drag.md`
severity: low
reason: Fold-on-press, a dropped Default guard, swapped `<`/`>` in the awk or a dropped `readlink -f` all build and pass unnoticed. Same owner call as DW-3/DW-5/DW-8/DW-10/DW-14.
status: open

### DW-16: `tabrail-act rename-group` turns a symlinked groups file into a regular file, while `move-group` writes through the link.
origin: bmad-build review, dispatch
location: tabrail-act:45-46
source_spec: `implementation-artifacts/spec-group-drag.md`
severity: low
reason: `rename_group` writes `$CONF.new` and `mv`s it onto `$CONF` unresolved; after a rename, later moves edit the copy, not the link target. Predates spec-group-drag, whose Code Map left it alone.
status: open

### DW-17: A group name containing `'` breaks the fold command, and one containing `|` folds as two names.
origin: bmad-build review, dispatch
location: src/drag.rs `fold`
source_spec: `implementation-artifacts/spec-group-drag.md`
severity: low
reason: `fold` writes raw names into `set -g @sidebar_fold '…'` joined by `|`; moved verbatim from the old press branch. `tabrail-act fold` handles any name.
status: open

### DW-18: The menus' New tab and Fold items pass group names to `sh` unquoted, so a group named `R&D` runs `tabrail-act new R` in the background and then `D …`.
origin: bmad-build review, dispatch
location: src/menu.rs:26, :55
source_spec: `implementation-artifacts/spec-group-drag.md`
severity: low
reason: `run-shell -b '{h} new {group} {pane}'` and `'{h} fold {name}'`; `clean` drops quotes and `$` but not `& | < > ( ) * ?` or backticks, and New group… allows them. Group drag quotes its names since spec-group-drag review R5.
status: open

### DW-19: No automated test pins the right-aligned status: every tab row with a status ends with it in column `width`, padding before it, at any shared width.
origin: bmad-build review, oneshot
location: src/draw.rs (screen, tab row)
source_spec: `implementation-artifacts/spec-status-right.md`
severity: low
reason: Moving `{rest}` back after `{state}` builds and passes; only a throwaway-tmux capture at widths 26 and 34 checked it. Same owner call as DW-3/DW-5/DW-8/DW-10/DW-14/DW-15; DW-10's rows_test.rs could also assert that each status ends at `width`.
status: open

### DW-20: spec-status-right adopts `../spec-tab-rows/row-layout.md` as a companion, which still says "status right after the name" (columns table: `name, space, status`).
origin: bmad-build review, oneshot
location: _bmad-output/specs/spec-status-right/SPEC.md (companions), _bmad-output/specs/spec-tab-rows/row-layout.md
source_spec: `implementation-artifacts/spec-status-right.md`
severity: low
reason: Its own row-layout.md shows the right-aligned rows, so the companion set holds both rules. A line in SPEC.md that CAP-1 supersedes that part (or dropping the companion) settles it; spec edits go through bmad-spec.
status: open
