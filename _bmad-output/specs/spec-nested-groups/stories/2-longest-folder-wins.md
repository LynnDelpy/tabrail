---
title: 'Longest folder wins'
type: 'feature'
created: '2026-11-23'
status: 'ready-for-dev'
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
- [ ] `tabrail-tmux` -- `group()` keeps the longest matching folder, strict `>` so ties stay with the first line -- CAP-3
- [ ] `tabrail-groups` -- `folder GROUP`: first folder of GROUP, else of its nearest ancestor, else nothing -- one tree reader on the bash side
- [ ] `tabrail-act` -- `new()` uses `tabrail-groups folder`, `~` when empty -- folderless subgroups open where their parent works
- [ ] `tabrail-tree-test` -- the matrix rows, calling `tabrail-tmux group` with `TMUX` set to the test server -- regression
- [ ] `README.md`, `groups.conf.example` -- longest folder wins; line order is display order only -- README is the behaviour reference

**Acceptance Criteria:**
- Given egov and St.Gallen as above on a test server, when `new-window -c ~/x/egov/stgallen/src` runs, then that tab's `@group` is St.Gallen.

## Verification

**Commands:**
- `bash -n tabrail-tmux tabrail-groups tabrail-act && wc -l tabrail-tmux tabrail-groups tabrail-act` -- expected: syntax ok, each under 100
- `./tabrail-tree-test` -- expected: every line `ok`
