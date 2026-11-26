---
id: SPEC-more-colours
companions:
  - new-colours.md
  - ../spec-group-colour/SPEC.md
  - ../../../README.md
  - ../../../AGENTS.md
sources: []
---

> **Canonical contract.** This SPEC and the files in `companions:` are the complete, preservation-validated contract for what to build, test, and validate. Source documents listed in frontmatter are for traceability — consult them only if you need narrative rationale or prose color this contract intentionally omits.

# Enough group colours

## Why

Lynn does not have enough colours. Their groups file has 29 groups, but the palette has 25 colours, so Brown, Green, Violet and Lavender deep each mark two groups. Once group icons go (spec-no-group-icon), the tint is the only thing that tells groups apart. Today's 25 also bunch up: blended onto the dark base, most of them land between blue and pink.

## Capabilities

- **CAP-1**
  - **intent:** Colour… and New group… offer 40 colours, so each of today's 29 groups, and 11 more, can have its own.
  - **success:** Colour… lists 40 swatches. With all 25 old colours in use, New group… gives the new group `#4f4821` (Mustard).
- **CAP-2**
  - **intent:** Each new colour can be told apart at a glance from every other palette colour behind a group row.
  - **success:** Every new colour is at least 4.0 (OKLab × 100) from all 39 others, more than twice the 1.8 of today's closest pair, and carries the bold header text at 4.5:1 or more (`new-colours.md`).

## Constraints

- Today's 25 stay as they are (same hex, name and order) and come first. Groups that use them keep their current mark, and New group… still takes them first. The 15 new ones are appended.
- New colours keep today's look: OKLab L 0.30–0.47 and chroma up to 0.06, so they are no brighter or stronger than the deep tints already in use.
- Each new colour has a unique one-word name, and searching an existing colour's name (e.g. `teal`) in the picker still finds only that colour's rows.
- Nothing else changes: `palette` stays `#rrggbb Name` per line, Colour… and New group… read it as today, and the popup stays no taller than the terminal (fzf scrolls the rest).

## Non-goals

- Changing today's 25, including their near-duplicate pairs: that would recolour groups already using them.
- Colours for Default, tab rows or the name text.
- Recolouring anyone's groups file automatically.
- Typing a custom hex in the picker.

## Success signal

On a throwaway tmux server: Colour… on a group shows 40 swatch rows, typing `mustard` leaves one, and Enter writes `#4f4821` into the group's column 2. With every old colour in use, New group… writes `#4f4821`. `tabrail-colour-test` passes.

## Assumptions

- Hues between Catppuccin's accents are fine. Blending the 14 accents at more depths cannot reach 40 without near-duplicates.
- Lynn gives the 4 groups that share a tint new colours by hand with Colour… once the palette grows.
