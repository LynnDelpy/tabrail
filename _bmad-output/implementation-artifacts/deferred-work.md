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
