# Journal records

What the two kinds of journal record carry: the contract between the hooks that write them (CAP-1, CAP-2) and the reader chat (CAP-3). One record per line or block, greppable; encoding is the build's choice.

## Raw record (one per command or edit, CAP-1)

| Field | Holds |
|---|---|
| time | When the tool call finished, local time with date |
| session | Claude Code session id, so the transcript can be found |
| folder | The session's working folder |
| repo, branch | Git top-level folder and branch, when inside a repo |
| kind | `command` or `edit` |
| what | The command line, or the edited file's path |
| versions | `name version` pairs found in the command or its output (`tmux 3.8`, `serde 1.0.210`); empty when none |

Never the command's output, the file's contents, or the diff. Secrets in `what` (tokens, passwords, keys, credentials in arguments, `VAR=value`, URL userinfo, auth headers) are masked before writing; the same holds for every field of a distilled entry.

## Distilled entry (one per turn that changed something, CAP-2)

| Field | Holds |
|---|---|
| time | The turn's start and end |
| session, transcript | Session id and transcript path |
| where | Repo or folder, branch |
| done | One to three lines: what was done and why |
| versions changed | `name old → new` per bump, install or deploy |
| versions seen | `name version` the turn learned about without changing it |
| pointers | Files, commits, PR URLs, hosts named, commands handed to the user |

At most 5 lines when rendered; anything longer is left to the pointers.
