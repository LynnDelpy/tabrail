---
id: SPEC-agent-journal
companions:
  - journal-records.md
  - ../../../README.md
  - ../../../AGENTS.md
sources: []
---

> **Canonical contract.** This SPEC and the files in `companions:` are the complete, preservation-validated contract for what to build, test, and validate. Source documents listed in frontmatter are for traceability — consult them only if you need narrative rationale or prose color this contract intentionally omits.

# Agent journal and reader pane

## Why

Pain for the user of many Claude Code sessions on f4l002: what an agent did, where, and which versions it changed or saw is buried in per-session transcripts (291 MB of them), so "which version of X did we bump, where, and when" means digging. Every session should leave a short record of its work in one place, and one chat, always at hand under the sidebar, should answer from it quickly.

## Capabilities

- **CAP-1**
  - **intent:** Every Claude Code session on f4l002, in any project, leaves a raw record of each command it runs and each file it edits, with folder, repo, session and time.
  - **success:** A session in any folder that runs `rpm -q tmux` and edits a file adds one record for each to the journal, carrying the fields in `journal-records.md`, including `tmux` with its version; a command carrying `--password hunter2` or `TOKEN=abc123` is recorded with the secret masked.
- **CAP-2**
  - **intent:** Each task that changed something gets a short distilled entry: what was done, where, which versions changed or were seen, and pointers to repo, files, commits, PR and transcript.
  - **success:** Within a minute of a turn that bumps a dependency and opens a PR, the journal holds an entry of at most 5 lines naming the package, old and new version, the repo, the PR URL and the transcript.
- **CAP-3**
  - **intent:** One reader chat answers questions about versions agents touched, tool versions on this VM, and what was done where, naming the journal entries each answer comes from.
  - **success:** Asked "which tmux version is here and when did an agent last change it?" and "which session changed tabrail-act, and why?", it answers both from the journal without opening a transcript, citing the entries; a question the journal cannot answer gets the command to run instead.
- **CAP-4**
  - **intent:** The reader chat sits in a pane under the sidebar's tab list and follows the user from tab to tab, keeping its conversation.
  - **success:** With the pane open in tab 1 after a question, switching to tab 3 shows the same conversation under tab 3's sidebar (same `claude` pid), and tab 1's sidebar column shows only the sidebar.
- **CAP-6**
  - **intent:** The reader pane is always open and really small; clicking into it makes it big enough to work in, and leaving it makes it small again.
  - **success:** In every tab the pane shows at about 4 rows; a click into it grows it, a click into the work pane shrinks it back, every sidebar returns to its width and the work pane to its size, and the conversation and `claude` pid are unchanged. CAP-5 (show and hide) is retired.

## Constraints

- Hooks print nothing and never fail or block a tool call; a raw record costs under 50 ms per call; the distilled entry is written in the background, never delaying the turn's end.
- One central journal for all projects: an index of where things are (repo, file, commit, PR, transcript), not copies of them.
- The journal is a local folder outside any git repo, never committed or synced.
- The journal never holds secrets: tokens, passwords, keys and credentials in commands (arguments, `VAR=value`, URL userinfo, auth headers) are masked before a raw record or entry is written.
- The reader chat never connects to servers or company services; what the journal and its skills cannot answer, it answers with the command to run.
- The reader chat reads the journal, not transcripts, for questions the journal answers; a transcript only to drill into an entry that points to it.
- tabrail works as before without the journal or the reader pane, like the optional Claude state.
- Growing the reader pane never changes the stored sidebar width (`@sidebar_width`): leaving it brings every sidebar and the work pane back to their sizes.
- A tab holding only a sidebar and the reader pane closes like a sidebar-only tab, the pane moving on first; `tabrail-tmux restart` never kills the reader chat.
- `README.md` is the behaviour reference: its sidebar and optional-inputs text change with this work.

## Non-goals

- Live version lookups on servers or company services.
- Replacing, moving or deleting Claude Code's own transcripts.
- Recording what the user types in shells outside Claude Code.
- A web UI or dashboard for the journal.
- More than one reader chat.
- Hiding the reader pane.

## Success signal

After a day of normal agent work in two projects, asking the reader pane "which versions did agents change today?" lists each change with repo and PR, matching `git log` in those repos, and "which tmux is on this VM?" answers from the journal with where it was seen; `grep` for a password used that day finds nothing in the journal; switching tabs keeps the small pane and its conversation under the sidebar, and clicking into it grows it.

## Assumptions

- Distilled entries only for turns that edited files or ran commands; turns that only read or answer get none.
- Summaries come from a small fast model (Haiku 5.5) run headless in the background.
- A raw record keeps commands, paths and version strings found in output, never the output itself.
- The journal hooks and the reader chat's setup (its folder, CLAUDE.md and skills) live in `~/Dev/vmConfig` beside `tmux-claude-state`; tabrail hosts only the pane.
- The reader chat's own sessions are not journaled.
- Small is about 4 rows at the sidebar's width (Claude Code's input box); big is the column widened to about 80 columns with the pane taking about two thirds of its height.
- With two tmux sessions attached, the pane sits in the tab most recently switched to.
- Journal entries are kept; nothing prunes them.
