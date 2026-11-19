---
title: 'Show the program running in each tab'
type: 'feature'
created: '2026-11-19'
status: 'done'
route: 'dispatch'
baseline_commit: 'f626705466ba8a51b270f2afc42ffb4b8995c751'
review_loop_iteration: 2
context:
  - '{project-root}/_bmad-output/specs/spec-tab-program/SPEC.md'
  - '{project-root}/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** A tab's name shows its program only sometimes (a hand name, Claude topic or program title hides it), and a Claude tab is known only by its status glyph, which a crashed Claude leaves on a plain shell tab.

**Approach:** Story 1 (CAP-4): Claude is the live process only (`pane_current_command` = `claude`); a stale `@claude` drives neither name nor status. Story 2 (CAP-1..3): a dim `· program` after the name while a non-shell runs, `✳` for claude, nothing when the name's first word is the program.

## Boundaries & Constraints

**Always:** Program = `pane_current_command` (model.rs `f[17]`), no new tmux field or call, nothing spawned. Shells `zsh bash sh fish` count as none. Hand names verbatim. Row: ` ❯ ` name, then ` · prog` or ` ✳` dim, then ` ` status. The name is cut first; the mark shows only while ` ❯ ` plus min(8, name length) name characters fit beside it and the status, else it is left out whole (user, review loop 1). This is measured against the status as it reads now, so the mark may come and go as the timer's width changes (user, review loop 2). Status is never cut; no row is wider than the pane. CAP-2 compares the program with the basename of the name's first word, for claude too. Files under 100 lines.

**Never:** A glyph or colour per program; a setting to hide it; changing status icons, sidebar width, group rows, the clock; touching the default tmux socket or a real host.

## I/O & Edge-Case Matrix

| Scenario | name, command, `@claude` | Row (width 26) |
|---|---|---|
| CAP-1 renamed | `deploy`, `vim`, - | `deploy · vim` |
| CAP-1 exited | `deploy`, `zsh`, - | `deploy` |
| CAP-1 ssh | `lde@app01 infra`, `ssh`, - | `lde@app01 infra · ssh` |
| Interpreter | `npm run build`, `node`, - | `npm run build · node` |
| CAP-2 | `htop`/`make test`/`/usr/bin/htop`/`sudo vim x`/`ssh`, same program | name only |
| Shell or none | any, `zsh`/`bash`/`sh`/`fish`/empty, - | name only |
| CAP-3 | `Fix login redirect`, `claude`, - | `Fix login redirect ✳` |
| CAP-3 cut | `Fix login redirect`, `claude`, `running` 2m | `Fix login redire ✳ ⚙ 2m` |
| CAP-4 | folder `infra`, `zsh`, `waiting` | `infra` (no `● waits`) |
| CAP-4 named | title `✳ Fix login`, `zsh` idle, `done` | folder·branch, no `✓ took` |
| CAP-4 timed | `ssh` 5 s, stale `running` | `… · ssh ⏱ 5s` |
| No room | `deploy` (current), `ansible-playbook`, ⏱ 1h6m | ` ❯ deploy ⏱ 1h6m` |

</frozen-after-approval>

## Code Map

- `src/model.rs:38` -- `claude: !f[7].is_empty() || f[17] == "claude"`: keep only the second half.
- `src/model.rs:70-93` -- `status`: the `match tab.f[7]` Claude arms must apply only when `f[17] == "claude"`; the timer, `@finished` and `✗` arms below stay as they are.
- `src/model.rs:14-21,41` -- `Tab`: add `pub program: String`, set where `name` is computed.
- `src/name.rs:24` -- comment "an ssh … outranks its stale @claude" is moot once `claude` means the live process; drop it. `:37` inline shell list becomes `shell(f.command)`.
- `src/draw.rs:58-66` -- tab row: drop the mark if ` ❯ ` + min(8, name) chars + mark + gap + status exceed width; then `room` = width minus status, gap and the mark kept; the label alone is cut, trailing space trimmed. `:1-2` module doc.
- `src/main.rs:5-12` -- `mod` list, alphabetical.
- `README.md:6-21,49-51,58` -- example, tab-name and status bullets, Claude hook paragraph, Files `src/` cell.
- `_bmad-output/implementation-artifacts/deferred-work.md` -- DW-6 (status follows stale `@claude`) is CAP-4.
- Read-only: `tabrail-tmux:12` WIDTH=26; `vmConfig/tmux-claude-state:16` unsets `@claude` only on SessionEnd. Interactive claude has argv0 `claude` and stays the tty's foreground pgrp while its tools run (checked with `ps -o pgid,tpgid`).

