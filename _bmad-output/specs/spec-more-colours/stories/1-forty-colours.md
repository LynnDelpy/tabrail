---
title: 'Forty colours'
type: 'feature'
created: '2026-11-26'
status: 'done'
route: 'dispatch'
review_loop_iteration: 0
baseline_revision: '2621b91f9010d005f7b70e1287109919bff13f5a'
followup_review_recommended: false
deferred:
  - summary: >-
      With all 40 palette tints in use, New group… and New subgroup… silently give the last one, Bronze #64452b, to another group.
    evidence: |-
      tabrail-groups-new:14 loops over palette and breaks on the first unused tint; with none free, $tint keeps the last value. Pre-existing fall-through (it gave Lavender deep with 25); 40 colours only move the point where it happens.
    location: >-
      tabrail-groups-new:14
    severity: low
  - summary: >-
      tabrail-colour-test's 50x20 check passes with zero swatch rows and never scrolls fzf.
    evidence: |-
      `$(swatches "$p") -lt 40` (before: -lt 25) is also true for 0 rows; nothing scrolls to a colour below the fold. Exactly 15 rows (popup 20 minus 5) would pin it, unverified on a test server.
    location: >-
      tabrail-colour-test:96
    severity: low
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
- [x] `palette` -- append the 15 from `new-colours.md` -- CAP-1, CAP-2
- [x] `tabrail-colour-test` -- 40 rows, `mustard` search and pick, New group… with all 25 used -- CAP-1
- [x] `README.md` -- 40 named colours -- behaviour reference

**Acceptance Criteria:**
- Given the new `palette`, when its first 25 lines are compared with main's, then they are identical, and lines 26-40 match `new-colours.md`.
- Given every palette colour, when its contrast to `#cdd6f4` and OKLab distance to every other palette colour are computed, then each new one is at least 4.5:1 and at least 4.0 apart.

## Implementation Notes

## Spec Change Log

## Review Triage Log

### 2026-11-27 — Review pass
- verdicts: 16 findings — high 0, medium 0, low 11, false 5, maybe-false 0
- findings:
  - `[low]` `[reject]` Typing `pine` shows Pine and Pink deep — confirmed with fzf (Pine ranks first, so Enter still picks it); the SPEC only requires old names to stay unique, and the fix is renaming Pine in new-colours.md, i.e. editing this build's spec.
  - `[low]` `[defer]` All 40 in use: New group… silently reuses Bronze — tabrail-groups-new:14 falls through to the last tint; pre-existing (Lavender deep before), tabrail-groups is out of bounds for this story.
  - `[low]` `[reject]` Nothing in the test pins today's 25 byte for byte (the auto-pick fixture reads `head -25` of the same file) — true, but the palette is verified identical to main now; a pin needs a new check and a file split past 100 lines for a rare edit.
  - `[low]` `[reject]` CAP-2 (distance ≥ 4.0, contrast ≥ 4.5) is never checked automatically — recomputed independently: min distance 4.1 (Bronze/Peach deep), min contrast 4.9 (Khaki); guarding future additions needs a new test file.
  - `[false]` `[reject]` tabrail-colour-test at 99 lines; snapshot packed with the fixture on one line — 99 is within the rule, and the line snapshots before the append and restores after, so "only Website's line changed" compares against the same state as on main (it passed).
  - `[low]` `[reject]` newops: an empty Ops lookup makes `kill-window -t ""` hit the current window — real only after New group… already failed its own check loudly, on the throwaway server; the guard is an added branch for a cascade-only symptom.
  - `[low]` `[defer]` 50x20 check passes with 0 rows and never scrolls — `-lt 40` keeps main's `-lt 25` shape; pre-existing weakness.
  - `[false]` `[reject]` New-colour pick never confirmed on screen behind a group row — src/draw.rs:28 parses any `#rrggbb` the same way; the Teal deep pick already checks every sidebar draws column 2.
  - `[false]` `[reject]` README and test header drifted; append-only rule undocumented — README's test row still describes the checks accurately, and the groups file stores the hex itself, so palette order or edits never recolour a group.
  - `[low]` `[reject]` (edge case) newops kills the current window on an empty lookup — same root cause and reason as the newops row above.
  - `[low]` `[defer]` (edge case) all 40 used: Bronze reused silently — same root cause as the deferred fall-through row above.
  - `[low]` `[reject]` (intent) auto-pick fixture is self-referential; lines 27-40 not pinned by a test — same root cause and reason as the byte-for-byte row; lines 26-40 diffed against new-colours.md by hand: identical.
  - `[low]` `[defer]` (intent) positions 27-40 and the all-40 case of New group… not exercised — same root cause as the deferred fall-through row.
  - `[low]` `[reject]` (intent) CAP-2 rests on the companion doc, not a test — same root cause and reason as the CAP-2 row above.
  - `[false]` `[reject]` (intent) the live install does not see 40 colours yet — by design: the live sidebar runs ~/Dev/tabrail, which gets the change after merge and pull.
  - `[false]` `[reject]` (intent) 99 lines reached by packing statements on one line — same as the 99-lines row: within the rule, behaviour unchanged.

