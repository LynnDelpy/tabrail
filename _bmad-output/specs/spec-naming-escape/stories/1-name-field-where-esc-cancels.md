---
title: 'Name field where Esc cancels'
type: 'feature'
created: '2026-11-19'
status: 'done'
route: 'dispatch'
baseline_commit: '30453185c287bc30e88a44f2fc55fe8f16ea225a'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/specs/spec-naming-escape/SPEC.md'
  - '{project-root}/_bmad-output/specs/spec-naming-escape/brownfield.md'
  - '{project-root}/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Rename, Rename group and both New group… items take names through tmux `command-prompt`. With `EDITOR=/usr/bin/vim` tmux defaults `status-keys` to vi, so Escape only enters normal mode and only `q` or `C-c` cancel.

**Approach:** The four items open a popup name field in `tabrail-pick`'s style (fzf, like Move to group): one Esc closes it with nothing changed, Enter saves through today's commands (`rename-window`, `tabrail-act rename-group`, `tabrail-act new-group`).

## Boundaries & Constraints

**Always:** Esc cancels under vi and emacs `status-keys`. The field opens with the tab's shown name (Rename), the group's name (Rename group), empty (New group…). Titles `Rename tab`, `Rename group`, `New group`; footer `Enter to save · Esc to cancel`. Enter on an empty or blank field changes nothing. Name rules stay: `menu::clean()` for names in tmux strings; `tabrail-act` turns spaces into `-` and refuses `Default`. After the popup closes the keyboard is in the current tab's content pane. The typed name reaches tmux and `tabrail-act` as an argument, never inside a tmux or shell command string. Files under 100 lines.

**Never:** Set `status-keys` or change tmux's own prompts (prefix `,`, `:`). Touch Close's `confirm-before` (SPEC open question), Move to group, or `tabrail-act`'s interface. Add a dependency beyond tmux, bash, fzf. Test on the default tmux socket.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Rename, Enter | tab shown as `website·release-2`; field holds that; replace with `deploy`, Enter | tab named `deploy`, `automatic-rename` off | N/A |
| Rename group, Enter | group `Notes`, field `Notes`; replace with `My notes`, Enter | `My-notes` in groups.conf, on its tabs' `@group`, in `@sidebar_fold` | N/A |
| New group from a tab | type `Ops`, Enter | groups.conf gains `Ops` with an unused tint and icon; that tab's `@group` is `Ops` | N/A |
| New group from a group or empty row | type `Ops2`, Enter | a new tab in group `Ops2`, in the sidebar's session | N/A |
| Any of the four, Esc | text typed, one Esc | popup gone; groups.conf, every tab's name, `automatic-rename`, `@group`, tab count as before | N/A |
| Empty or blank | `` or `   `, Enter | popup gone, nothing changes | N/A |
| `Default` | New group or Rename group to `Default` | nothing changes | `tabrail-act` refuses, as today |

</frozen-after-approval>

## Code Map

