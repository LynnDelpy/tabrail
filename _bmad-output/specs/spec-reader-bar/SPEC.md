---
id: SPEC-reader-bar
companions:
  - stack.md
  - ../../../README.md
  - ../../../AGENTS.md
sources: []
---

> **Canonical contract.** This SPEC and the files in `companions:` are the complete, preservation-validated contract for what to build, test, and validate. Source documents listed in frontmatter are for traceability — consult them only if you need narrative rationale or prose color this contract intentionally omits.

# Reader bar: the reader chat across the bottom, in its own frontend

## Why

Pain for the user of the reader pane (spec-agent-journal): the chat sits 26 columns wide under the sidebar, and Claude Code's own TUI spends its 5 rows on a prompt box and a mode line, so the small pane shows almost nothing of an answer. Moved to a full-width bar along the bottom of the tab and driven by a frontend built for a few rows, the small bar shows the conversation itself, readable without clicking in. This supersedes the pane's placement and small size in spec-agent-journal CAP-4 and CAP-6; following tabs, growing (now sized by the chat in it, CAP-2) and the housekeeping rules stay.

## Capabilities

- **CAP-1**
  - **intent:** The reader pane sits as a bar across the full width of the tab shown, below the sidebar and the tab's own panes, and follows from tab to tab with the same process and conversation; every other tab keeps an idle slot of the bar's small size in its place, which the bar swaps places with, so a tab switch resizes none of the tabs' own panes.
  - **success:** With the bar in tab 1 after a question, switching to tab 3 shows it at the bottom of tab 3 with `pane_width` equal to `window_width`, `pane_left` 0 and the same pid; tab 1 holds a 4-row slot where the bar was, no reader pane; switching 1, 3, 1, 3 changes the size of no pane in either tab but the bar's and the slots'.
- **CAP-2**
  - **intent:** The bar is small; a click into it gives it the keyboard; with the frontend in it, it sizes to what it shows while it has the keyboard, up to a cap, and two buttons in it make it bigger and smaller; a click into the tab's own pane shrinks it again.
  - **success:** Small, the bar is 4 rows; a click into it gives it the keyboard at 4 rows; with the keyboard, the input or an answer changing sets it to the rows of the input and the latest question and its answer (tool lines included), rounded up to a multiple of 4, between 4 rows and a third of the window height, at most one resize every 500 ms; while a new question is typed (no turn running, the input not empty) the input's rows and one conversation row instead, so a short question after a long answer keeps it at 4 rows; `▲` at the end of the input's last row grows it by a sixth of the window height, at most 2/3, `▼` shrinks it by a sixth, at least 4 rows, a size that holds until the next question is sent or the bar is back at 4 rows; a click into the tab's own pane brings it back to 4 rows, where it stays while answers arrive; it leaves a tab at 4 rows and arrives at 4; a double-click, a click or focus alone never grows it; through all, every pane keeps its width, `@sidebar_width` is never written, a zoomed tab stays zoomed, and after a terminal resize the bar and the slots get their height back.
- **CAP-3**
  - **intent:** A chat frontend for Claude Code that spends every row of a few-row pane on the conversation: the input line and the latest answer, streaming in as it is written.
  - **success:** In a 4-row bar, after a question, the bar shows the last 3 lines of the answer wrapped at the bar's full width above the input line while the answer is still arriving; no logo, box, border, status or mode line is drawn.
- **CAP-4**
  - **intent:** The frontend runs any Claude Code chat: the command that starts it picks the folder, model and Claude Code options, and the journal reader chat runs in it.
  - **success:** `~/Dev/vmConfig/journal-reader/start` runs the frontend, which answers "which tmux is on this VM?" on Haiku 5.5 from journal-reader's folder through the journal-search skill under that folder's settings, and after the chat is restarted the next answer still knows the previous question.
- **CAP-5**
  - **intent:** The user scrolls back through the whole conversation of the running chat with the mouse wheel and keys.
  - **success:** After three questions, scrolling up in the small bar reaches the first question; typing a key or a new answer arriving shows the bottom again.
