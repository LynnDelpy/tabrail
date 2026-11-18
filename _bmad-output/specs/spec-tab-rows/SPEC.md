---
id: SPEC-tab-rows
companions:
  - row-layout.md
  - ../../../README.md
  - ../../../AGENTS.md
sources: []
---

> **Canonical contract.** This SPEC and the files in `companions:` are the complete, preservation-validated contract for what to build, test, and validate. Source documents listed in frontmatter are for traceability — consult them only if you need narrative rationale or prose color this contract intentionally omits.

# Tab rows: readable ssh names, no numbers, closer to the left

## Why

Pain, for the one user of the sidebar. Their shell alias `ssh='TERM=xterm-256color ssh'` reaches the pane title expanded, so every ssh tab is named `TERM=xterm…` and remote tabs cannot be told apart, which is exactly where knowing the host matters. Window numbers and indentation spend the sidebar's few columns on nothing the user reads.

## Capabilities

- **CAP-1**
  - **intent:** A tab named after its running command shows the command without leading `VAR=value` assignments, for any command.
  - **success:** Title `TERM=xterm-256color htop` shows `htop`; `FOO=1 BAR=2 make test` shows `make test`.
- **CAP-2**
  - **intent:** While connected, an ssh tab shows where it is: remote user, host and folder at a shell prompt, the application and its file while one runs, the hostname when the remote reports nothing.
  - **success:** Remote title `lde@app01: ~/src/infra` gives `lde@app01 infra`; at the remote home, `lde@app01 ~`; remote vim titling `nginx.conf (/etc/nginx) - VIM` gives `vim nginx.conf`; no remote title (yet) gives `app01`; after ssh exits, the tab is named as any local tab again.
- **CAP-3**
  - **intent:** Tab rows show no window number.
  - **success:** No row contains the window index; tab order, click-to-switch and drag-to-move behave as before.
- **CAP-4**
  - **intent:** Rows sit closer to the left edge.
  - **success:** Group rows (open and folded) start in column 1; tab rows carry the `❯` marker in column 2 and the name from column 4, per `row-layout.md`; every row still clicks to the same tab or group.

## Constraints

- Hand-given names (rename) stay verbatim; CAP-1 and CAP-2 change auto-derived names only.
- The local shell's own idle title (`lde@f4l002:~/Dev`) stays ignored; a `user@host:path` title counts as remote only while ssh runs in the pane.
- No remote-side setup: work from the titles remote shells already send. Prod hosts are not edited.
- Tab order, drag targets and click rows unchanged; the window index stays the ordering key, only its display goes.
- Verification never ssh-es to a real host: simulate the remote with a stand-in `ssh` that prints an OSC 2 title and waits, on a throwaway tmux server.
- Remote folder is its last path component (`~` for home), after `user@host` and a space, like local tabs.
- `README.md` is the behaviour reference: its example and tab-name description change with this work.

## Non-goals

- Sidebar width, colours, status icons, group tints, the clock.
- tmux window numbering or the prefix+digit bindings.
- Fixing titles at the source (the ssh alias, oh-my-zsh, remote shell config).
- A setting for the layout.

## Success signal

On a throwaway tmux server running tabrail, a tab that runs `TERM=xterm-256color ssh app01` against a stand-in remote titling `lde@app01: ~/src/infra` reads `lde@app01 infra`, and `capture-pane` of the whole sidebar matches the after-example in `row-layout.md`: no numbers, group rows from column 1, tab names from column 4.

## Assumptions

- Hostname is the ssh destination without user and domain (`lde@app01.example.ch` gives `app01`; an ssh-config alias as typed); the file is its basename.