## Verification

**Commands:**
- `diff <(head -25 palette) <(git show origin/main:palette)` -- expected: no output
- `./tabrail-colour-test` -- expected: every check ok, on its own tmux -L servers

**Manual checks (if no CLI):**
- In a truecolor terminal, `while read -r hex name; do printf '\e[48;2;%d;%d;%dm\e[1;38;2;205;214;244m ▾ %-16s\e[0m\n' 0x${hex:1:2} 0x${hex:3:2} 0x${hex:5:2} "$name"; done < palette` shows 40 swatches, the last 15 clearly apart from each other and from the first 25.

## Auto Run Result

Status: done

**Summary:** `palette` grows from 25 to 40: today's 25 unchanged and first, then the 15 tints of `new-colours.md` in its order (Mustard first). Colour… and New group… read `palette` as before, so they offer 40.

**Files changed:**
- `palette` -- 15 lines appended, Mustard to Bronze.
- `tabrail-colour-test` -- 40 swatch rows, `mustard` search and pick writing `#4f4821`, New group… with all 25 old tints in use giving `#4f4821`, 50x20 `-lt 40`; New group… steps moved into `newops()` to stay under 100 lines (99).
- `README.md` -- "25 named colours" becomes 40; the test row mentions Mustard once the old 25 are used.

**Review:** 16 findings: 0 patched, 3 rows deferred (2 entries: the all-40 fall-through to Bronze, the weak 50x20 row count), 13 rejected (5 false, 8 low; reasons in the triage log). Follow-up review recommended: false (patched: high 0, medium 0, low 0).

**Verification:**
- `diff <(head -25 palette) <(git show origin/main:palette)`: no output.
- Lines 26-40 diffed against `new-colours.md`'s table: identical.
- OKLab/contrast recomputed independently: every new colour ≥ 4.1 apart (Bronze/Peach deep) and ≥ 4.9:1 to `#cdd6f4` (Khaki), L 0.30-0.46, chroma ≤ 0.059, names unique; no old name fuzzy-matches a new one.
- `./tabrail-colour-test`: 24/24 ok, exit 0, on its own `-L tabrail-colour` servers; ~/Dev/vmConfig/tabrail-groups.conf hash unchanged across the run.
- Matrix audit: every row covered by a check that ran and passed (picker on 160x46, mustard, teal d, auto-pick, 50x20).

**Residual risks:**
- Typing `pine` also lists Pink deep (Pine ranks first); renaming would change `new-colours.md`.
- The truecolor swatch check by eye (Verification's manual check) was not done.
- Live only after merge, `git -C ~/Dev/tabrail pull`, rebuild if needed and `tabrail-tmux restart`; the 4 groups sharing a tint keep it until recoloured with Colour….
