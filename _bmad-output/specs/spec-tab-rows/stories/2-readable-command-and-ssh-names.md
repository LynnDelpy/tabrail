---
title: 'Readable command and ssh names'
type: 'feature'
created: '2026-11-19'
status: 'done'
baseline_revision: 'ce7b8ccb3c586f6c3bbde5da7116ed15eef551e9'
baseline_commit: 'ce7b8ccb3c586f6c3bbde5da7116ed15eef551e9'
review_loop_iteration: 1
followup_review_recommended: false
context:
  - '{project-root}/_bmad-output/specs/spec-tab-rows/SPEC.md'
  - '{project-root}/AGENTS.md'
warnings:
  - oversized
deferred: []
---

<intent-contract>

## Intent

**Problem:** The ssh alias `ssh='TERM=xterm-256color ssh'` reaches the pane title expanded, so ssh tabs read `TERM=xterm…` and remote tabs cannot be told apart; any command started with `VAR=value` has the same noise.

**Approach:** Strip leading `VAR=value` words from running-command titles (CAP-1). Tabs whose foreground process is `ssh` get their own naming in a new `src/ssh.rs` (CAP-2): the remote's prompt title gives `user@host folder`, vim's title gives `vim file`, the local command line still in the title gives the destination host, else `ssh`.

## Boundaries & Constraints

**Always:** Hand-renamed tabs verbatim. Work from `pane_title` and `pane_current_command` only (no new tmux fields, no `/proc`, no spawning). A `user@host:path` title is remote only while `pane_current_command` is `ssh` and its host is not the local `host_short`. Host = destination without user, `ssh://`, `:port` and domain; an IP address stays whole; remote prompt hosts shortened the same way. Folder = last path component, `~` for home, `/` for root. `src/name.rs` and `src/ssh.rs` each under 100 lines.

**Never:** Edit `draw.rs`, `drag.rs`, `input.rs`, `menu.rs`, `control.rs`, `tabrail.tmux`; change Claude topic names or idle folder·branch names; touch remote hosts or the user's default tmux socket; ssh to a real host.

## I/O & Edge-Case Matrix

| Scenario | Input (command, title) | Name shown | Error Handling |
|----------|------------------------|------------|----------------|
| CAP-1 | `htop`, `TERM=xterm-256color htop` | `htop` | No error expected |
| CAP-1 several | `make`, `FOO=1 BAR=2 make test` | `make test` | No error expected |
| Quoted value | `make`, `FOO='a b' make` | `make` | No error expected |
| Only assignments | `zsh` running, `FOO=1` | `FOO=1` | Nothing left: keep title |
| Renamed | window renamed `TERM=x foo` | `TERM=x foo` | No error expected |
| ssh, no remote title | `ssh`, `TERM=xterm-256color ssh app01` | `app01` | No error expected |
| ssh options | `ssh -p 2222 -t lde@app01.example.ch` / `ssh -i k 10.0.0.5` / `ssh ssh://lde@app01:2222` | `app01` / `10.0.0.5` / `app01` | No destination: `ssh` |
| Remote prompt | `lde@app01: ~/src/infra`, `lde@app01:~`, `root@app01.example.ch:/etc/nginx`, `lde@app01:/` | `lde@app01 infra`, `lde@app01 ~`, `root@app01 nginx`, `lde@app01 /` | No error expected |
| Remote vim | `nginx.conf (/etc/nginx) - VIM`, `nginx.conf + (/etc/nginx) - VIM`, `x.rs (~/p) - NVIM` | `vim nginx.conf`, `vim nginx.conf`, `nvim x.rs` | No error expected |
| Local leftovers | `ssh`, `lde@f4l002:~/Dev` / empty / `f4l002` (local host) | `ssh` | No error expected |
| Other remote title | `ssh`, `htop` / `FOO=1 make` | `htop` / `make` | No error expected |
| ssh exited | `zsh` idle, any title | folder·branch as before | No error expected |

</intent-contract>

## Code Map

