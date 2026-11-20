---
title: 'Pick a group colour from its menu'
type: 'feature'
created: '2026-11-20'
status: 'ready-for-dev'
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
- [ ] `palette` -- new, the 25 lines from Design Notes as `#rrggbb Name` -- one list for picker and auto-pick
- [ ] `tabrail-act` -- `new_group` takes the first palette hex not used by any group line; drop `TINTS` -- one palette
- [ ] `tabrail-colour` -- new: `tabrail-colour GROUP CLIENT` builds the swatch list and opens the popup (height `min(rows + 5, client_height)`); `--pick GROUP LIST` runs `tabrail-pick "Colour for GROUP?"`, writes column 2 safely, refreshes -- the picker and the write
- [ ] `src/menu.rs`, `src/main.rs` -- `Place.colourer`; group menu item `'Colour…' c` after Rename group, not for Default, name through `clean()` -- the entry point
- [ ] `README.md`, `groups.conf.example` -- Colour… in the groups and mouse text, `tabrail-colour` and `palette` in Files -- README is the behaviour reference

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
