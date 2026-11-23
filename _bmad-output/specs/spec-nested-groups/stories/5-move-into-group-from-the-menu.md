---
title: 'Move into group from the menu'
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

**Problem:** A group cannot be taken out of its parent with the mouse at all, and nesting needs an aimed drag.

**Approach:** A group's menu gets Move into group…, a picker popup like Move to group…. It lists Top level first, then every group except this one and its subtree, in tree order, indented, with the current parent marked `· current`. A pick runs story 4's nest, and Top level moves the tree to the end of the top level, above Default.

## Boundaries & Constraints

**Always:** Esc changes nothing. Picking the current parent makes the group its last subgroup; Top level for a top-level group moves it to the end of the top level. The groups file is written by `tabrail-groups nest`, nothing else. The popup is no taller than the client; fzf scrolls the rest.

**Never:** The item on Default. Offering Default, the group itself or its descendants.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Nest | Köniz → St.Gallen | Köniz's tree is St.Gallen's last subgroup | N/A |
| Out | St.Gallen → Top level | St.Gallen's tree at depth 0 after the last top-level tree | N/A |
| List | menu on egov | Top level, 4tw, …; not egov, St.Gallen, Archiv, Default | N/A |
| Current | menu on St.Gallen | egov's row `· current` | N/A |
| Cancel | Esc | `cmp` identical | N/A |
| Gone | group renamed while the picker is open | file identical | nest finds no line, exit 0 |

</frozen-after-approval>

## Code Map

- `tabrail-move` -- the model: `WINDOW CLIENT` builds a list in a mktemp file and opens `display-popup -c CLIENT -B -E -w 60 -h ROWS+4`; `--pick` runs `tabrail-pick` and acts. Its indented tree list (story 1) can be reused.
- `tabrail-pick` -- prints the value column; an empty value reads as Esc, so Top level needs a value no group name can be (e.g. `/`).
- `src/menu.rs` group branch and `Place` -- add the new script's path; `src/main.rs:29-31` builds the paths.
- `tabrail-groups nest SRC PARENT` (story 4) -- PARENT empty means top level.
- `README.md` mouse bullet's menu list and Files table.

## Tasks & Acceptance

**Execution:**
- [ ] `tabrail-nest` -- new, `GROUP CLIENT` opens the picker, `--pick GROUP LIST` runs `tabrail-groups nest` -- one script per job, like tabrail-move
- [ ] `src/menu.rs`, `src/main.rs` -- `'Move into group…' i` after Rename group, not for Default, name through `clean()` -- the entry point
- [ ] `tabrail-tree-test` -- the matrix rows (drive the popup with `send-keys` on the outer server) -- regression
- [ ] `README.md` -- Move into group… in the menu list, `tabrail-nest` in Files -- behaviour reference

**Acceptance Criteria:**
- Given a right-click on Default's header, then its menu has no Move into group….
- Given St.Gallen moved to Top level, when the sidebars redraw, then St.Gallen's header starts at column 1 and egov's tree no longer holds it.

## Verification

**Commands:**
- `cargo build --release --offline` -- expected: builds, no new warnings
- `bash -n tabrail-nest && wc -l tabrail-nest src/menu.rs` -- expected: syntax ok, under 100
- `./tabrail-tree-test` -- expected: every line `ok`
