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

- source_spec: `_bmad-output/implementation-artifacts/spec-agent-journal.md`
  summary: After a tmux-resurrect restore the reader pane may come back unmarked (no `@reader`), as a stray pane under a sidebar, possibly relaunched by the `~claude->withproxy claude --continue` rule, while `follow` starts a second reader (medium, unverified).
  evidence: resurrect saves every pane and the pane's child command but no user pane options; the reader has no shell, so what it saves for it is unknown. Settle with a save and restore on a `-L` test server with `@resurrect-dir` in a scratch folder; likely fix is `start` marking its own pane plus a `"~claude-haiku-5-5->~/Dev/vmConfig/journal-reader/start"` rule ahead of `~claude` in tmux.conf.
- source_spec: `_bmad-output/implementation-artifacts/spec-agent-journal.md`
  summary: AGENTS.md lacks the reader-pane pitfalls (split-window, join-pane and resize-pane unzoom a tab; `#{P:}` guards in FORMAT and hooks) and the harness's outer server should start with `-f /dev/null`.
  evidence: `tmux -L outer new …` as AGENTS.md writes it loads ~/.tmux.conf with resurrect, continuum and a second tabrail; the implementer had to work around it.
- source_spec: `_bmad-output/specs/spec-reader-bar/stories/5-slot-in-every-tab-grow-on-double-click.md`
  summary: AGENTS.md does not list the new pane option `@reader_slot` among the names running sidebars find each other by, nor story 5's tmux pitfalls.
  evidence: AGENTS.md:26-30 lists `@sidebar`, `@reader`, `@reader_big`… but not `@reader_slot`, which src/model.rs, src/width.rs and the bash helpers now use. Pitfalls found in story 5 live only in its Implementation Notes: `swap-pane -d` hands the leaving tab's keyboard to the slot (needs `last-pane`); `DoubleClick1Pane` fires after the second release; a pane in copy mode takes mouse keys from the mode's table before root; resurrect's ps strategy saves only the pane pid's direct children. Agent-context file, so deferred.
- source_spec: `_bmad-output/specs/spec-reader-bar/stories/5-slot-in-every-tab-grow-on-double-click.md`
  summary: Sidebars and the reader pane do not come back through resurrect; a restore leaves a stray shell in each sidebar's cell (and the reader's), and ensure adds new sidebars.
  evidence: medium (unverified by a live restore). The user's save file records sidebars as `:tmux -C attach-session …` (their child process); `~release/tabrail` in @resurrect-processes is a regex on that string and never matches; the reader pane is saved with no command. Pre-existing, found by story 5's restore test; settle with a restore on a -L server that counts every pane per tab.
- source_spec: `_bmad-output/specs/spec-reader-bar/stories/5-slot-in-every-tab-grow-on-double-click.md`
  summary: The reader and ensure locks (/tmp/tabrail-reader-$UID.lock, /tmp/tabrail-$UID.lock) and ensure's restore wait (pgrep restore.sh) are per user, not per tmux server, so test servers and the live one serialize each other.
  evidence: low. While a tabrail-*-test runs, a live `follow` can wait up to 5 s and give up (`flock -w 5 8 || exit 0`), and a live ensure waits during the slot test's restore. Pre-existing; key the locks by `#{pid}` and scope the restore wait to the server.
- source_spec: `_bmad-output/specs/spec-reader-bar/stories/5-slot-in-every-tab-grow-on-double-click.md`
  summary: A double-click on a sidebar row selects the sidebar pane (keyboard moves into it), unlike a single click.
  evidence: low. The DoubleClick1Pane else branch (tmux's default, and vmConfig's tmux-mouse.conf binding) starts with `select-pane -t =`; MouseDown1Pane in tabrail.tmux skips select-pane for `@sidebar`. Pre-existing: tmux's default did the same before story 5.
