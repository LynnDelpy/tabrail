---
title: 'Permission prompts, Esc, /clear and /model'
type: 'feature'
created: '2026-11-23'
status: 'ready-for-dev'
route: 'dispatch'
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/specs/spec-reader-bar/SPEC.md'
  - '{project-root}/_bmad-output/specs/spec-reader-bar/stack.md'
  - '{project-root}/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** After story 2, `tabrail-chat` auto-denies every permission prompt, cannot stop a running answer, and has no way to start over or change model (SPEC CAP-7, CAP-8, Esc assumption).

**Approach:** Use Claude Code's stream-json control protocol (stack.md, verified on 2.1.323): answer `can_use_tool` from a one-line question in the bar, send `interrupt` on Esc and `set_model` on `/model`, and restart claude without continuing for `/clear`.

## Boundaries & Constraints

**Always:**
- Claude starts with `--permission-prompt-tool stdio` instead of `--permission-prompts none`.
- `can_use_tool`: the input row becomes `? <icon> <tool> <arg>  y/n` (story 2's tool-line form), bold; `y` replies `{"behavior":"allow","updatedInput":<the request's input, unchanged>}`, `n` or Esc replies `{"behavior":"deny","message":"denied in the bar"}`, both under the request's `request_id`; typing other keys does nothing while it asks; a second request waits in a queue behind the first. The answer is kept as a dim tool line (`✗` prefix when denied).
- Esc with a turn running and no question pending: `interrupt` control request; the turn's end shows dim `interrupted`.
- Replies to our control requests are matched by `request_id`, never by order; a reply with `subtype":"error"` shows one dim line with its `error`.
- `/model <name>`: `set_model` with `<name>` as typed; success shows dim `model <reply's model>`. `/model` without a name shows the current one if known.
- `/clear`: claude is ended (stdin closed, then killed after 2 s) and started again with the start options minus `--continue`, `-c`, `--resume <id>`, `-r <id>`; the conversation view is emptied; the last session id is forgotten.
- Any other line starting with `/`: dim `/x not supported here`, nothing sent. `//text` sends `/text`.

**Never:** Add other slash commands; write permission rules to settings (`permission_suggestions` is ignored); auto-allow anything; touch the sidebar, `tabrail-reader*` or `~/Dev/vmConfig`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Allow | fake asks `can_use_tool` Bash `uname -r` | `? ⚙ Bash uname -r  y/n`; `y` → reply allow with input; dim `⚙ Bash uname -r` | N/A |
| Deny | same, `n` | reply deny; dim `✗ ⚙ Bash uname -r`; turn continues | N/A |
| Two at once | two requests | asked one after the other, each answered under its own id | N/A |
| Esc | turn streaming | `interrupt` sent; `interrupted` after the turn ends | no turn: nothing |
| /model | `/model sonnet` | `set_model` `sonnet`; dim `model claude-sonnet-5-5` | error reply shown |
| /clear | conversation on screen | view empty; fake restarted without `--continue` | N/A |
| Unknown | `/compact` | dim `/compact not supported here`; fake receives nothing | N/A |
| Out of order | replies r3 before r2 | each matched to its request | N/A |

</frozen-after-approval>

## Code Map

- `src/chat/claude.rs` (story 2) -- start flags, control writes, reader events; add `can_use_tool` and `control_response` events, request ids, restart with options filtered.
- `src/chat/keys.rs` (story 2) -- Esc already parsed alone vs as a sequence prefix; `y`/`n` are plain keys.
- `src/chat/view.rs` (story 2) -- the input row and dim info lines.
- `src/chat/main.rs` (story 2) -- slash commands parsed where Enter sends; a pending-question queue.
- `src/chat/json.rs` (story 2) -- re-serialise `input` unchanged: keep the raw text of that field or write the value back out.
- `tabrail-chat-test` (story 2) -- the fake claude gains a `can_use_tool` scenario, logs every received line for checks, answers `set_model`/`interrupt`.
- `stack.md` -- the protocol table; reply shape verified with a coproc probe (allow ran the tool and the turn finished).

## Tasks & Acceptance

**Execution:**
- [ ] `src/chat/claude.rs` -- stdio prompt flag, request ids, control events, `/clear` restart.
- [ ] `src/chat/main.rs` -- prompt queue and y/n/Esc handling; slash commands.
- [ ] `src/chat/view.rs` -- question row; info lines.
- [ ] `src/chat/json.rs` -- whatever re-serialising `input` needs.
- [ ] `tabrail-chat-test` -- one check per I/O row; stay under 100 lines (split the fake into its own file under the test if needed).
- [ ] `README.md` -- tabrail-chat line: y/n prompts, Esc, `/clear`, `/model`.

**Acceptance Criteria:**
- Given a real claude in a scratch folder (`--permission-mode manual --no-session-persistence --model claude-haiku-5-5`) asked "Use Bash to run: uname -r > u.txt", when `y` is pressed at the prompt, then `u.txt` holds the kernel release and the turn ends.
- Given the same with `n`, then no `u.txt` and the answer says it was denied.
- Given `tabrail-chat-test`, then every line is `ok`.

## Implementation Notes

## Spec Change Log

## Review Triage Log

## Verification

**Commands:**
- `cargo build --release --offline && wc -l src/chat/*.rs tabrail-chat-test` -- expected: builds, every file under 100 lines.
- `./tabrail-chat-test` -- expected: all `ok`, exit 0.

**Manual checks (if no CLI):**
- The two real-claude acceptance runs, once each, in `tmux -L chat-try -f /dev/null`, then `kill-server`.
