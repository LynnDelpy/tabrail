---
id: SPEC-naming-escape
companions:
  - brownfield.md
  - ../../../README.md
  - ../../../AGENTS.md
sources: []
---

> **Canonical contract.** This SPEC and the files in `companions:` are the complete, preservation-validated contract for what to build, test, and validate. Source documents listed in frontmatter are for traceability — consult them only if you need narrative rationale or prose color this contract intentionally omits.

# Escape leaves naming

## Why

Pain, for the one user of the sidebar: Escape does not leave a naming prompt. The sidebar's menus name things through tmux's `command-prompt`, and with `EDITOR=vim` tmux gives prompts vi keys: Escape only enters normal mode, and only `q` or `C-c` cancels. Changing `status-keys` would change every tmux prompt, so tabrail's own naming moves to a field where Escape always cancels. Details: `brownfield.md`.

## Capabilities

- **CAP-1**
  - **intent:** Rename (tab), Rename group and New group… (from a tab, a group or empty space) take the name in a popup field, and one Esc closes it with nothing changed.
  - **success:** With `status-keys` vi, for each of the four menu items: type text, press Esc once; the popup is gone, and tab names, groups.conf, every tab's group and the tab count are as before.
- **CAP-2**
  - **intent:** The field opens holding the current name for a rename and empty for a new group; Enter saves with today's effects.
  - **success:** Rename shows the tab's name, and Enter on `deploy` renames the tab `deploy`. Rename group renames it in groups.conf, on its tabs and in the fold state. New group from a tab moves the tab in; from a group or empty space it opens a new tab in the new group.

## Constraints

- Esc cancels whatever `status-keys` is set to; tmux's own prompts (prefix `,`, `:`) and `status-keys` stay untouched.
- Looks like `tabrail-pick`: rounded box, the question as title, key hints in the footer, Catppuccin Mocha. No dependency beyond tmux, bash and fzf.
- Today's name rules stay: `clean()` for names in tmux strings, group names one word (spaces become `-`), `Default` cannot be renamed or created.
- After the field closes, either way, the keyboard is in the tab's content pane, as after any menu.
- A new script gets a row in README's Files table; files stay under 100 lines.
- Verification only on a throwaway tmux server started with `EDITOR=/usr/bin/vim`, the viewer with `-f /dev/null`; menus opened by a real right-click, Esc sent through the viewer.

## Non-goals

- tmux's own prompts and the `status-keys` setting.
- Resetting a tab to its automatic name.
- Typing a name inline in the sidebar row.
- The Move to group picker (fzf; Esc already cancels).

## Success signal

On a throwaway tmux server with vi prompt keys running tabrail: right-click a tab, pick Rename, type, press Esc once; the popup closes, the name is unchanged and the keyboard is back in the content pane. The same holds for Rename group and both New group… variants, and Enter in each saves as it does today.

## Assumptions

- Enter on an empty field changes nothing, same as Esc.
- Popup titles `Rename tab`, `Rename group`, `New group`; footer `Enter to save · Esc to cancel`.

## Open Questions

- Close's `(y/n)` confirmation has the same fault (Esc does not cancel with vi keys): in scope, as a `tabrail-pick` yes/no?