- source_spec: `_bmad-output/specs/spec-reader-bar/stories/2-tabrail-chat-frontend.md`
  summary: tabrail-chat starts claude in its own process group on the pane's terminal, so a tool that opens /dev/tty (sudo, ssh, git credential prompts) could read keys typed into the bar, and the SIGKILL path leaves claude's tool processes behind.
  evidence: medium (unverified). claude.rs spawns with no process_group/setsid and kills only claude's pid. Whether Claude Code's Bash tool detaches its children from the controlling terminal is not checked; settle with a fake tool that opens /dev/tty under real claude in a -L pane. Possible fix: `CommandExt::process_group(0)` and `kill(-pgid)`, relying on stdin EOF when the pane closes.
- source_spec: `_bmad-output/specs/spec-reader-bar/stories/2-tabrail-chat-frontend.md`
  summary: AGENTS.md does not describe the second binary (tabrail-chat) nor its terminal pitfall.
  evidence: with autowrap off, an erase after a row that fills the width takes the row's last character, so rows are cleared (`ESC[2K`) before they are written (src/chat/main.rs changes). Agent-context file, so deferred.
- source_spec: `_bmad-output/specs/spec-reader-bar/stories/4-journal-reader-on-the-frontend.md`
  summary: tabrail-chat-session-test checks the restart drops `-c` but never `--continue`, the flag journal-reader/start actually passes.
  evidence: low. Narrowing `"--continue" | "-c"` in src/chat/session.rs to `"-c"` passes every tabrail-chat test; the reader would then restart with `--continue --resume <sid>`. Story 4 may not edit tabrail; add `--continue` beside `-c` at tabrail-chat-session-test:9 and its expected args.