- `src/name.rs:18-21` -- `label` returns `window` for renamed tabs; the ssh branch goes right after, before the Claude branch (foreground `ssh` beats a stale `@claude`), and passes `f.title` untrimmed: line 24 strips every leading non-alphanumeric, which would eat vim's `[No Name]` bracket, so ssh.rs strips only Claude's glyphs itself.
- `src/name.rs:31-32` -- running-command branch returns `title` when `told`; wrap in the new `bare`. `told` (line 25-26) already rejects `@`+`:` titles and the local host for non-ssh tabs; keep it.
- `src/name.rs:1-3` -- module doc lists naming sources; list them in `label`'s order (hand name, ssh via ssh.rs, Claude topic, running command without `VAR=value`, idle folder and branch).
- `src/model.rs:37-40` -- read-only: `Facts` gets `command` = `pane_current_command` (f[17]), `host` = `host_short` (f[20]).
- `src/main.rs:5-11` -- `mod` list, alphabetical: add `mod ssh;`.
- `src/main.rs:84` -- read-only: loop wakes every second, so new titles show within ~1 s.
- `README.md:11,17,56` -- example row `ssh app01 ⏱ 4m` (becomes `lde@app01 infra ⏱ 4m`, comment column 36), tab-name bullet, Files table `src/` cell (add `ssh.rs` ssh tab names).
- `~/.oh-my-zsh/lib/termsupport.zsh:108-110,49` -- read-only evidence: preexec title is `%100>...>$2%<<` (alias-expanded line); precmd idle title `%n@%m:%~`. `~/Dev/vmConfig/tmux.zsh` sends OSC 133.

## Tasks & Acceptance

**Execution:**
- [x] `src/name.rs` -- add `pub fn bare(line: &str) -> &str` (skip leading `NAME=value` words, NAME `[A-Za-z_][A-Za-z0-9_]*`, value up to the first space outside `'`/`"` quotes; nothing left: return input); use it in the running-command branch; route `f.command == "ssh"` to `ssh::label`.
- [x] `src/ssh.rs` (new) -- `pub fn label(title, local_host) -> String`. First drop leading whitespace and Claude's glyphs (`✳`, braille U+2800-U+28FF) from `title`, keeping any other leading character (vim's `[No Name]`). On that title, try the remote prompt, then the editor title `FILE[ FLAG][ (DIR)][ (N of M)] - APP`: APP is `vim`, `gvim` or `nvim` in any case, optionally followed by digits (real titles: `- VIM`, `- VIM1`, `- Nvim`), shown lower case without the digits; file = text before the first ` (`, a trailing flag word of `+`/`-`/`=` dropped, basename; any other ` - WORD` is not an editor. Only then, on `bare(title)`: a first word whose basename is `ssh` (destination: first argument that is not an option or the value of one of `BbcDEeFIiJLlmOoPpQRSWw`, attached or next word), then local leftovers (empty, local host, local prompt, `Claude Code`) -> `ssh`, else `bare(title)`.
- [x] `src/main.rs` -- `mod ssh;`.
- [x] `README.md` -- example row, tab-name bullet (`VAR=value` dropped; ssh tabs: remote `user@host folder`, app and file, or host), Files table.
- [x] Scratch unit test (scratchpad, not committed) -- `rustc --test` crate including `src/name.rs` and `src/ssh.rs` via `#[path]`; one assert per I/O matrix row, plus titles captured from vim 9.2 / nvim 0.12.6 and Claude through ssh: `a.conf (/etc/nginx) (1 of 2) - VIM` and `a.conf + (/etc/nginx) (2 of 3) - VIM` -> `vim a.conf`, `x.rs (~/p) - Nvim` -> `nvim x.rs`, `x.rs (~/p) - VIM1` -> `vim x.rs`, `[No Name] - VIM` -> `vim [No Name]`, `a=b.txt (/etc) - VIM` -> `vim a=b.txt`, `sudo -i - SU` -> `sudo -i - SU`, `⠂ Fix login redirect` -> `Fix login redirect`, `✳ Claude Code` -> `ssh`.
- [x] Throwaway tmux harness (scratchpad) -- stand-in `ssh` compiled from C (argv0 `ssh`, prints OSC 2 titles read from stdin lines, exits on `exit`/EOF), zsh with ZDOTDIR sourcing termsupport.zsh, tmux.zsh and the alias; check the ACs with `capture-pane`.

