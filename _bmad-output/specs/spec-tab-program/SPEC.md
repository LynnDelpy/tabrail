---
id: SPEC-tab-program
companions:
  - ../spec-tab-rows/SPEC.md
  - ../spec-tab-rows/row-layout.md
  - ../../../README.md
  - ../../../AGENTS.md
sources: []
---

> **Canonical contract.** This SPEC and the files in `companions:` are the complete, preservation-validated contract for what to build, test, and validate. Source documents listed in frontmatter are for traceability — consult them only if you need narrative rationale or prose color this contract intentionally omits.

# Show the program running in each tab

## Why

Pain, for the one user of the sidebar: they want to see at a glance whether Claude runs in a tab, and generally which program runs. Today the tab name shows the program only sometimes. A name given by hand, a Claude topic, or a program's own title hides it, and a Claude tab is recognisable only by its status glyph, which a crashed Claude leaves behind on a plain shell tab.

## Capabilities

- **CAP-1**
  - **intent:** While a program other than the shell runs in a tab, its row shows the program's name, dimmed, after the tab name; an idle shell shows nothing extra.
  - **success:** A tab renamed `deploy` running vim reads `deploy · vim`; after vim exits it reads `deploy`; an ssh tab reads `lde@app01 infra · ssh`.
- **CAP-2**
  - **intent:** When the tab name's first word is the program, nothing is added.
  - **success:** A tab running htop with no title reads `htop`, not `htop · htop`; `make test` reads `make test`.
- **CAP-3**
  - **intent:** A tab where the claude process runs shows `✳` in place of `· claude`.
  - **success:** A Claude tab reads `Fix login redirect ✳ ⚙ 2m`; once claude exits, no `✳`.
- **CAP-4**
  - **intent:** Once the claude process exits, the tab is shown as any shell tab, with no Claude status left over from a stale `@claude`.
  - **success:** A tab with `@claude` `waiting` running a plain shell shows no `● waits` and is named and timed as a shell tab.

## Constraints

- The program is `pane_current_command` of the tab's content pane, already read (`model.rs` FIELDS); no new tmux call, nothing spawned on redraw or tab switch.
- Shells (`zsh`, `bash`, `sh`, `fish`, as `name.rs` lists them) count as no program.
- Hand-given names stay verbatim, and the program is added beside a name, never replaces it.
- Status icons keep their meaning; the program shows as a word, and `✳` is the only new glyph (`✎` and `⚙` already mean Claude states).
- Built after tab-rows: both change `name.rs` and `draw.rs`; this work uses its row layout and ssh names.
- `README.md` is the behaviour reference: its example and tab-name description change with this work.
- Verification only on a throwaway tmux server, with stand-in `claude` and `ssh` scripts; never the user's tmux.

## Non-goals

- A glyph or colour per program.
- The program running on the remote side of ssh beyond what tab-rows' name already says.
- A setting to turn the program display off.
- Sidebar width, group rows, the clock.

## Success signal

On a throwaway tmux server running tabrail with tab-rows built, `capture-pane` of a sidebar shows `deploy · vim` for a renamed tab running vim, `htop` for htop, only the name for an idle shell, `✳` after a stand-in Claude tab's name, `lde@app01 infra · ssh` for a stand-in ssh tab titling `lde@app01: ~/src/infra`, and no `● waits` on a plain shell tab with `@claude` set to `waiting`.

## Assumptions

- A long name is cut first; program and status stay whole.
- The program and `✳` sit between name and status, in the dim colour.
- The program is what tmux reports: interpreters show as themselves (`npm run build · node`), `sudo vim` as `sudo`.
- A shell running a function or loop (`pane_current_command` is the shell) shows no program.
- `pane_current_command` stays `claude` while Claude runs tool commands; otherwise CAP-4 would hide the status mid-turn.