## Tasks & Acceptance

**Execution:**
- [x] `src/model.rs` -- story 1: live-Claude `claude` fact; `status` ignores `@claude` unless `f[17]` is `claude`.
- [x] `src/name.rs` -- story 1: drop the moot comment; story 2: `pub fn shell(command: &str) -> bool`, used in `label`.
- [x] `src/program.rs` (new) -- `pub fn mark(name: &str, command: &str) -> String`: `""` for empty or shell command or CAP-2 match, `✳` for `claude`, else `· {command}`. `command` is first cut to its basename (tmux keeps a relative argv0 like `./target/release/x` whole).
- [x] `src/model.rs`, `src/main.rs` -- `Tab.program = program::mark(&name, &f[17])`; `mod program;`.
- [x] `src/draw.rs` -- program in `fg("dim")` between label and status, also on the current row; the drop rule above; a cut label loses its trailing space.
- [x] `README.md` -- example rows as they really render at 26 columns; bullets: mark is the foreground program as tmux reports it (`deploy · sudo`), none when the name's first word is it, dropped when there is no room; interpreter example `x.py · python3` (npm retitles itself to `npm`); hook paragraph: state shown while `claude` is the foreground program of the tab's active pane, a stale `@claude` ignored (the options stay set); Files table.
- [x] `deferred-work.md` -- DW-6 `status: resolved` (spec-tab-program CAP-4).
- [x] Scratch `rustc --test` crate (not committed) -- `#[path]` the src modules, feed `model::tabs` FIELDS lines, run `draw::screen` at width 26, strip ANSI; one assert per matrix row, plus `./target/release/x` and a cut ending in a space.
- [x] Scratch harness -- one C stand-in (prints each stdin line as an OSC 2 title, exits on `exit`/EOF) copied as `claude`, `ssh`, `htop`, `ansible-playbook` first in PATH; zsh with oh-my-zsh termsupport and `~/Dev/vmConfig/tmux.zsh`.

**Acceptance Criteria:**
- Given a throwaway tmux server running tabrail, when a tab renamed `deploy` runs vim and quits, then its row reads `deploy · vim`, then `deploy`.
- Given that server, when stand-in `claude` titles `✳ Fix login redirect`, then the row reads `Fix login redirect ✳`; with `@claude` set to `waiting`, `Fix login red ✳ ● waits`; after `exit`, the folder name with no `✳` and no `● waits`.
- Given that server, when stand-in `ssh app01` titles `lde@app01: ~/src/infra`, then the row reads `lde@app01 infra · ssh` (cut once `⏱` appears); `htop` reads `htop`; an idle shell only its folder.
- Given that server, when the current tab renamed `deploy` runs stand-in `ansible-playbook` for 3 s, then it reads ` ❯ deploy ⏱ 3s` and no row is wider than 26 columns.
- Given `cargo build --release --offline`, then no warnings, and every touched file stays under 100 lines.

## Implementation Notes