- source_spec: `_bmad-output/specs/spec-reader-bar/stories/4-journal-reader-on-the-frontend.md`
  summary: After a resurrect restore the reader's cell comes back running a full `withproxy claude --continue` TUI, because the reader's child `claude -p … --continue` matches `~claude` in @resurrect-processes.
  evidence: medium. With tabrail-chat as the reader, the pane pid is tabrail-chat and its child is `claude -p --input-format stream-json …`, which resurrect's ps strategy saves; `~claude` is a substring match. Before story 4 the reader's own pid was claude, saved with no command. A fix needs both a narrower entry (resurrect's non-`~` match is `^name( |$)`, so tabrail-chat would also start claude by absolute path) and a restore test on a -L server; overlaps the story-5 deferral on sidebars and the reader not restoring.
- source_spec: `_bmad-output/specs/spec-reader-bar/stories/4-journal-reader-on-the-frontend.md`
  summary: journal-reader/start's TUI fallback ends at once with "No conversation found to continue" when the folder has only tabrail-chat (`claude -p`) transcripts.
  evidence: medium. Claude Code's TUI `--continue` skips transcripts written by `claude -p` (checked with one transcript under both labels); `start` passes `--continue` whenever any transcript exists. Today 4 TUI transcripts keep it working; after Claude Code's 30-day cleanup the fallback would end every time and tabrail would restart it every 10 s. Only the fallback is affected (tabrail-chat not built, or outside tmux). Fix: give the TUI `--continue` only for a TUI transcript, or none.
- source_spec: `_bmad-output/specs/spec-reader-bar/stories/6-the-bar-sizes-to-the-chat.md`
  summary: AGENTS.md still calls `@reader_big` "whether it is grown" (it now holds the bar's height in rows) and lacks story 6's cross-process contract and pitfall.
  evidence: tabrail-chat runs `$(tmux show -gv @tabrail_dir)/tabrail-reader size PANE ROWS|up|down`; the reader command unsets TMUX_PANE, so tabrail-chat finds its pane by `#{pane_tty}` against /proc/self/fd/0. Agent-context file, so deferred.
- source_spec: `_bmad-output/specs/spec-performance/stories/1-speed-test-at-5-and-50-tabs.md`
  summary: Test servers take the same `/tmp/tabrail-$UID.lock` and `/tmp/tabrail-reader-$UID.lock` as the user's live tmux, so a test run can make a live `tabrail-reader follow` give up (5 s) and live switches leak into test reader times.
  evidence: medium, pre-existing. The lock paths in tabrail-tmux and tabrail-reader are keyed by UID, not by tmux server; every tabrail-*-test shares them, tabrail-perf-test holds the first for 49 ensures in a row at 50 tabs. Fix: key the locks by the server's socket or pid.
- source_spec: `_bmad-output/specs/spec-performance/stories/1-speed-test-at-5-and-50-tabs.md`
  summary: Story 1's AC 3 and its manual check freeze a sidebar with `kill -STOP`, which tmux undoes, and AGENTS.md lacks that pitfall; the working check (XOFF) is only in the story's Implementation Notes.
  evidence: medium. tmux sends SIGCONT to a pane process that stops (a stopped pane ran again within 50 ms, tmux 3.8). Freezing the sidebar's output works: `t send-keys -t <tab K's sidebar> C-s` before the switch, `C-q` after, gives `max≈10000ms` and `tabs=N: 1 waits timed out` on stderr (the sidebar's `stty -icanon -echo` leaves IXON on). Fix: reword AC 3 and the Verification bullet, add the pitfall to AGENTS.md (agent-context file).
- source_spec: `_bmad-output/specs/spec-performance/stories/1-speed-test-at-5-and-50-tabs.md`
  summary: `tabrail-perf-test` matches the `for t in ./tabrail-*-test` sweep in stories 2-4's verification, adding ~12 min per sweep and a pass that carries no signal (it always exits 0).
  evidence: medium. `2-hidden-sidebars-stay-idle.md:84` runs the glob; SPEC.md:44 asks every `tabrail-*-test` to pass. The name is fixed by story 1's intent; exclude it in those sweeps (e.g. `[[ $t == *perf-test ]] && continue`).
- source_spec: `_bmad-output/specs/spec-performance/stories/2-hidden-sidebars-stay-idle.md`
  summary: `tabrail.tmux:44` (`pane-command-finished`) tests `session_attached`, which counts the sidebars' control clients, so a finished command in the current tab gets no ✓ while no terminal is attached.
  evidence: medium, pre-existing. Story 2 found `session_attached` counts control clients; the condition `#{!:#{&&:#{window_active},#{session_attached}}}` is then false for the current tab even when detached. Fix: count terminals (`#{session_attached}` minus control clients, or a `list-clients` filter).
- source_spec: `_bmad-output/specs/spec-performance/stories/2-hidden-sidebars-stay-idle.md`
  summary: AGENTS.md tmux pitfalls lack story 2's findings: `swap-window` onto the current index sends no `%session-window-changed` and no hook; `session_attached` counts control clients; `%client-session-changed`/`%client-detached` fire for control clients (named `client-<pid>`), command clients send none.
  evidence: low. Verified on tmux 3.8 during story 2; the fix edits an agent-context file.
- source_spec: `_bmad-output/specs/spec-performance/stories/2-hidden-sidebars-stay-idle.md`
  summary: Nothing automated checks that the shown sidebar's `list-panes` lists only tabs' own panes, and `tabrail-perf-test` always exits 0, so the story 2 speed-up can regress silently.
  evidence: medium. Reverting the `-f` filter passes every test (`model::tabs` drops the same rows); a timing threshold would flake on the shared VM. A check could count list-panes rows via a tmux `-v` log or a wrapper.
- source_spec: `_bmad-output/specs/spec-performance/stories/2-hidden-sidebars-stay-idle.md`
  summary: Two story 2 paths have no test: the open-menu mark forgotten when a sidebar is hidden (`main.rs:57`), and notifications kept while an `ask` waits (`control.rs:46`).
  evidence: low. No test sends a right-button event; a deterministic note-during-ask test needs a fake `Tmux` built from a channel instead of `ChildStdin`.
- source_spec: `_bmad-output/specs/spec-performance/stories/4-claude-hook-in-one-tmux-call.md`
  summary: ~/.claude/settings.json calls tmux-claude-state for six events but not PreToolUse, so the writing/running/reading/searching/delegating states never show.
  evidence: jq over settings.json hooks lists SessionStart, UserPromptSubmit, PostToolUse, Notification, Stop, SessionEnd; the hook's header, cost-today.md and the perf test's Pre/Post load assume PreToolUse.
- source_spec: `_bmad-output/specs/spec-group-timer/stories/1-group-timer-from-the-menu.md`
  summary: The timers file's fallback path (`${XDG_STATE_HOME:-$HOME/.local/state}/tabrail/timers`) is worked out twice, in `src/timer.rs` and `tabrail-timer`, and no test runs with `XDG_STATE_HOME` unset to check they agree.
  evidence: low. Every test exports `XDG_STATE_HOME`; a drift (e.g. Rust not treating an empty value as unset) would show a dead timer while the file is written. A check needs a server started with a temp HOME and no XDG_STATE_HOME.
- source_spec: `_bmad-output/specs/spec-group-timer/stories/1-group-timer-from-the-menu.md`
  summary: AGENTS.md does not name the timers state file: Conventions lacks `$XDG_STATE_HOME/tabrail/timers` (+ `timers.lock`, `tabrail-timer` its only writer), and Building and testing does not say a hand-run `tmux -L test` server needs `XDG_STATE_HOME` on a temp dir.
  evidence: medium. Timer state is per user, not per server: a test server's menu or rename-group writes the user's real timers. `tabrail-test-lib` now sets it for the scripted tests; manual test servers still do not. Agent-context file, so deferred.
- source_spec: `_bmad-output/specs/spec-group-timer/stories/1-group-timer-from-the-menu.md`
  summary: rename-group with an unwritable groups.conf is untested: the timer must stay under OLD (the `&&` after the `mv`).
  evidence: low. Putting the call back before the awk or `;` instead of `&&` passes every test; a check needs `chmod a-w` on the groups folder around the call.
- source_spec: `_bmad-output/specs/spec-nested-groups/stories/1-show-nested-groups.md`
  summary: `tabrail-groups rename-group` sets the tabs' `@group` and `@sidebar_fold` to NEW even when the groups-file write failed, so those tabs vanish from every sidebar (draw.rs draws only groups in the file).
  evidence: medium. The loops run after `fi`, outside the success branch, as they did after `> $CONF.new && mv` before; a read-only folder of the real groups file triggers it. Fix: move them and `refresh` into the `then` branch.
- source_spec: `_bmad-output/specs/spec-nested-groups/stories/2-longest-folder-wins.md`
  summary: `tabrail-tmux ensure` (after-new-window, `run-shell -b`) can overwrite the `@group` that `tabrail-act new` sets right after `new-window`, if it reads the empty option first; a folderless subgroup's tab then lands in its parent (before nesting: Default).
  evidence: low. Needs ensure's read to precede tabrail-act's `set -w`, which pgrep and flock make rare; `tmux set -wo` in ensure (2>/dev/null) would let an @group already set win.
- source_spec: `_bmad-output/specs/spec-nested-groups/stories/2-longest-folder-wins.md`
  summary: A groups-file folder written with a trailing slash (`~/src/web/`) never matches in `tabrail-tmux group()`; `/` matches only itself.
  evidence: low. `[[ /a/b/c == "/a/b/"/* ]]` is false and pane_current_path has no trailing slash; same before longest-match. Fix: `dir=${dir%/}`, keeping `/`.
- source_spec: `_bmad-output/specs/spec-nested-groups/stories/2-longest-folder-wins.md`
  summary: `tabrail-tmux group()` ignores the groups file's last line when it has no trailing newline (`while read` drops it).
  evidence: low. src/groups.rs reads it, so the group shows but takes no tabs by folder; same before. Fix: `|| [[ -n $name ]]` on the loop.
