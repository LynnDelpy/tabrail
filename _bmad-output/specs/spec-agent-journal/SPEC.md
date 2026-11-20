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
  - **success:** A session in any folder that runs `rpm -q tmux` and edits a file adds one record for each to the journal, carrying the fields in `journal-records.md`, including `tmux` with its version.
- **CAP-2**
  - **intent:** Each task that changed something gets a short distilled entry: what was done, where, which versions changed or were seen, and pointers to repo, files, commits, PR and transcript.
  - **success:** Within a minute of a turn that bumps a dependency and opens a PR, the journal holds an entry of at most 5 lines naming the package, old and new version, the repo, the PR URL and the transcript.
- **CAP-3**
  - **intent:** One reader chat answers questions about versions agents touched, tool versions on this VM, and what was done where, naming the journal entries each answer comes from.
  - **success:** Asked "which tmux version is here and when did an agent last change it?" and "which session changed tabrail-act, and why?", it answers both from the journal without opening a transcript, citing the entries; a question the journal cannot answer gets the command to run instead.
- **CAP-4**
  - **intent:** The reader chat sits in a pane under the sidebar's tab list and follows the user from tab to tab, keeping its conversation.
  - **success:** With the pane open in tab 1 after a question, switching to tab 3 shows the same conversation under tab 3's sidebar (same `claude` pid), and tab 1's sidebar column shows only the sidebar.
- **CAP-5**
  - **intent:** The user can show and hide the reader pane; hidden, it keeps its conversation.
  - **success:** Hide the pane, switch tabs, show it: the conversation and the `claude` pid are unchanged.

## Constraints

- Hooks print nothing and never fail or block a tool call; a raw record costs under 50 ms per call; the distilled entry is written in the background, never delaying the turn's end.
- One central journal for all projects: an index of where things are (repo, file, commit, PR, transcript), not copies of them.
- The reader chat never connects to servers or company services; what the journal and its skills cannot answer, it answers with the command to run.
- The reader chat reads the journal, not transcripts, for questions the journal answers; a transcript only to drill into an entry that points to it.
- tabrail works as before without the journal or the reader pane, like the optional Claude state.
- A tab holding only a sidebar and the reader pane closes like a sidebar-only tab, the pane moving on first; `tabrail-tmux restart` never kills the reader chat.
- `README.md` is the behaviour reference: its sidebar and optional-inputs text change with this work.

## Non-goals

- Live version lookups on servers or company services.
- Replacing, moving or deleting Claude Code's own transcripts.
- Recording what the user types in shells outside Claude Code.
- A web UI or dashboard for the journal.
- More than one reader chat.

## Success signal

After a day of normal agent work in two projects, asking the reader pane "which versions did agents change today?" lists each change with repo and PR, matching `git log` in those repos, and "which tmux is on this VM?" answers from the journal with where it was seen; switching tabs keeps the pane and its conversation under the sidebar.

## Assumptions

- Distilled entries only for turns that edited files or ran commands; turns that only read or answer get none.
- Summaries come from a small fast model (Haiku 5.5) run headless in the background.
- A raw record keeps commands, paths and version strings found in output, never the output itself.
- The journal hooks and the reader chat's setup (its folder, CLAUDE.md and skills) live in `~/Dev/vmConfig` beside `tmux-claude-state`; tabrail hosts only the pane.
- The reader chat's own sessions are not journaled.
- The reader pane takes the bottom half of the sidebar column, resizable by its top border, height kept across tabs; a click on a bottom row of the sidebar or a prefix key shows and hides it.

## Open Questions

- Journal location: inside `~/Dev/vmConfig` (git, synced, needs scrubbing) or a local folder outside any repo?
- Secrets and retention: scrub tokens, passwords and keys from recorded commands before writing? Keep entries forever or prune after some months?
- Reader pane width: Claude Code is cramped at ~26 columns; widen the sidebar column (all tabs, still one width) while the pane is shown, or keep it narrow?
