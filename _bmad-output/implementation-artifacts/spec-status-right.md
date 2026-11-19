---
title: 'Status and timers at the right edge'
type: 'feature'
created: '2026-11-19'
status: 'done'
route: 'oneshot'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/specs/spec-status-right/SPEC.md'
  - '{project-root}/_bmad-output/specs/spec-status-right/row-layout.md'
  - '{project-root}/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** A tab row's status (`⚙ 2m`, `⏱ 4m`, `✓ 3m12s`, `✗ 2`) follows its name and program, so timers end at a different column on every row and cannot be scanned down.

**Approach:** Story 1 of spec-status-right (CAP-1, CAP-2): the tab row's padding moves from after the status to before it, so the whole status ends at the sidebar's current width; name, program, the cut and drop rules, rows without status, group rows and the clock line stay as they are. README's example rows and status description show the right-aligned status.

</frozen-after-approval>

## Implementation Notes

- `src/draw.rs` screen: the tab row is now label, program, padding, status (was label, program, status, padding); `room`, `rest`, the cut and the program's drop rule are untouched, so rows that fill the width come out byte-identical and rows without status look as before (the status colour code now follows the padding). The padding follows the program's dim colour code; spaces carry no visible foreground and tab rows set no background, so it reads as plain space. The trim comment now names only the program: before the status a gap wider than one space is the alignment.
- `README.md`: example rows `website·release-2 ✳   ◇` and `infra               ✗ 2` right-aligned at 26 (annotations stay at column 36); the Status bullet says the status sits at the right edge. The clock line stays as it was (DW-2).
- Verified on `tmux -L stest` shown in `tmux -L sout` (bash --norc, OSC 133 and titles sent by printf, `exec -a vim|claude sleep` for the foreground program): at width 26 `deploy · vim ⏱ 5s`, `infra ✗ 2`, `Ready specs ✳ ✓ 3m12s` and `Fix login redire ✳ ⚙ 2m` (current and not) all end in column 26, `notes` unchanged; after `set -g @sidebar_width 34` all end in column 34 and the name shows whole. `pipe-pane` on one sidebar over 3 s: only row 3 (the timer) rewritten per tick, no `\e[2J`.
- Surprise: `TMUX_TMPDIR` in the scratchpad exceeds the unix socket path limit; the test used `-L` names in the default dir.
- Review patches: README's clock line centred as `draw.rs` draws it (`{clock:^26}`, 4 spaces) so it no longer sticks out past the right edge the rows now show; module doc says "the clock at the bottom".

## Review Triage Log

- Double-width character in a name wraps the status's last character: low, rejected. Real, but DW-1 (char-count width) predates this story and already wrapped a padding space into the next row; rare in tab names and the fix needs a wcwidth table.
- No regression test for the new layout: low, deferred as DW-19 (same owner call as DW-3/DW-10).
- README clock line ends in column 29, past the right edge the example rows now show: low, patched (clock centred as drawn; DW-2's other points stay open).
- README "times of all rows end in one column" inaccurate for statuses without a time: false. The bullet puts the status at the right edge; times ending in one column is the consequence, not a claim that only times align.
- Spec companion `spec-tab-rows/row-layout.md` contradicts CAP-1: low, deferred as DW-20 (spec edit).
- Trim comment no longer explains the gap before the status: false. Line 73 says the status sits at the edge and `rest` is computed as the gap; nothing to break.
- Sidebar narrower than its status overflows: low, rejected. Unchanged by this story (`room` and `rest` were already 0 there); needs a width clamp in width.rs, and nobody drags the sidebar under 9 columns.
- Verification misses cut-at-space, program drop/return and group rows at 34: false. The row is `label + prog + rest + state` with `rest = width - len(label) - len(prog) - len(state)` and the cut keeps `len(label) <= width - len(prog) - len(state)`, so every case ends at `width`; group and folded rows go through untouched `fit`.
- Module doc "the time at the bottom" ambiguous next to timers: low, patched ("the clock").

