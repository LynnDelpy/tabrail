---
title: 'Forty colours'
type: 'feature'
created: '2026-11-26'
status: 'ready-for-dev'
route: 'dispatch'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/specs/spec-more-colours/SPEC.md'
  - '{project-root}/_bmad-output/specs/spec-more-colours/new-colours.md'
  - '{project-root}/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** The palette has 25 colours for Lynn's 29 groups, so four tints mark two groups each, and with group icons going the tint is a group's only mark (2026-11-26).

**Approach:** Append the 15 tints of `new-colours.md` to `palette`, in that order, after today's 25. Colour… and New group… read `palette` as they do now, so they offer 40.

## Boundaries & Constraints

**Always:** Today's 25 lines byte for byte, first. The 15 exactly as `new-colours.md` lists them (`#rrggbb Name`, lowercase hex, its order). README's count follows. Files under 100 lines.

**Never:** Changes to tabrail-colour, tabrail-groups or the picker; touching anyone's groups file; testing on the default tmux socket.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Picker | Colour… on Website, 160x46 client | 40 swatch rows, Blue marked current | N/A |
| Search new | type `mustard` | one row, Mustard; Enter writes `#4f4821` into column 2 | N/A |
| Search old | type `teal d` | one row, Teal deep, as today | N/A |
| Auto-pick | all 25 old tints in use, New group… `Ops` | `Ops` line with `#4f4821` | N/A |
| Small client | 50x20 | popup fits, fzf scrolls, fewer than 40 rows shown | N/A |

</frozen-after-approval>

## Code Map

- `palette` -- 25 lines `#rrggbb Name`; append 15.
- `tabrail-colour:34-46` -- one swatch row per palette line, popup height `rows + 5` capped at the terminal: no change.
- `tabrail-groups:79` -- New group… takes the first palette hex no group line uses (case-insensitive): no change.
- `tabrail-colour-test:48-50` -- "25 swatch rows" becomes 40 (the 160x46 client fits 40 + 5); `:55-56` "te: five rows" stays 5 (no new name matches `te`); `:90` `-lt 25` becomes `-lt 40`. Add a New group… check with a groups file using all 25 old tints expecting `#4f4821`. The file is 91 lines: split by job if it passes 100.
- `README.md:18` -- "a picker of the 25 named colours" becomes 40.

## Tasks & Acceptance

**Execution:**
- [ ] `palette` -- append the 15 from `new-colours.md` -- CAP-1, CAP-2
- [ ] `tabrail-colour-test` -- 40 rows, `mustard` search and pick, New group… with all 25 used -- CAP-1
- [ ] `README.md` -- 40 named colours -- behaviour reference

**Acceptance Criteria:**
- Given the new `palette`, when its first 25 lines are compared with main's, then they are identical, and lines 26-40 match `new-colours.md`.
- Given every palette colour, when its contrast to `#cdd6f4` and OKLab distance to every other palette colour are computed, then each new one is at least 4.5:1 and at least 4.0 apart.

## Implementation Notes

## Spec Change Log

## Review Triage Log

## Verification

**Commands:**
- `diff <(head -25 palette) <(git show origin/main:palette)` -- expected: no output
- `./tabrail-colour-test` -- expected: every check ok, on its own tmux -L servers

**Manual checks (if no CLI):**
- In a truecolor terminal, `while read -r hex name; do printf '\e[48;2;%d;%d;%dm\e[1;38;2;205;214;244m ▾ %-16s\e[0m\n' 0x${hex:1:2} 0x${hex:3:2} 0x${hex:5:2} "$name"; done < palette` shows 40 swatches, the last 15 clearly apart from each other and from the first 25.