- `src/menu.rs:28-30` -- `new_group` closure: `command-prompt` → `tabrail-name new`. Keep `%`→`p` (it is `tabrail-act new-group`'s TARGET form); rewrite its comment, the `%1` reason is gone.
- `src/menu.rs:38`, `:50-51` -- Rename and Rename group items. `:40` Move to group is the pattern (`run-shell -b \"{mover} {id} '#{{client_name}}'\"`). `:43` Close stays.
- `src/menu.rs:11-15` -- `Place`: add `namer` (tabrail-name). `src/main.rs:26-27,39` -- build it like `mover`.
- `src/main.rs:70-74` -- read-only: the right press runs `last-pane`, so the content pane is active when the popup opens; a popup keeps the active pane.
- `tabrail-pick:7-22` -- `opts` holds the look; add the field mode next to `-n`.
- `tabrail-move:36` -- popup precedent. `tabrail-act:40-66` -- unchanged, called with the typed name.
- `README.md:56-63` -- Files table.

## Tasks & Acceptance

**Execution:**
- [x] `tabrail-pick` -- add `-q TEXT`: a name field instead of a list (`--disabled --query TEXT --prompt '❯ ' --bind enter:print-query`, stdin `/dev/null`, the footer above) on the shared `opts`; prints the text on Enter, nothing on Esc -- one place for the look.
- [x] `tabrail-name` (new, executable) -- `tabrail-name KIND TARGET CLIENT [TEXT]`, KIND `tab`, `group` or `new`: opens `display-popup -c CLIENT -B -E -w 60 -h 5` running itself `--ask KIND TARGET TEXT` as separate arguments (no shell string). `--ask` runs `tabrail-pick -q`, exits on an empty or blank answer, else `tmux rename-window -t TARGET -- NAME`, `tabrail-act rename-group TARGET NAME` or `tabrail-act new-group NAME TARGET`.
- [x] `src/menu.rs`, `src/main.rs` -- `Place.namer`; the three items become `run-shell -b \"{namer} KIND TARGET '#{{client_name}}' 'TEXT'\"` with cleaned names.
- [x] `README.md` -- Files row for `tabrail-name`; `tabrail-pick` row mentions the name field.
- [x] Scratch harness (scratchpad) -- throwaway servers per AGENTS.md, groups file via `@tabrail_groups`.

**Acceptance Criteria:**
- Given a throwaway server started with `EDITOR=/usr/bin/vim` (`show -gv status-keys` is `vi`) running tabrail, the viewer `tmux -L outer -f /dev/null`, when each of the four items is opened by a real right-click and one `Escape` is sent through the viewer after typing, then the popup is gone and a snapshot (groups.conf checksum; `list-windows -a -F '#{window_id} #{window_name} #{automatic-rename} #{@group}'`; `@sidebar_fold`) equals the one before.
- Given that server, when Enter follows typing, then each item has its matrix effect.
- Given either way out, when the popup closes, then the current window's active pane is not the sidebar (`#{@sidebar}` empty).

## Implementation Notes

- Files: `tabrail-pick` (+`-q`), `tabrail-name` (new, 29 lines), `src/menu.rs` (`namer` closure shared by the three items), `src/main.rs`, `README.md`. `tabrail-act`, `tabrail.tmux` untouched.
- Harness (scratchpad, `-L nesc` started with `EDITOR=/usr/bin/vim`, viewer `-L nescout -f /dev/null`, SGR right-clicks): Esc on all five menu entries passes under `status-keys` vi and emacs, snapshot unchanged, keyboard in the content pane; empty, blank and `Default` change nothing; Enter effects match the matrix (`deploy` with automatic-rename 0; `My-notes` in groups.conf, `@group`, fold; `Ops` with an unused tint and icon; new tabs in `Ops2`, `Ops3`); the field starts with the full tab name, the group name, or empty.
- Typed names with quotes, `$(…)`, backticks, `;`, `\`, a leading `-`, `%1 %%` are stored as a direct `rename-window -- NAME` stores them; no shell injection. tmux itself expands `#{…}` in window names, as with the old prompt.
- Review loop 1 patches: TARGET single-quoted in the `namer` string (a group `a&b|c` now reaches tabrail-name whole), README Setup names fzf for the name fields, harness Enter effects asserted. Rejected and deferred findings in the Review Triage Log; DW-8, DW-9.

## Spec Change Log

## Review Triage Log

| # | Source | Finding | Verdict | Route | Evidence |
|---|---|---|---|---|---|
| R1 | blind | `namer` leaves TARGET unquoted in the run-shell string | low | patch | New group writes `a&b` as typed; its Rename group item then runs `tabrail-name group a &` plus `b …` (reproduced with `sh -c` by verification-gap). Fix `'{target}'`; `clean()` drops `'`. |
| R2 | blind | typed names with `clean()`'s characters do not round-trip through the menus (`C#` reaches rename-group as `C`) | low | reject | Real; the old prompt let the same names in, they are rare, and the fix adds an input filter. |
| R3 | blind | Enter on the untouched field pins the tab's automatic name | low | reject | Today's effect: the old `command-prompt -I '{name}' … rename-window '%%'` did the same, and CAP-2 keeps today's effects. |
| R4 | blind | `FZF_DEFAULT_OPTS` could rebind Esc or break the 5-row popup | low | reject | The one user sets neither; the pickers share the exposure, so a guard is a tabrail-pick-wide call. |
| R5 | blind | naming now needs fzf, README Setup says "for the pickers"; fzf failure is silent | low | patch | README.md:29 wording. The exit-status check is rejected: fzf is installed and the flags are the pickers' own. |
| R6 | blind | errors after the field are lost (target gone while the popup is open) | low | reject | Needs the tab closed while its popup covers the client; fix adds a guard. |
| R7 | blind | no usage line or check for an unknown KIND | low | reject | Only menu.rs calls it, with fixed kinds; tabrail-move has no usage check either. |
| R8 | blind | surrounding whitespace not trimmed (` Ops` → `-Ops`) | low | reject | Same with the old prompt; fix adds trimming. |
| R9 | blind | TEXT redundant for `group` and `new` | false | reject | TEXT is cleaned and quoted; no harm named, one interface for all kinds. |
| R10 | blind | no committed test pins the menu strings | low | defer (DW-8) | Repo has no tests; same owner call as DW-3/DW-5. |
| R11 | blind | AGENTS.md pitfalls miss `status-keys` from EDITOR and popup argv | low | defer (DW-9) | Fix edits an agent-context file. |
| R12 | verification-gap | Enter effects printed, not asserted, in the harness | low | patch | Pre-verified: dropping `%`→`p` leaves every printed line PASS. |
| R13 | verification-gap (other) | TARGET unquoted | low | patch (R1) | Same defect as R1. |
| R14 | verification-gap (other) | `awk -v` turns `\n` in a typed group name into a newline in groups.conf | low | reject | Pre-existing in tabrail-act:43,49; the old prompt delivered the same text; rare; fix (ENVIRON) is outside the story. |
| R15 | edge-case | popup wider than a client under 60 columns does not open | low | reject | Real; with a 26-column sidebar such clients are rare, tabrail-move has the same `-w 60`, fix adds a guard. |
| R16 | edge-case | untouched Enter turns automatic-rename off | low | reject | Same as R3. |
| R17 | edge-case | untouched Enter stores the `clean()`ed name | low | reject | Same as R3: the old `-I` held the cleaned name too. |
| R18 | edge-case | a typed name ending in `;` loses it (tmux argv separator) | low | reject | Real; rare; fix adds escaping. |
| R19 | edge-case | group names with `clean()`'s characters or `\|` | low | reject | Same as R2; `\|` splits the fold state with the old prompt too. |
| R20 | edge-case | new group starting with `#` becomes a comment line | low | reject | Pre-existing in tabrail-act; rare; fix adds a guard. |
| R21 | edge-case | backslash escapes in group names | low | reject | Same as R14. |
| R22 | edge-case | whitespace not trimmed | low | reject | Same as R8. |
| R23 | edge-case | TARGET and install path unquoted in menu.rs | low | patch (R1) | TARGET as R1; the path is unquoted like helper and mover since before. |
| R24 | edge-case | `FZF_DEFAULT_OPTS` with `--exit-0` closes the field | low | reject | Same as R4. |
| R25 | edge-case | unknown KIND or too few arguments | low | reject | Same as R7. |
| R26 | edge-case | target closed while the popup is open | low | reject | Same as R6. |

## Design Notes

fzf, not `read -e`: readline follows the inputrc (vi mode repeats the fault); fzf aborts on Esc whatever tmux's keys are. Prototyped (fzf 0.74.5): `-h 5` fits border, field, rule, footer; Enter prints the query (exit 0), Esc nothing (130).

Arguments, not a string: tmux 3.8 `display-popup` takes `shell-command [argument ...]`; bash `printf %q` gives `$'…'` under a C locale, which sh cannot read.

## Verification

**Commands:**
- `cargo build --release --offline` -- expected: no warnings.
- `bash -n tabrail-name tabrail-pick` -- expected: clean.
- `tmux -L outer capture-pane -p` with the inner server attached -- expected: the ACs.

**Manual checks (if no CLI):**
- Kill both test servers and their sidebars, remove their menu files in `$XDG_RUNTIME_DIR/tabrail`; never touch the default socket.