**Acceptance Criteria:**
- Given a throwaway tmux server running tabrail with the stand-in ssh first in PATH, when `ssh app01` is typed in an Infra tab, then its row reads `app01` before the stand-in sends a title, `lde@app01 infra` after it sends `lde@app01: ~/src/infra`, `lde@app01 ~` after `lde@app01: ~`, `vim nginx.conf` after `nginx.conf (/etc/nginx) - VIM`, and the local folder name after `exit`.
- Given that server with groups Website, Infra, Notes (folded), a hand-renamed Claude tab, an idle Claude tab on branch `release-2` and a failed `(exit 2)` tab, when the outer `capture-pane -p` reads the sidebar, then it matches the after-example of `row-layout.md` row for row, status times aside.
- Given that server, when `TERM=xterm-256color htop` and `FOO=1 BAR=2 make test` run (stand-in programs), then their rows read `htop` and `make test`; a tab renamed by hand to `TERM=x foo` reads `TERM=x foo`.

## Spec Change Log

- **Loop 1 (review of ce7b8cc + working tree).** Triggers: R2/R4/R6/R7/R17 (ssh.rs `app` takes any all-uppercase ` - APP`: real nvim 0.12.6 titles end ` - Nvim`, gvim's ` - VIM1`, and `sudo -i - SU` became `su sudo -i`), R3/R5 (`app` cut at the last ` (`: vim's `a.conf (/etc/nginx) (1 of 2) - VIM` became `vim nginx)`), R13/R25 (ssh path kept Claude's spinner, so a remote Claude topic flickered, and showed `Claude Code`), R22 (`bare` ran before `prompt`/`app`, so `a=b.txt (/etc) - VIM` became `vim etc)`), R16 (module doc listed ssh after Claude). Amended: Code Map name.rs entries, Tasks ssh.rs and scratch-test lines, Design Notes, Verification. Known-bad states avoided: the five outputs above. KEEP: `bare`/`assignment` in name.rs as they were (quote tracking, NAME check, input returned when nothing is left); the ssh branch right after the renamed check with its "before Claude" comment; ssh.rs `prompt`, `folder`, `destination` with `VALUED` and the bundle rule (`-vp 2222`, `-p2222`), `host` with the `[v6]:port`/bare-v6/IPv4 handling; `mod ssh;`; README changes as they were; the scratch crate's 14 cases calling `name::label` through `Facts`.

## Review Triage Log

| # | Source | Finding | Verdict | Route | Evidence |
|---|---|---|---|---|---|
| R1 | verification-gap | `bare` (CAP-1) pinned by no test that runs | low | defer (DW-5) | Pre-verified: `cargo test` runs 0 tests, the scratch crate lives outside the repo; first committed test is the owner's call (DW-3). |
| R2 | verification-gap | ssh naming pinned by no test that runs | low | defer (DW-5) | Same root cause as R1. |
| R3 | verification-gap | vim with several files gives `vim nginx)` | medium | bad_spec | `ssh.rs:52` `rsplit_once(" (")` cuts at `(1 of 2)`; vim 9.2 and nvim's titlestring (`%t%( %M%)%( (dir)%)%a`) put the arg count after the dir. |
| R4 | verification-gap | nvim titles (` - Nvim`) not recognised | medium | bad_spec | `strings /sbin/nvim`: `%t%( %M%)%( (%{expand('%:p:~:h')})%)%a - Nvim`; `ssh.rs:49` requires all-uppercase, as Tasks said. |
| R5 | blind | vim file-count suffix | medium | bad_spec | Same defect as R3. |
| R6 | blind | nvim ` - Nvim`, gvim ` - VIM1` rejected | medium | bad_spec | Same defect as R4; vim's NAME is v:servername or `VIM`. |
| R7 | blind | any ` - UPPERCASE` title rewritten (`sudo -i - SU` -> `su sudo -i`) | low | bad_spec | Traced `app`: head `sudo -i`, no ` (`, flag check fails, so `su sudo -i`. Same defect as R4 (app found by suffix casing). |
| R8 | blind | `env`/`exec`/`time`/`command`/`\ssh` first word shows the raw command line | low | reject | Real but rare interactively, and only until the remote titles; fix adds a wrapper-word list. |
| R9 | blind | mosh, autossh, `sudo ssh` never reach `ssh::label` | low | reject | Intent scopes ssh naming to foreground `ssh`; those tabs behave as before this story. |
| R10 | blind | host lost once the remote runs a command (`htop`) | low | reject | Frozen matrix row "Other remote title: `htop` -> `htop`" sets this. |
| R11 | blind | domain cut makes `web1.staging…` and `web1.prod…` both `web1` | low | reject | Frozen intent: "Host = destination without … domain". |
| R12 | blind | status column still uses a stale `@claude` | low | defer (DW-6) | `model::status` reads `f[7]` first, unchanged by this story; `@claude` is unset on SessionEnd (vmConfig/tmux-claude-state:16), stale only after a crash or with Claude suspended. |
| R13 | blind | remote Claude over ssh keeps its spinner (flickers) and shows `Claude Code` | low | bad_spec | ssh path gets `f.title.trim()` (Code Map), so `⠂ Topic` is shown as is; the old path stripped it. Fix is a one-line trim. |
| R14 | blind | 100-char oh-my-zsh cut gives `...` as host | low | reject | `host("...")` passes the IPv4 test, real; needs an ssh line over 100 chars, and the fix adds a guard. |
| R15 | blind | `bare` misreads `FOO=a\ b`, leaves `X+=1` | low | reject | Real, rare; fix adds escape state to the scanner. |
| R16 | blind | name.rs module doc lists ssh after Claude | low | bad_spec | Doc reads as precedence ("rename wins; …"), `label` checks ssh before Claude; direct doc fix. |
| R17 | blind | scratch fixtures made up (`- NVIM`), not captured | low | bad_spec | Why R3/R4 passed; grouped with R4: the amended scratch test uses titles from vim 9.2 / nvim 0.12.6. |
| R18 | edge-case | quoted `-o 'ProxyCommand=ssh -W %h:%p jump'` gives `jump'` | low | reject | Traced `split_whitespace`; rare (ProxyJump/ssh_config are the norm), fix needs shell-word splitting. |
| R19 | edge-case | truncated `lde@a...` gives `a` | low | reject | Same as R14. |
| R20 | edge-case | precommand words (`\ssh`, `exec ssh`, `noglob ssh`) | low | reject | Same as R8. |
| R21 | edge-case | `FOO=$(git rev-parse HEAD) make` gives `rev-parse HEAD) make` | low | reject | Same as R15: value grammar is spec'd as up to the first unquoted space; fix adds nesting state. |
| R22 | edge-case | `bare` before `prompt`/`app`: `a=b.txt (/etc) - VIM` -> `vim etc)` | low | bad_spec | Tasks said "on `bare(title)`, try remote prompt, then …". ssh.rs half is a reorder; name.rs local half rejected (telling app titles from command lines there needs a new guard). |
| R23 | edge-case | Neovim ` - Nvim` | medium | bad_spec | Same as R4. |
| R24 | edge-case | mosh-client/autossh/sshpass | low | reject | Same as R9. |
| R25 | edge-case | spinner strip and `Claude Code` filter no longer apply to ssh tabs | low | bad_spec | Same as R13. |
| R26 | verification-gap (loop 2) | naming pinned by no test `cargo test` runs | low | carried defer (DW-5) | carried: same claim as R1/R2; DW-5 open, the cases are listed in this spec's I/O matrix and Tasks, so they outlive the scratchpad. |
| R27 | verification-gap (loop 2) | scratch crate leaves `host` v6 arms, local FQDN leftover and `prompt`'s whitespace guard unpinned | low | patch | Pre-verified: reviewer mutated each branch and all 16 cases still passed; add `[::1]:2222`, `::1`, `lde@[fe80::1]`, `f4l002.example.ch`, `ssh lde@app01 -L 8080:localhost:80`. |
| R28 | edge-case (loop 2) | quoted ssh args (`-o 'ProxyCommand …'`, `'app01'`) | low | carried reject | carried: R18. |
| R29 | edge-case (loop 2) | precommand words before ssh | low | carried reject | carried: R8/R20. |
| R30 | edge-case (loop 2) | 100-char title cut gives `...` or a cut host | low | carried reject | carried: R14/R19. |
| R31 | edge-case (loop 2) | file name containing ` (` cut there (`my file (1).txt (/tmp) - VIM` -> `vim my file`) | low | reject | Traced `editor`: real. Cutting at the last ` (` instead breaks on a dir containing ` (`; either way a rare name on a server loses, and the right-side parse needs an `(N of M)` pattern. |
| R32 | edge-case (loop 2) | local `a=b.txt (/etc) - VIM` -> `(/etc) - VIM` | low | carried reject | carried: R22 local half. |
| R33 | edge-case (loop 2) | `bare` and `$(…)`, backticks, `\ ` | low | carried reject | carried: R15/R21. |
| R34 | edge-case (loop 2) | mosh-client, autossh, sshpass, et | low | carried reject | carried: R9/R24. |
| R35 | blind (loop 2) | local vim/nvim titles shown raw while ssh gets `vim file` | low | defer (DW-7) | Real (name.rs running branch has no editor parse) but local naming is unchanged by this story; `ssh::editor` could serve both, which would also settle R22's local half. |
| R36 | blind (loop 2) | `editor` cuts at the first ` (` | low | reject | Same as R31. |
| R37 | blind (loop 2) | any title ending ` - Vim`/` - Nvim` is rewritten (`Configure editor - Vim`) | low | reject | Real; needs a remote title ending in a vim name, and the fix adds a layout guard (dir group or `[No Name]`). |
| R38 | blind (loop 2) | `bare` ends a value at `' '` only, ssh.rs splits on any whitespace | low | patch | Traced: `TERM=xterm-256color\tssh -p 2222 lde@app01.example.ch` -> `-p 2222 lde@app01.example.ch`. Rare (zsh completes on Tab) but the fix is `c.is_whitespace()`. |
| R39 | blind (loop 2) | two rule sets for leftovers and leading glyphs (name.rs `told` vs ssh.rs `local`/`claude`) | low | reject | A different Claude glyph would be stripped locally but shown over ssh; unverified that Claude uses any outside `✳`/braille, and the fix is a shared helper (refactor). |
| R40 | blind (loop 2) | `user@` costs 4 of ~18 columns | low | reject | Frozen intent: remote prompt gives `user@host folder`. |
| R41 | blind (loop 2) | README bullet says "the app and its file" though only vim/gvim/nvim are parsed | low | patch | README.md:18-19 vs `editor`'s `matches!`; direct wording fix. |
| R42 | blind (loop 2) | no committed test pins the parsers | low | carried defer (DW-5) | carried: R1/R2. |

## Design Notes

Title-based, not `/proc/<tpgid>/cmdline`: oh-my-zsh puts the expanded command line in the title before ssh starts, the SPEC asks to work from titles, and a destination is only needed until the remote sends one. Cost: a shell that sets no running title shows `ssh` until the remote reports.

Editor titles are vim's and nvim's defaults, not a guessed shape: nvim 0.12.6 titlestring `%t%( %M%)%( (%{expand('%:p:~:h')})%)%a - Nvim`, vim's ` - NAME` with NAME = v:servername or `VIM`; `%a` (`(1 of 2)`) follows the dir group, hence the first ` (`. Any other ` - WORD` title is left alone rather than guessed at.

No committed tests: the repo has none and adding the first is the owner's call (DW-3); the scratch `rustc --test` crate covers the parsing edges, the harness the surface.

## Verification

**Commands:**
- `cargo build --release --offline` -- expected: no warnings.
- `rustc --edition 2021 --test <scratch>/names.rs -o <scratch>/names && <scratch>/names` -- expected: all I/O rows and the captured-title rows pass.
- `tmux -L outer capture-pane -p` (inner `tmux -L <test>` attached) -- expected: ACs above.

**Manual checks (if no CLI):**
- Kill both test servers and remove their menu files in `$XDG_RUNTIME_DIR/tabrail`; never touch the default socket.
