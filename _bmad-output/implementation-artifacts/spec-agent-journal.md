---
title: 'Agent journal and reader pane'
type: 'feature'
created: '2026-11-20'
status: 'done'
route: 'dispatch'
review_loop_iteration: 0
baseline_commit: '8bbebbc4e2c0db96c3a4f67745e945fa6ef6a53f'
baseline_commit_vmconfig: '16ef2ae2bb95e59d7d4938927d6b648263027bc6' # ~/Dev/vmConfig: stories 1-3 live there
context:
  - '{project-root}/_bmad-output/specs/spec-agent-journal/SPEC.md'
  - '{project-root}/_bmad-output/specs/spec-agent-journal/journal-records.md'
  - '{project-root}/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** What agents did, where, and which versions they changed or saw is buried in 291 MB of per-session transcripts; answering "which version of X did we bump, where, when" means digging.

**Approach:** All four stories of spec-agent-journal (CAP-1..4, CAP-6). 1: a PostToolUse hook in `~/Dev/vmConfig` appends a masked raw record per command and edit to one local journal. 2: its Stop branch starts a detached Haiku 5.5 run that writes a ≤5-line entry per turn that changed something. 3: a reader chat folder in vmConfig (CLAUDE.md, search skill, start script). 4: tabrail hosts that chat in an always-open pane under the sidebar of the current tab, growing while focused.

## Boundaries & Constraints

**Always:** Hooks print nothing, exit 0, raw record < 50 ms; Stop returns at once (`setsid -f`, all fds redirected). Journal `${CLAUDE_JOURNAL_DIR:-~/.local/share/claude-journal}`, outside any repo. Masking (one jq module) runs on every raw field and on the distiller's input and output: `--password x`/`--token=x`/`--api-key x`, `*TOKEN*|*SECRET*|*PASS*|*KEY*|*AUTH*|*CREDENTIAL*=value`, URL userinfo, `Authorization`/`Private-Token`/`X-Api-Key`/`Cookie` headers, `curl -u u:p`, known token shapes (ghp_, github_pat_, glpat-, sk-ant-, xox?-, AKIA, JWT), private key blocks. Env `CLAUDE_JOURNAL=off` turns the journal hook off; the distiller and the reader run with it and with `TMUX_PANE` unset (so `tmux-claude-state` leaves their host tab alone). Reader chat (decided): `claude --model claude-haiku-5-5 --continue`, a fresh one when there is nothing to continue. Reader pane only when global `@tabrail_reader` (its command) is set; hooks spawn nothing otherwise. Growing never writes `@sidebar_width`. Pane markers `@reader`, `@reader_big`, apart from `@sidebar`. Test tmux only on `-L` servers. vmConfig: commit only the new files, on `lde/agent-journal`.

**Never:** Edit `~/.claude/settings.json`, `~/.tmux.conf`/vmConfig `tmux.conf`, or the live `tmux-claude-state` (hand Lynn the changes). Record command output, file contents or diffs. Connect to servers. Prune the journal. Journal the reader's or distiller's own sessions.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected |
|---|---|---|
| version from output | Bash `rpm -q tmux` → `tmux-3.8-1.fc44.x86_64` | raw `versions: ["tmux 3.8"]` |
| secret | `curl -u lde:hunter2 -H 'Authorization: Bearer abc' https://u:p@h`, `TOKEN=abc123 x --password hunter2` | no `hunter2`, `abc`, `abc123`, `p` left; names kept |
| edit elsewhere | Edit of `~/Dev/vmConfig/x` from a tabrail session | folder = session cwd, repo/branch = the file's repo |
| read-only turn | Stop, no raw records since the turn's prompt | no entry, no model call |
| model fails | `claude -p` errors or times out (90 s) | no entry, nothing printed |
| tab switch | reader in tab 1 (big or small), go to tab 3 | reader small under tab 3's sidebar, same pid; tab 1 sidebar alone at shared width |
| close | tab with only sidebar + reader; or menu Close on the reader's tab | reader moves to the tab tmux shows next, then the tab closes |
| reader gone | its pane exited or killed | next tab switch or reap starts it again, not within 10 s of the last start |
| no reader set | `@tabrail_reader` unset | tabrail exactly as before |

</frozen-after-approval>

## Code Map

