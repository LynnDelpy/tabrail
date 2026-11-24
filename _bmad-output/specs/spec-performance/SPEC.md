---
id: SPEC-performance
companions:
  - cost-today.md
  - ../../../README.md
  - ../../../AGENTS.md
sources: []
---

> **Canonical contract.** This SPEC and the files in `companions:` are the complete, preservation-validated contract for what to build, test, and validate. Source documents listed in frontmatter are for traceability — consult them only if you need narrative rationale or prose color this contract intentionally omits.

# Tab switches stay fast with many tabs

## Why

Pain, for the one user of the sidebar: switching tabs is slow at times. With 53 tabs the tmux server averages 35% of a core, almost all of it sidebars of tabs nobody sees listing the whole session every second and on every change; a tab switch, or a Claude hook call, wakes all of them several times, and the new tab's sidebar waits in line behind them (cost-today.md). The cost grows with tabs × panes, so it gets worse with every tab.

## Capabilities

- **CAP-1**
  - **intent:** A tab switch shows the new tab with its sidebar current as fast with 50 tabs as with 5, the reader bar on.
  - **success:** On a throwaway server, the median time from `select-window` to the new tab's sidebar marking it current is at most 50 ms at 50 tabs and at most 1.5× the 5-tab median.
- **CAP-2**
  - **intent:** tabrail's load on the tmux server while nothing changes does not grow with the number of tabs.
  - **success:** tmux server CPU over 60 idle seconds at 50 tabs is at most 2× that at 5 tabs.
- **CAP-3**
  - **intent:** Busy Claudes do not slow tab switches.
  - **success:** With hook calls arriving 10 times a second, CAP-1's limits still hold at 50 tabs.
- **CAP-4**
  - **intent:** One Claude state hook call is cheap.
  - **success:** 100 calls in a row against a throwaway server take under 1 s together, and the options they leave are the same as today's.
- **CAP-5**
  - **intent:** The reader bar reaches the new tab soon after a switch.
  - **success:** At 50 tabs, the reader pane sits in the new tab at most 100 ms after the switch (median).
- **CAP-6**
  - **intent:** A reader slot waits without a shell.
  - **success:** `ps` shows no `bash` for any slot; a resurrect save and restore still brings every slot back.
- **CAP-7**
  - **intent:** One command measures switch time, reader time, idle load and hook cost at a given number of tabs on a throwaway server.
  - **success:** `tabrail-perf-test 5 50` prints, per tab count, the switch and reader medians in ms, idle tmux CPU in % and hook ms per call, and leaves no server or file behind.

## Constraints

- Speed only: screens, clicks, menus, drag, timers, widths and the reader bar behave as today; every existing `tabrail-*-test` passes.
- The sidebar shown keeps redrawing on every change and every second; a sidebar that becomes shown (switch, attach, session change) is current at once.
- Standard library only; files under 100 lines (`src/control.rs` is 106 and `src/main.rs` 99 already); tmux option and buffer names kept (AGENTS.md).
- The hook is `~/Dev/vmConfig/tmux-claude-state`, a separate repo: commit there on an `lde/` branch; its options and their meanings stay. spec-claude-jump's story 2 edits the same file.
- Resurrect keeps restoring slots: it saves the pane process's child, matched by `~tabrail-slot` in vmConfig `tmux.conf` `@resurrect-processes`.
- Measured and verified only on throwaway tmux servers (`tabrail-test-lib`), CPU read from the test server's pid; never the user's server.
- `README.md` is the behaviour reference: its "Fast:" bullet and Files table change with this work.

## Non-goals

- Startup and resurrect restore speed.
- Typing lag beyond tabrail's share of the tmux server.
- The reader chat's own speed (tabrail-chat, journal-reader), Claude Code's speed.
- New features or anything on screen.

## Success signal

`tabrail-perf-test 5 50` on a throwaway server prints a 50-tab switch median of at most 50 ms and at most 1.5× the 5-tab one, also with hook calls arriving 10 times a second; idle CPU at 50 tabs at most 2× that at 5; the reader in the new tab within 100 ms; hook calls under 10 ms each; and `ps` shows no `bash` waiting in any slot.

## Assumptions

- The targets (50 ms, 1.5×, 2×, 100 ms, 10 ms) are set by the spec from README's "~15 ms until the sidebar shows it", not by the user.
- Shown means: the sidebar's window is current in its session, and a client is attached to that session.
- The slot saving is small: about 1.5 MB RSS per tab (bash 3.9 MB replaced by a second `sleep`, 2.4 MB), ~75 MB at 50 tabs.
