---
title: 'Longest folder wins'
type: 'feature'
created: '2026-11-23'
status: 'done'
baseline_commit: 'e51331c4024e102ab98a2806b2e7f8ef4f31144a'
route: 'dispatch'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/specs/spec-nested-groups/SPEC.md'
  - '{project-root}/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** A new tab joins the first group in line order whose folder holds it. A parent's line comes before its subgroups' lines, so a subgroup inside the parent's folder never gets new tabs.

**Approach:** The group whose folder is the longest one holding the tab's folder wins, at any level, with ties going to the first line. A New tab in a group without folders opens in its nearest ancestor's first folder, else home.

## Boundaries & Constraints

**Always:** Folders compare after `~` expansion with symlinks not resolved; a folder holds a dir when `dir == folder` or `dir == folder/*`, as today. Line order keeps setting the display order. Only tabs without `@group` get one, as today.

**Never:** Change a tab's group once set. Match Default by folder. Resolve symlinks.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Subgroup deeper | egov `~/x/egov`, `  St.Gallen` `~/x/egov/stgallen` | `group ~/x/egov/stgallen/src` → St.Gallen | N/A |
| Parent | same | `group ~/x/egov/core` → egov | N/A |
| Line order | St.Gallen's line above egov's | same two results | N/A |
| Tie | two groups at `~/x` | first line | N/A |
| Name prefix | folder `~/x/egov`, dir `~/x/egov2` | not egov; Default if nothing else | N/A |
| No folders | Archiv none, parent St.Gallen `~/x/egov/stgallen` | New tab in Archiv opens in `~/x/egov/stgallen` | N/A |
| No ancestor folder | top-level group without folders | opens in `$HOME`, as today | N/A |

</frozen-after-approval>

## Code Map

- `tabrail-tmux:22` `group()` -- `read -r name tint icon dirs` (leading whitespace stripped by read), returns on the first match; becomes a longest-match loop. 82 lines.
- `tabrail-act` `new()` -- `awk ... print $4` takes the group's first folder; needs the parent rule to walk up. The tree logic lives in `tabrail-groups` (story 1): add a `folder GROUP` command there that prints the group's or nearest ancestor's first folder, and call it from `new()`.
- `README.md` groups bullet: "which is also the order a new tab's folder is matched in"; `groups.conf.example` comment: "A new tab joins the first group one of whose folders...".

## Tasks & Acceptance

**Execution:**
- [x] `tabrail-tmux` -- `group()` keeps the longest matching folder, strict `>` so ties stay with the first line -- CAP-3
- [x] `tabrail-groups` -- `folder GROUP`: first folder of GROUP, else of its nearest ancestor, else nothing -- one tree reader on the bash side
- [x] `tabrail-act` -- `new()` uses `tabrail-groups folder`, `~` when empty -- folderless subgroups open where their parent works
- [x] `tabrail-tree-test` -- the matrix rows, calling `tabrail-tmux group` with `TMUX` set to the test server -- regression
- [x] `README.md`, `groups.conf.example` -- longest folder wins; line order is display order only -- README is the behaviour reference

**Acceptance Criteria:**
- Given egov and St.Gallen as above on a test server, when `new-window -c ~/x/egov/stgallen/src` runs, then that tab's `@group` is St.Gallen.

## Review Triage Log

| # | Layer | Location | Claim | Verdict | Route | Evidence |
|---|-------|----------|-------|---------|-------|----------|
| 1 | verification-gap | `tabrail-groups:29-33` | New tab in a group with its own folder is unchecked | medium | patch | Pre-verified: a mutant skipping the group's own line passes `nw Archiv`/`nw Notes` |
| 2 | verification-gap, blind | `tabrail-groups:32` | The ancestor walk is checked one level up only | medium | patch | Pre-verified: `while`→`if` passes; a folderless parent under a foldered grandparent would open in home |
| 3 | verification-gap, blind, edge-case | `tabrail-tmux:29` | The Default-line skip in group() is not pinned | low | patch | Pre-verified mutant; Never: match Default by folder. Dropping the skip (edge-case's fix) would break that rule; `folder Default` serves New tab, not matching, so it keeps it |
| 4 | blind | `tabrail-tree-test` | `nw` checks the cwd, not the new tab's `@group` | low | patch | `new()` exists to set the group; one assertion |
| 5 | blind, edge-case | `tabrail-act:14-17`, `tabrail-tmux ensure` | ensure can overwrite the `@group` new() sets, filing the tab under the parent | low | defer | Race predates this story (then Default); ensure runs after pgrep and flock, so new()'s `set -w` nearly always wins |
| 6 | blind | README nested bullet | "nearest ancestor's first folder" ambiguous when that ancestor has none | low | patch | Code skips folderless ancestors; direct text fix |
| 7 | blind | README, `groups.conf.example` | Should say folders must be real paths: pane_current_path is resolved | low | reject | Same as before this story; the spec keeps symlinks unresolved |
| 8 | blind, edge-case | `tabrail-tmux:30-32` | A trailing-slash folder never matches | low | defer | Same test before this story |
| 9 | edge-case | `tabrail-tmux:28` | A last line without a newline is skipped by `read` | low | defer | Same loop before this story; new-group writes `\n` |
| 10 | blind | `tabrail-tree-test` | Tie checked in one order; a subgroup sharing its parent's folder never gets tabs | false | reject | Frozen Approach: ties go to the first line; one order catches `>=` |
| 11 | blind | `tabrail-tree-test` | At 99 lines; folder checks belong in their own test file | low | patch | Rows 1-4 do not fit under 100, and story 4 adds drag checks there |
| 12 | blind | groups docs | Names must be unique across the tree; duplicates draw twice | low | reject | Hand-written duplicates behaved so before; story 1 row 2 |
| 13 | blind | `tabrail-tree-test` `nw` | `main:$` picks the wrong tab if a window index gap appears | low | reject | No window is killed before it; a future edit's concern |
| 14 | blind | `tabrail-act` header, `groups.conf.example` | Three comments read badly | low | patch | Direct text corrections |

## Verification

**Commands:**
- `bash -n tabrail-tmux tabrail-groups tabrail-act && wc -l tabrail-tmux tabrail-groups tabrail-act` -- expected: syntax ok, each under 100
- `./tabrail-tree-test` -- expected: every line `ok`