- `~/Dev/vmConfig/tmux-claude-state` -- style model for hooks (event `case`, `jq` one-pass read, `exit 0`); exits without `TMUX_PANE`. Read only.
- `~/.claude/settings.json:34-115` -- hooks wiring; hand Lynn: PostToolUse matcher `Bash|PowerShell|Edit|Write|MultiEdit|NotebookEdit` and Stop → `~/Dev/vmConfig/claude-journal`, timeout 5.
- Hook input: `session_id`, `transcript_path`, `cwd`, `hook_event_name`, `tool_name`, `tool_input.command|file_path|notebook_path`, `tool_response.stdout|stderr`. Transcript JSONL: turn starts at the last `type=="user"` line with `turnOrigin` set; `assistant` lines hold `text`/`tool_use`; `tool_result` in `user` arrays; ISO `timestamp`.
- `claude -p --model claude-haiku-5-5 --tools "" --strict-mcp-config --no-session-persistence` for the distiller. jq 1.8 (~3 ms), git (~2 ms).
- `src/width.rs:14-15,26-31` -- FORMAT gets `#{P:#{?@reader,#{?@reader_big,big,small},}}`; `big` → reset `seen/pane/moved`, return false; with a reader the column flags are `01100`.
- `src/model.rs:6-11,30` -- add `@reader` to FIELDS, skip those panes like `@sidebar`.
- `src/menu.rs:45` -- Close runs `#{@tabrail_dir}/tabrail-tmux close {id}` (formats expand in run-shell, as `#{client_name}` does). Not `main.rs` (99 lines).
- `tabrail-tmux:39-66` -- `ensure` ends with `tabrail-reader follow`; `reap` treats `@reader` like `@sidebar` and closes via new `close WINDOW` (`tabrail-reader away`, `kill-window`), then `follow`. `restart` stays sidebar-only.
- `tabrail.tmux:22-33` -- hooks `[50]` guarded by `if -F` on `@tabrail_reader` and `#{P:#{@reader}}`.
- `README.md`, `AGENTS.md:22-24` -- reader bullet, optional input `@tabrail_reader`, Files rows; option names list.

## Tasks & Acceptance

