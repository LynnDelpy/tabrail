---
title: 'Pick a group colour from its menu'
type: 'feature'
created: '2026-11-20'
status: 'done'
baseline_commit: '0aa0a52a4e7ce14a5370d44a785d85408262fa23'
route: 'dispatch'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/specs/spec-group-colour/SPEC.md'
  - '{project-root}/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** A group's tint (column 2 of its line in the groups file) is set once by New group… and otherwise only by hand-editing the file.

**Approach:** The group menu gets Colour…, which opens a searchable `tabrail-pick` popup of 25 named swatches (one palette file, also used by New group…'s auto-pick); the choice is written into column 2 safely and every sidebar redraws.

## Boundaries & Constraints

**Always:** Only column 2 of the chosen group's line changes; comments and every other line stay byte for byte, the changed line keeps its spacing. Write like `move_group`: through the symlink (`readlink -f`), mode kept (`chmod --reference`), temp file renamed into place, nothing written unless exactly one line differs. Redraw with `tmux set-buffer -b sidebar-refresh x`. Each swatch row shows the group's own icon and name in bold `#cdd6f4` on the tint. Files under 100 lines.

**Never:** Colour… for Default. Custom hex input. Colour on New group… or in the tab menu. Changing draw.rs colouring (tab rows, icon, text). New dependencies.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Pick | Colour… on Website, type `teal d`, Enter | Website's column 2 becomes `#415960`; all sidebars show it | N/A |
| Cancel | Esc in the picker | File identical (`cmp`) | N/A |
| Current | Website's tint `#283147` | `Blue` row marked `· current` | N/A |
| Hand-written tint | Tint not in palette | No row marked; picking still works | N/A |
| Group gone | Group renamed or removed while the picker is open | File untouched | awk finds no line: temp removed, exit 0 |
| Small terminal | 25 rows, client 20 rows high | Popup no taller than the client; fzf scrolls | N/A |

</frozen-after-approval>

## Code Map

- `tabrail-act` -- 98 lines. `TINTS` array (line 15) and `new_group` (auto-pick of first unused tint) move to reading `palette`; `move_group` is the safe-write pattern to copy.
- `tabrail-move` -- the model for the new script: two modes (`WINDOW CLIENT` opens the popup, `--pick` runs the picker), list in a mktemp file, `display-popup -c CLIENT -B -E -w 60 -h ROWS+4`.
- `tabrail-pick` -- search mode (no `-n`) filters by label as you type; labels may carry ANSI; prints the value column.
- `src/menu.rs` -- `Place` (paths of helpers), `Some(Row::Group(name))` branch builds the group menu; Rename group is already skipped for Default; `clean()` for names in tmux strings.
- `src/main.rs:29-42` -- builds helper paths with `dir.join(...)` and the `Place`.
- `src/model.rs:50` `groups()` -- reads column 2 as the tint; unchanged. `src/draw.rs` -- paints it; unchanged.
- `README.md` -- groups bullet, mouse bullet (menu list), Files table. `groups.conf.example` -- header comment describes the tint column.
- Merge note: the agent-journal build also edits `src/menu.rs`; branch from main after it merges.

## Tasks & Acceptance

**Execution:**
- [x] `palette` -- new, the 25 lines from Design Notes as `#rrggbb Name` -- one list for picker and auto-pick
- [x] `tabrail-act` -- `new_group` takes the first palette hex not used by any group line; drop `TINTS` -- one palette
- [x] `tabrail-colour` -- new: `tabrail-colour GROUP CLIENT` builds the swatch list and opens the popup (height `min(rows + 5, client_height)`); `--pick GROUP LIST` runs `tabrail-pick "Colour for GROUP?"`, writes column 2 safely, refreshes -- the picker and the write
- [x] `src/menu.rs`, `src/main.rs` -- `Place.colourer`; group menu item `'Colour…' c` after Rename group, not for Default, name through `clean()` -- the entry point
- [x] `README.md`, `groups.conf.example` -- Colour… in the groups and mouse text, `tabrail-colour` and `palette` in Files -- README is the behaviour reference

**Acceptance Criteria:**
- Given two groups with tabs on a `tmux -L test` server, when New group… adds a third, then it gets the first palette colour no group uses.
- Given the groups file is a symlink, when a colour is picked, then the link is still a link and the target's mode is unchanged.
- Given a right-click on Default's row, then its menu has no Colour… entry.

## Design Notes

Palette, in this order (existing 7 first so current groups get marked and auto-pick starts there; Catppuccin Mocha accents blended onto `#1e1e2e`, near-duplicates dropped, every one at least 4.7:1 against `#cdd6f4`):

```
#283147 Blue          #3b2f2a Brown         #2a3b2f Green        #3b2a3a Plum
#2a383b Teal          #3b392a Olive         #2f2a3b Violet       #403d4a Rosewater faint
#5e5862 Rosewater deep #5e4f66 Pink deep    #3a344e Mauve faint  #52476a Mauve deep
#402f42 Red faint     #5e3f53 Red deep      #41363c Peach faint  #604b49 Peach deep
#605955 Yellow deep   #343e40 Green faint   #475950 Green deep   #313d49 Teal faint
#415960 Teal deep     #385167 Sapphire deep #2f364f Blue faint   #3e4b6b Blue deep
#4b4e6c Lavender deep
```

Swatch label: `\e[48;2;R;G;Bm\e[1;38;2;205;214;244m ICON NAME \e[0m  Colour name` plus dim `· current`.

## Verification

**Commands:**
- `cargo build --release --offline` -- expected: builds, no warnings added
- `bash -n tabrail-colour tabrail-act && wc -l tabrail-colour tabrail-act src/menu.rs` -- expected: syntax ok, each under 100
- On `tmux -L test` with `@tabrail_groups` pointing at a temp copy (AGENTS.md): SGR right-click a group row, Colour…, type `teal d`, Enter -- expected: `capture-pane -e` of every sidebar shows `48;2;65;89;96` on that row; `diff` against the copy shows only column 2 of that line
- Same, ending in Esc -- expected: `cmp` identical

## Review Triage Log

| # | Source | Finding | Verdict | Evidence | Route |
|---|--------|---------|---------|----------|-------|
| 1 | verification-gap 1-4, blind 3 | No repo test runs `tabrail-colour`'s write, Colour… presence/absence, New group…'s palette auto-pick or the `--nth=-1` search scope | medium | Pre-verified: no `tabrail-*-test` or `tests.rs` names colour/palette/new-group; dropping `readlink -f`, `chmod`, the empty-pick exit or `--nth` fails nothing | patch |
| 2 | edge-case 1, blind 4 | Colour… offered for a group whose name `clean()` changes (`Q'a`): picker edits nothing, or recolours a group `Qa` | low | `clean()` drops `' " { } ; \ $ #` (menu.rs:49); awk then matches no line or the other group; the timer items already guard with `name == *raw` | patch |
| 3 | edge-case 2 | Picker opens for a group with no line: blank icon, Enter silently no-op | false | Matrix row "Group gone" prescribes file untouched, exit 0; the cleaned-name trigger is #2 | reject |
| 4 | edge-case 3, blind 5a | Popup width fixed at 60: on a client under 60 columns `display-popup` fails "width too large" | low | Blind reproduced on tmux 3.8; height is clamped to the client, width is not | patch |
| 5 | blind 5b | A group name over ~26 chars pushes the colour name and `· current` off the 60-column row | low | Row is name+30 columns; group names are one word, such names are rare and the fix (truncating) adds logic | reject |
| 6 | edge-case 4, 12 | No lock against tabrail-act's group-file writers: a concurrent New group… or rename could be lost | low | Needs two writes within the same ms from one user; tabrail-act's own writers do not lock each other either; fix adds cross-script locking | reject |
| 7 | edge-case 5, 11 (exhausted) | With all 25 palette colours used, New group… reuses the last | low | Same as the old 7-tint loop; SPEC Assumptions allow shared colours; needs 25+ groups | reject |
| 8 | edge-case 6, 11 (case), blind 6a | `new_group`'s used check is case-sensitive while the picker's current mark is not: a hand-written `#3B2F2A` counts as unused | low | `grep -qF` at tabrail-act:81 vs `${hex,,}` in tabrail-colour; one-flag fix on a line this change rewrote | patch |
| 9 | edge-case 7, blind 6b | Missing `palette` leaves the tint empty, New group… writes a 2-field line model.rs drops | low | `palette` is tracked beside the scripts; only a broken checkout triggers it; fix is a guard | reject |
| 10 | edge-case 8 | `palette` without a final newline drops its last colour from the picker | low | The tracked file ends with a newline (diff shows none missing); only a hand-edit triggers it; fix is a guard | reject |
| 11 | edge-case 9, blind 6c | Blank, comment or tab-separated `palette` lines break printf or the auto-pick | low | Tracked file with one fixed format; fix is a guard | reject |
| 12 | edge-case 10 | A hand-written tint cannot be picked back after changing it | false | Never: custom hex input; matrix row "Hand-written tint" prescribes no mark | reject |
| 13 | blind 1a | Typing `te` also lists Rosewater faint and deep, so SPEC CAP-3's "only the teal rows" does not hold | low | `fzf --filter=te` on the built list: Teal, Teal deep, Teal faint, Rosewater deep, Rosewater faint (teals first); fuzzy fzf is a SPEC constraint, fixes are a SPEC edit or replacing fzf's matching | reject |
| 14 | blind 1b | Typing `cur` filters to the current row | false | Harmless: `· current` is part of the searched label; no wrong outcome | reject |
| 15 | blind 2 | Near-duplicate palette colours (Plum/Red faint ΔE 3.0, Blue/Blue faint 3.2, Teal/Green faint 3.4) | low | The palette is the spec's Design Notes list; fix is a spec edit | reject |
| 16 | blind 7 | The popup's temp list file in /tmp leaks if the popup is killed mid-pick | low | Only on kill-server or detach mid-pick, ~2 KB; same pattern as tabrail-move; fix restructures the script | reject |
| 17 | blind 8, verification-gap other | A groups file without a final newline gets one on write | low | Same as move_group, the prescribed model; tabrail-act's appends and editors end files with a newline; fix adds awk branches | reject |
| 18 | blind 9 | README Files table not updated for tabrail-pick's search mode | false | The Files table gives each file's job, not its flags (no `-n` either); the search contract sits in tabrail-pick's header with the other modes | reject |