- **CAP-6**
  - **intent:** Each tool call Claude makes shows as one dim line naming the tool and a short form of its argument, never its output.
  - **success:** A turn that runs `./journal-search -R tmux` twice and reads one file adds three one-line entries (two Bash with their command, one Read with its path), each cut to the bar's width, and no tool output.
- **CAP-7**
  - **intent:** When Claude Code asks permission for a tool call, the user answers allow or deny in the bar.
  - **success:** A Bash call outside the folder's allowlist shows one question line naming the tool and its command; `y` runs it, `n` denies it and Claude continues the turn; an allowlisted call never asks.
- **CAP-8**
  - **intent:** `/clear` starts a new conversation and `/model <name>` switches the model; other slash commands are refused in the bar.
  - **success:** After `/clear` the bar is empty and the next question has no memory of earlier ones; after `/model sonnet` the next answer comes from Sonnet; `/compact` shows a one-line "not supported" and nothing is sent to Claude.

## Constraints

- Standard library only, no crates (AGENTS.md): the frontend's JSON, terminal and process handling are hand-written.
- The frontend drives Claude Code's own CLI, not the API or an SDK package: same login, settings, hooks, skills and CLAUDE.md, no API key.
- Moving, growing and shrinking the bar change pane heights only: no pane's width changes and `@sidebar_width` is never written.
- tabrail works as before without `@tabrail_reader`; the frontend runs only when the reader command starts it.
- Carried from spec-agent-journal: a tab left with only its sidebar and the reader pane (or its slot) closes, the pane moving on first; `tabrail-tmux restart` never kills the reader chat; the option names `@reader`, `@reader_big`, `@tabrail_reader`, `@tabrail_reader_started` stay.
- Files under 100 lines, split by job; each new file gets a row in README's Files table.
- `README.md` is the behaviour reference: its reader-pane text and Files table change with this work.

## Non-goals

- Replacing Claude Code's TUI for work tabs: no other slash commands, attachments, images, diffs, plan or subagent views.
- Rendering markdown.
- Editing, rewinding or forking earlier turns.
- More than one reader chat, or hiding the bar.

## Success signal

Through a day of normal work the 4-row bar along the bottom of every tab shows the reader chat's latest answer at full width, readable without clicking in; asking "which versions did agents change today?" streams the answer into the small bar, tool calls show as single lines, and with the keyboard in it the bar grows with a long answer (`▲` for more) that the wheel scrolls back through. Tab switches keep the conversation and every pane's width.

## Assumptions

- The bar spans the whole window, under the sidebar too: the sidebar column ends above it ("over the full page").
- Small is 4 rows: 3 conversation lines and the input line. The bar's own height reaches at most a third of the window height, its `▲` button 2/3.
- Each tab's slot is a pane of its own (`@reader_slot`, running `tabrail-slot`), as big as the small bar; the bar takes its place with `swap-pane`, so tab switches resize no other pane. A slot comes back after a resurrect restore through its `@resurrect-processes` entry.
- The bar grows only when the chat's content changes while it has the keyboard, or on its `▲` button; never on a click, a double-click, focus or a tab switch alone: a single click right under the work pane's prompt popped it up too often, and a double-click went unnoticed (tmux counts one only within about 300 ms, and nothing on screen hints at it). tabrail binds no `DoubleClick1Pane`.
- Answers show as plain text wrapped at the bar's width; one character is one column.
- Enter sends a single input line; Esc interrupts a running answer.
- Scroll back covers the running frontend's turns; after a restart the conversation continues in Claude's context but earlier turns are not shown.
- `/model` lasts until the frontend restarts; then the start command's model applies again.
- `journal-reader/start` and its CLAUDE.md (~/Dev/vmConfig) change to run the frontend and drop "lines under 80 columns" and "under the sidebar"; committed in vmConfig.