**Execution:**
- [x] `vmConfig/journal-mask.jq` -- `def mask` and `def versions` (rpm NEVRA, `name v1.2`, `name version 1.2`, `@`, `==`, `image:tag`; stopwords; ≤20 unique, from command + first 64 KB of output).
- [x] `vmConfig/claude-journal` -- hook: PostToolUse → one JSONL line to `raw/YYYY-MM-DD.jsonl` (contract fields + `epoch`, `what` ≤1000 chars); Stop with records → `setsid -f claude-journal-distill`.
- [x] `vmConfig/claude-journal-distill` -- turn slice (prompt, assistant text, tool inputs, results ≤400 chars, ≤40 KB) + the turn's raw records, masked → Haiku answers `done:/changed:/seen:/pointers:`; render under a header we build (time span, where, branch, session) with transcript path; mask; append under `flock` to `entries/YYYY-MM-DD.md`.
- [x] `vmConfig/claude-journal-test` -- piped hook JSON in a temp journal: matrix rows 1-4, timing, a fake `claude` on PATH for render and no-entry cases.
- [x] `vmConfig/journal-reader/{CLAUDE.md,start,journal-search,.claude/skills/journal-search/SKILL.md,.claude/settings.json}` -- rules (journal first, cite entry headers or record time+session, transcripts only via an entry's pointer, commands instead of connecting, fabasoft-infra for infra commands); search by terms, repo, date range, whole entries.
- [x] `tabrail-reader` -- `follow [SESSION]`, `fit WINDOW`, `away WINDOW`; small 4 rows at `@sidebar_width`, big `min(80, window_width-40)` × 2/3 height; flock; 10 s restart guard (`@tabrail_reader_started`).
- [x] `tabrail.tmux`, `tabrail-tmux`, `src/width.rs`, `src/model.rs`, `src/menu.rs`, `README.md`, `AGENTS.md` -- per Code Map.

**Acceptance Criteria:**
- Given the hook wired in a scratch session, when it runs `rpm -q tmux` and edits a file, then two raw records carry all contract fields, the first with `tmux 3.8`.
- Given a turn bumping a package and opening a PR, when it stops, then within a minute `entries/` holds ≤5 lines naming package, old → new, repo, PR URL and transcript.
- Given a seeded journal, when the reader is asked the two CAP-3 questions, then it answers from the journal citing entries and names a command for what the journal lacks.
- Given the reader in tab 1, when clicked into, then it grows; when the work pane is clicked, every pane is back to its size and `@sidebar_width` never changed; `tabrail-tmux restart` keeps its pid.

## Implementation Notes

- vmConfig committed as 15c13e7 by the implementer (local, not pushed); tabrail changes uncommitted.
- Hook adds `pending/<session>` (first record's epoch) so Stop knows a turn had records; distiller unsets the parent session's `CLAUDECODE`/`CLAUDE_CODE_*` vars.
- Step-03 checks: `claude-journal-test` 18/18; scratch tmux harness rows 6-9 + AC4 pass, resize drift fails (4→9 rows on +10); live AC1 (`claude -p --settings` hooks, temp journal), entry in ~6 s; live AC3 reader run cites records, gives `ssh app01t …` command. Reader folder untrusted until Lynn accepts the trust dialog once.
- Verified in review: a failing Bash call fires `PostToolUseFailure` (`.error`, no `tool_response`), not `PostToolUse`.
- Review patch round (all triage `patch` rows): vmConfig 1b5900e; tabrail adds `tabrail-reader-size` (layout save/restore in `@reader_layout`, fit reapplies size), `tabrail-reader-test` (19 checks, `-L tabrail-rt/-ro`, own-pid menu cleanup), `client-session-changed`/`window-resized` hooks, `bind &` → `tabrail-tmux close`, `dup` uses the tab's own pane. Implementer also updated AGENTS.md's harness line (`-f /dev/null`, own-pid menu files), partly resolving deferred #25.
- Re-verified: build 0 warnings; `claude-journal-test` 63/63 (runs mask, distill, search suites); `tabrail-reader-test` 19/19; live AC1 with PostToolUseFailure wiring records the failed command masked, entry written; live AC3 answers from journal; no test secrets in the journal; live tmux menu files intact.

## Spec Change Log

## Review Triage Log

| # | Source | Finding | Verdict | Evidence | Route |
|---|---|---|---|---|---|
| 1 | VG | mask rules beyond curl/TOKEN/--password/ghp_ unpinned | medium | 15 mutants pass all 18 checks (gap, pre-verified) | patch |
| 2 | VG | `versions` pinned only for rpm NEVRA | medium | `->`, `==`, IPv4 mutants pass (gap) | patch |
| 3 | VG | distiller session/epoch filter never exercised | medium | dropping either select clause passes (gap) | patch |
| 4 | VG | size caps, 400/40 KB cuts, sidechain drop unchecked | low | mutants pass; fix is test-only | patch |
| 5 | VG | journal-search and start's `--continue` untested | medium | dropping `~` normalisation passes (gap) | patch |
| 6 | VG, BH | tmux side verified only by a scratch harness | medium | `cargo test` 0 tests; harness in /tmp (gap) | patch |
| 7 | VG | model.rs reader skip unchecked | low | no capture-pane check (gap) | patch |
| 8 | VG | Duplicate on the reader's tab opens in the reader's folder | medium | right-click → `last-pane` returns keyboard to reader; `dup` reads active pane path | patch |
| 9 | VG other | scratch harness `rm *-%*.conf` wipes every server's menu files | high | glob matches live 463036-* files; they were regenerated at 11:10 | patch (in committed harness) |
| 10 | BH, VG other | resurrect restore brings the reader back unmarked | maybe-false | resurrect saves the pane's child command and all panes; unverified what a restore of the shell-less reader yields; settle with save+restore on a -L server with @resurrect-dir in scratch | defer (medium, unverified) |
| 11 | BH, ECH | secrets missed: `-p X` (docker/mysql/sshpass), X-Auth-Token/JOB-TOKEN, JSON/YAML password, aws_secret_access_key, hvs., MYSQL_PWD; over-masks AUTHOR/KEYCLOAK/SSH_AUTH_SOCK | medium | reproduced by reviewers against journal-mask.jq | patch |
| 12 | BH, ECH | turn bounds read at run time; queued prompt/notification or an Esc-interrupted turn loses its entry | medium | distiller takes last turnOrigin after `sleep 1`, no upper bound; Stop does not fire on interrupt | patch |
| 13 | BH, ECH | `\(.repo // .folder)` shows "" outside repos | low | jq `//` keeps "" | patch |
| 14 | BH | failed commands never recorded | high | verified: failing Bash fires PostToolUseFailure | patch |
| 15 | BH | pending file left after interrupt + SessionEnd | low | harmless file; fix adds a branch | reject |
| 16 | BH | prefix & kills the reader mid-answer | medium | stock `&` kill-window bypasses `close`; same tab always holds the reader | patch |
| 17 | BH, ECH | no client-session-changed hook | medium | switch-client / choose-tree across sessions leaves reader behind | patch |
| 18 | BH | reader settings allow sweeping ~/.claude/projects | low | `additionalDirectories` + `Read(~/.claude/projects/**)` contradict CLAUDE.md rule 3; direct deletion | patch |
| 19 | BH | versions misses `Client Version: v1.31.2`, `version "21.0.5"`; false `Downloaded 12.5` | low | case-sensitive `version`, no quotes, no unit lookahead; direct regex fixes | patch |
| 20 | BH | failing reader start gives no sign | low | needs remain-on-exit + branches | reject |
| 21 | BH | start runs bare `claude`, not withproxy env | low | Lynn's rule: launch claude with withproxy; direct addition | patch |
| 22 | BH | zoomed reader re-zooms another pane after a move | low | uncommon; needs a guard | reject |
| 23 | BH, ECH | `away` on a session's only window: empty next | low | reader dies, restarts via --continue; guard | reject |
| 24 | BH | distiller loads full Claude Code context | low | cost only; flag changes unverified | reject |
| 25 | BH | AGENTS.md lacks reader pitfalls and `-L outer -f /dev/null` | low | agent-context file | defer |
| 26 | BH | `-R` matches terms against field names | low | `tojson` of whole record; direct fix | patch |
| 27 | ECH | next tab without sidebar / join fails → reader killed | low | rare; guard | reject |
| 28 | ECH | flock timeout skips away | low | needs >5 s contention; guard | reject |
| 29 | ECH | close/fit/away with empty WINDOW act on current pane | low | hooks always pass ids; manual misuse only; guard | reject |
| 30 | ECH | dead reader with remain-on-exit | false | remain-on-exit unset in Lynn's tmux.conf (default off) | reject |
| 31 | ECH | reader active when moved: old tab's keyboard may land in sidebar | low | only after a right-click while in reader; guard | reject |
| 32 | ECH | fit skips while zoomed | low | rare; needs unzoom hook | reject |
| 33 | ECH, step-03 | window resize drifts reader off 4 rows | medium | harness: 26x4 → 26x9 on +10 rows | patch |
| 34 | ECH | width.rs expects 01100 when reader is elsewhere | low | only after a sidebar is recreated; guard | reject |
| 35 | ECH | distiller masks after tojson / cuts: escaped quotes leak, cut key block eats the slice | medium | `TOKEN=\"x\"` unmasked in model input (reviewer verified) | patch |
| 36 | ECH | turnOrigin line without timestamp | low | prompt lines always carry timestamps (sampled) | reject |
| 37 | ECH | one malformed raw line aborts jq in distiller and `-R` | low | direct fix `fromjson?` | patch |
| 38 | ECH | header takes first record's branch | low | `.[0]` of the group; direct fix | patch |
| 39 | ECH | hook `D` wrong via symlink/relative path | low | direct fix | patch |
| 40 | ECH | `-r` matches branch/session in header | low | direct fix | patch |
| 41 | ECH | `-R` term starting with `--` breaks jq | low | `--args -- "$@"` works (checked jq 1.8.1) | patch |
| 42 | ECH | menu Close needs `@tabrail_dir` | false | tabrail.tmux:7 sets it before any sidebar or menu file exists | reject |
| 43 | ECH | side-by-side work panes not restored after grow/shrink | high | reviewer: 51\|121 → 1\|117 → 55\|117; violates AC4 | patch |
| 44 | ECH | big width floored at `@sidebar_width` | false | floor only avoids "growing" smaller than small; no bad outcome | reject |

## Design Notes

Entry, 4 lines + blank:
```
### 2026-11-20 14:01–14:07 · ~/Dev/x · lde/bump-serde · session 3f2a1b9c
done: Bumped serde 1.0.209 → 1.0.210 for the CVE fix and opened a PR.
versions: changed serde 1.0.209 → 1.0.210 · seen tmux 3.8
pointers: Cargo.toml · https://github.com/o/x/pull/12 · transcript ~/.claude/projects/-home-lde-Dev-x/3f2a….jsonl
```

## Verification

**Commands:**
- `~/Dev/vmConfig/claude-journal-test` -- all pass, raw record < 50 ms.
- `cargo build --release --offline` -- no warnings.
- `tmux -L test` + `-L outer` harness (AGENTS.md) with `@tabrail_reader 'sleep 99999'`, then once with `journal-reader/start` -- tab switch, grow/shrink, close, restart rows.
- One live distill and one reader `claude -p` on a seeded `CLAUDE_JOURNAL_DIR`.