- Loop 3: loop 2's KEEP and patches carried in unchanged (model.rs, name.rs, program.rs, main.rs, README, DW-6); README status bullet and hook paragraph take R25's wording (foreground program of the tab's active pane) and the mark bullet says it can come and go as a timer grows.
- draw.rs re-planned to 96 lines: the status carries its own leading space, one `len` closure, `least` decides the mark, `right` = mark + status, the label alone is cut to `width - right` and trimmed only when cut.
- Scratch `rows.rs`: loop 2's 18 tests plus `mark_follows_status_width` (`deploy` + `terraform`: shown at `⏱ 9s`, dropped at `⏱ 10s`, back at `⏱ 1m`); 19 pass. Mutants (no `least` check, no trim, `min(7)`, stale `@claude` status, no basename, `shell(name)`, old `claude` fact) each fail at least one test.
- Throwaway tmux (`tp-test` in `tp-outer`): AC1-AC3 pass as written; AC4 needs polling, since the sidebar redraws once a second on whole seconds: the first timer frame reads ` ❯ deploy ⏱ 3s` (3.8 s after Enter), border column intact on every line.

## Spec Change Log

- **Loop 1 (review of the first implementation, intent_gap R5).** Trigger: R5 (program and status are never cut, so ` · ansible-playbook ⏱ 1h6m` wipes ` ❯ ` and the name, and `⏱ 11h6m` makes a 27-column row that wraps). Root cause in the frozen Always ("only the name is cut"); code reverted, decision asked of the human. Known-bad state: a row wider than the pane, or a row without ` ❯ ` and name. KEEP: model.rs `claude: f[17] == "claude"` and `status` matching `if tab.f[17] == "claude" { tab.f[7].as_str() } else { "" }` with its comment; name.rs `pub fn shell` and the dropped "stale @claude" comment; program.rs `mark` shape (first-word basename, shell or empty -> `""`, `✳`, `· cmd`) and its module doc; `Tab.program` beside `name`; draw.rs `prog` in `fg("dim")` after `\x1b[0m`; README hook paragraph, Files cell; DW-6 resolution; scratch `rows.rs` (13 tests, 26-column width assert). Patches to carry in: R4, R8, R10, R12, R18.

- **Loop 2 (review of the re-implementation, intent_gap R31).** Trigger: R31 (the drop rule measures the live status, so the mark toggles as the timer's width changes: `deploy` + `terraform` shows at `⏱ 9s`, drops at `⏱ 10s`, returns at `⏱ 1m`). Root cause in the frozen Always ("fit beside it and the status"); code reverted, decision asked of the human. Known-bad state: a mark that appears and disappears while one program runs. KEEP: everything in loop 1's KEEP; program.rs basename line with its tmux comment (R4); draw.rs `least`/`prog`/`right`/`full` shape and the `trim_end` of a cut label (R10); README mark bullet (`x.py · python3`, `deploy · sudo`), hook paragraph "options stay set, the sidebar ignores them"; scratch `rows.rs` 18 tests incl. `sweep_width_and_name`. draw.rs is at 99 lines: the re-plan must make room. Patches to carry in: R4, R8, R10, R12, R18, R25, R34.

## Review Triage Log

| # | Source | Finding | Verdict | Route | Evidence |
|---|---|---|---|---|---|
| R1 | verification-gap | stale-`@claude` gating (CAP-4) pinned by no test that runs | low | defer | Pre-verified: restoring `match tab.f[7]` builds and passes; only scratch `rows.rs` covers it. Same owner call as DW-3/5/8. |
| R2 | verification-gap | `program::mark` rules pinned by no test that runs | low | defer (with R1) | Pre-verified: dropping `shell(command)` or `.rsplit('/')` still builds. Same root cause as R1. |
| R3 | verification-gap | name-cut math with the program column unpinned | low | defer (with R1) | Pre-verified: leaving `prog` out of `right` builds, row 28 wide. Same root cause as R1. |
| R4 | verification-gap (other) | relative argv0 (`./target/release/x`) is not basenamed by tmux: CAP-2 misses, the mark shows the path | low | patch | tmux `parse_window_name` basenames only a leading `/`; reviewer saw `./target/release/mysleep` on tmux 3.8. Fix: basename `command` in `mark`. Overflow half is R5. |
| R5 | blind, edge-case x2, verification-gap | program and status never cut: ` ❯ ` and name vanish once they fill the row, and the row can exceed the pane and wrap | medium | intent_gap | draw.rs:78-80 `take(width - right)` with `right` unbounded; ` · ansible-playbook ⏱ 1h6m` = 26, `⏱ 11h6m` = 27. Frozen Always says only the name is cut; what yields next has several readings. |
| R6 | blind | Claude in an inactive split loses its status | low | reject | `tabs` reads the active content pane, as SPEC and spec-claude-jump define "the tab's content pane"; one content pane per tab is the norm, and the fix aggregates panes per window. |
| R7 | blind | `== "claude"` in three places, `Facts.claude` redundant | low | reject | Harm only when the rule changes (e.g. R6); fix adds a field or helper. |
| R8 | blind | README "a Claude that crashed leaves no state behind" overclaims; vmConfig tree still shows it | low | patch | `@claude*` stay set, only the sidebar ignores them; reword. The vmConfig `tmux-formats.conf` half is another repo, out of this build. |
| R9 | blind | local vim with a short title shows vim twice (`a.rs (~) - VIM · vim`) | low | reject | Needs a title short enough not to be cut; the fix is DW-7's editor parse in name.rs. |
| R10 | blind | cut names: no `…`, double space when cut at a space | low | patch | `…` rejected (SPEC says cut, nothing more). Double space real and now frequent (`lde@app01  · ssh`); fix: `trim_end` the cut label. |
| R11 | blind | DW entries' line numbers stale | low | reject | Ledger records location at filing; existing entries are not edited. Missing entry for this story is R1. |
| R12 | blind | README states CAP-2 as "starts with it", silent on foreground process (`deploy · sudo`) | low | patch | Code compares the first word's basename; README wording fix. |
| R13 | edge-case | prog + status wider than pane overflows | medium | intent_gap (R5) | Same defect as R5. |
| R14 | edge-case | `right >= width - 2` cuts ` ❯ ` | medium | intent_gap (R5) | Same defect as R5. |
| R15 | edge-case | live Claude whose folder or topic's first word is `claude` gets no `✳` | low | reject | Frozen: CAP-2 applies to claude too; needs a folder named exactly `claude` or a lowercase topic. |
| R16 | edge-case | precommand words (`nice make · make`, `withproxy claude ✳`) | low | reject | Fix adds a wrapper list; same call as tab-rows R8/R20. Claude retitles within a second. |
| R17 | edge-case | Claude with argv0 `…/versions/2.1.311` loses state | false | reject | `ps`: all 20 foreground claude leaders on tmux panes have argv0 `claude`; the `2.1.311` one runs on a `claude bg-pty-host` pty, not a tmux pane. |
| R18 | edge-case | README example `npm run build · node` is wrong | low | patch | npm sets its process title, so argv0 reads `npm run build` and `pane_current_command` is `npm`: CAP-2 hides the mark. Use a true example (`x.py · python3`). |
| R19 | verification-gap (loop 2) | `program::mark` rules pinned by no test that runs | low | carried defer (R2) | carried: same claim as R2; mutants (`shell(name)`, no basename) build and pass 0 tests. |
| R20 | verification-gap (loop 2) | drop rule and trimmed cut pinned by no test that runs | low | carried defer (R3) | carried: same claim as R3 on the reworked cut math. |
| R21 | verification-gap (loop 2) | CAP-4 status and name halves pinned by no test that runs | low | carried defer (R1) | carried: R1 covers both `model.rs:39` and `:79`. |
| R22 | verification-gap (loop 2), blind (loop 2) | ledger has no entry for R1-R3 | low | carried defer (R1) | Loop 1 looped back before defers were processed; the entry is appended when they are. |
| R23 | verification-gap (loop 2), blind (loop 2) | `mark` basenames `command`, `status`/`Facts.claude` compare raw `f[17]`: a `./claude` shows `✳` with no state | low | reject | R17: every tmux-pane Claude has argv0 `claude`; the fix plumbs a basenamed command through model, name and status. |
| R24 | edge-case (loop 2), blind (loop 2) | Claude in an inactive split loses status and `✳` | low | carried reject (R6) | carried: same location and claim as R6. |
| R25 | blind (loop 2) | README "while `claude` runs in the tab" overstates: it is the active pane's foreground program | low | patch | README.md hook paragraph vs `model::tabs` picking the active content pane; wording fix. |
| R26 | edge-case (loop 2) | argv0 other than `claude` (`2.1.311`, npx `node`, `sudo claude`) | low | carried reject (R17) | carried for `2.1.311`; npx and sudo variants unseen here (native install), fix guesses names. |
| R27 | edge-case (loop 2) | double-width characters make a row wider than the pane | low | reject | Predates this story: DW-1 (`chars().count()` widths). |
| R28 | edge-case (loop 2) | dash, ksh, tcsh, nu, pwsh idle shells get a mark | low | reject | SPEC constraint fixes the shell list "as name.rs lists them". |
| R29 | edge-case (loop 2) | empty name with a mark gives a double space | low | reject | Needs an empty window name or an untitled Claude in `/`; trimming an uncut label would move the mark to column 1. |
| R30 | edge-case (loop 2), blind (loop 2) | trim after the cut can leave 7 name characters | low | reject | The trimmed 8th character is a space; nothing readable is lost. |
| R31 | blind (loop 2) | the mark toggles as the timer's width changes | medium | intent_gap | Traced draw.rs `least` with `deploy`+`terraform`: 26 at `⏱ 9s`, 27 at `⏱ 10s`, 26 at `⏱ 1m`; `python3` beside a long name toggles at 10 s, 1 m, 10 m. Frozen Always measures "the status"; a stable measure has several readings. |
| R32 | blind (loop 2) | DW-6 resolution line has no `source_spec` | low | reject | Cosmetic; the resolution names spec-tab-program CAP-4. |
| R33 | blind (loop 2) | SPEC.md assumptions now contradict code and README | low | reject | The deviation is recorded here (frozen loop-1 decision, R18); SPEC.md is edited on the spec side (~/Dev/tabrail), not in this build. |
| R34 | blind (loop 2) | this story file is untracked, so its decisions would not merge | low | patch | Commit it with the story. |
| R35 | blind (loop 2) | AGENTS.md lacks the `pane_current_command` argv0 pitfall (sudo, npm title, relative path) | low | defer | Agent-context file: owner's call, like DW-9. |
| R36 | blind (loop 2) | a status wider than a narrow pane still overflows | low | reject | Predates this story (status never cut); needs a sidebar under ~11 columns, which spec-sidebar-width addresses. |
| R37 | blind (loop 2) | row math inline in `screen()`, draw.rs at 99 lines | low | reject | Refactor; the re-plan accounts for draw.rs's line budget. |
| R38 | verification-gap (loop 3) | CAP-4 gating pinned by no test that runs | low | carried defer (R1) | carried: same claim; mutant m1 builds, scratch `cap4*` fail. |
| R39 | verification-gap (loop 3) | `program::mark` rules pinned by no test that runs | low | carried defer (R2) | carried: same claim; mutant m2 (`shell(name)`). |
| R40 | verification-gap (loop 3) | drop rule, cut and trim pinned by no test that runs | low | carried defer (R3) | carried: same claim; mutants m3, m4. |
| R41 | edge-case (loop 3) | argv0 other than `claude` (`2.1.311`, npx, wrapper) | low | carried reject (R26) | carried. |
| R42 | edge-case (loop 3) | `./claude`: `mark` basenames, model compares raw `f[17]` | low | carried reject (R23) | carried. |
| R43 | edge-case (loop 3) | Claude tab whose name's first word is `claude` loses `✳` | low | carried reject (R15) | carried. |
| R44 | edge-case (loop 3) | wrapper words (`withproxy cargo build · cargo`) | low | carried reject (R16) | carried. |
| R45 | edge-case (loop 3) | dash, ksh, nu, pwsh idle shells | low | carried reject (R28) | carried. |
| R46 | edge-case (loop 3) | trim can leave 7 name characters | low | carried reject (R30) | carried. |
| R47 | edge-case (loop 3) | wide characters overflow the row | low | carried reject (R27) | carried. |
| R48 | edge-case (loop 3) | status wider than a narrowed pane | low | carried reject (R36) | carried. |
| R49 | edge-case (loop 3), blind (loop 3) | a crashed session's `@claude` shows again while a new `claude` runs before SessionStart (`claude update`, `doctor`, `mcp list`, resume picker) | low | patch (README) / reject (code) | Real: the gate asks only whether claude runs. Transient and only after a crash; the code fix compares `@claude_since` with the command start, a new guard. README "the sidebar ignores them" overclaims: reword. |
| R50 | blind (loop 3) | a suspended Claude reads `✗ 148` | low | defer | zsh reports 148 after Ctrl-Z, `f[17]` is `zsh`, so the fail arm shows; any suspended program (vim) already does this. Predates the story; CAP-4 makes Claude behave like the rest. |
| R51 | blind (loop 3) | ledger lacks the deferred entries | low | carried defer (R22) | carried: appended in this pass. |
| R52 | blind (loop 3) | story file untracked | low | carried patch (R34) | carried: committed with the story. |
| R53 | blind (loop 3) | a shell script (argv0 `bash`) gets no mark, README says "idle shell" | low | patch | Frozen Always counts shells as no program; README wording: nothing while a shell is the foreground program (idle prompt or shell script). |
| R54 | blind (loop 3) | README example `/usr/bin/htop` renders `usr/bin/htop` from a title | low | patch | `name::label` strips leading non-alphanumerics from titles; drop the example from README. |
| R55 | blind (loop 3) | `Facts.claude` duplicates `command` | low | carried reject (R7) | carried. |
| R56 | blind (loop 3) | drop rule hard-codes the ` ❯ ` prefix width and the mark's space | low | reject | The prefix is fixed by tab-rows' row layout; deriving it reshapes the row math for a change nobody plans. |

## Design Notes

At 26 columns the mark costs the name its room: `lde@app01 infra · ssh ⏱ 4m` renders `lde@app01 in · ssh ⏱ 4m`, and a Claude topic loses 2 columns to ` ✳`. That is SPEC's "a long name is cut first"; once 8 name characters are all that is left, the mark goes instead. The README example shows real output, not the uncut fragments of SPEC's success lines.

`program` is computed in `model::tabs` beside `name`, so draw.rs only lays it out. CAP-2 applies to claude as well: a name `claude --continue` (oh-my-zsh's title before Claude sets its own) gets no `✳`.

## Verification

**Commands:**
- `cargo build --release --offline` -- expected: no warnings.
- `rustc --edition 2021 --test <scratch>/rows.rs -o <scratch>/rows && <scratch>/rows` -- expected: all matrix rows pass.
- `tmux -L outer capture-pane -p` with `tmux -L <test>` attached inside -- expected: the ACs.

**Manual checks (if no CLI):**
- Kill both test servers, their sidebars and menu files in `$XDG_RUNTIME_DIR/tabrail`; never touch the default socket.
