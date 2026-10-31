# Incident 2026-10-31-A: agent session crash with unpushed master and conflicted working tree

- **Date:** 2026-10-31 (crash after local commit `59accbf` at 18:08:17 −0400; recovery same evening)
- **Repository:** EnZIME-v4 (`github.com/rebots-online/EnZIME-v4`), branch `master`
- **Severity:** High — finished release artifacts and a commit existed only on this workstation; `origin/master` did not match local `master`
- **Status:** Recovered (this record accompanies the recovery commit)
- **Policy authority:** KStore — the authoritative, active policy guide and assistant, which receives every output by hook. Admin-Manual is KStore's backup; anything written to Admin-Manual must be accepted/adopted by KStore. This document is project-level incident evidence feeding that policy chain, not policy itself.

## Summary

A Claude Code agent session crashed mid-turn — the underlying **Bun runtime process itself crashed** — after committing v1.6.55832 analysis work locally (`59accbf`) but **before pushing**. At crash time the working tree was mid-`git-stash-pop` with a conflict on `version.json`, and the session's finished v1.6.55832 Windows/Android release artifacts (~96 MB) plus verification logs were untracked. Consequence: `origin/master` sat one commit behind local `master`, and the only copies of the signed release artifacts were on this workstation. The divergence was discovered only because the operator was told "remote did not match local master" moments before the crash.

## Timeline (America/New_York unless UTC noted)

| When | Event |
|---|---|
| 2026-10-12 17:43 | `ec69f0d` commits `version.json` = 1.13.63942 ("record verified partial build and Android storage failure") |
| 2026-10-12 18:47 | `26db277` moves the project to the **v1.6.55832** line ("drop orphaned LFS pointers; vendor binaries committed as real blobs"); `version.json` is left stale at 1.13.63942 — the real v1.6.55832 state is carried only in the working tree |
| 2026-10-12 22:52 UTC | `4da392b` merges `codex/reader-dyndon` |
| 2026-10-13 06:57 | `4c2590d` README video tour/flowchart — last commit `origin/master` received |
| 2026-10-15 08:31 | v1.6.55832 rebuild performed (working-tree `buildDate`) |
| 2026-10-25 | offline-sync topology + Windows screencast gauntlet design specs authored (`DOCS/superpowers/specs/`) — left untracked |
| 2026-10-31 18:08 | `59accbf` committed locally (multi-domain analysis + four-state remediation architecture) — **never pushed** |
| 2026-10-31 evening | Session builds v1.6.55832 Windows x64 + Android universal artifacts and writes verification logs; manually stashes "pre-existing local edits (pnpm-workspace, version.json)"; `git stash pop` conflicts on `version.json`; session **crashes** mid-resolution |
| 2026-10-31 recovery | This recovery: conflict resolved, all session work committed and pushed, remote SHA verified |

## State found at recovery (evidence)

- `git status`: `UU version.json` (unmerged); untracked: `DOCS/superpowers/`, nine `LOGS/` files, five `dist/` v1.6.55832 files
- local `master` ahead of `origin/master` by exactly 1 commit (`59accbf`); origin at `4c2590d`; no remote-only commits
- `stash@{0}: On master: pre-existing local edits (pnpm-workspace, version.json)` retained (conflicted pop does not drop the stash)
- No `.git/MERGE_HEAD`; conflict markers labeled `Updated upstream` / `Stashed changes` → a **stash-pop conflict**, not a branch merge
- Version drift: HEAD and origin `version.json` said 1.13.63942 (built 2026-10-12) while the active release line, the `dist/` artifacts, and the titles of the two most recent commits were all v1.6.55832 (built 2026-10-15)

## Impact

- `origin/master` ≠ local `master`; the divergence was invisible to anyone working from origin
- Sole copies of the signed v1.6.55832 release artifacts — `enzime-reader-v1.6.55832-android-universal-release-signed.apk` (75,060,920 bytes), `enzime-reader-v1.6.55832-windows-x64.exe` (16,392,704 bytes), `enzime-reader-v1.6.55832-windows-x64-setup.exe` (4,462,116 bytes) — plus build and verification evidence existed only on this workstation; machine failure would have destroyed them
- Repository metadata inconsistency: commits titled v1.6.55832 while the committed `version.json` still read 1.13.63942

## Root causes

0. **Triggering failure: the Bun runtime process itself crashed** mid-turn — an infrastructure death entirely outside the agent's control. This is the reason no boundary other than per-turn commit/push is safe: the agent cannot assume its host process will survive to any later cleanup point.
1. **The turn/session did not end at a clean boundary.** The commit existed but the push had not run. A commit that is not pushed is not published; the commit → push → verify cycle is only safe when it completes within the turn.
2. **Manual, interactive git state manipulation.** Live edits were stashed and popped by hand, leaving the tree mid-operation at crash time. Nothing scripted or resumable recorded the intent, so recovery required archaeology (stash-message inspection, marker-label forensics, commit-date reconstruction).
3. **Manual build steps by a stateless agent.** The build existed only in the session's ephemeral context. Manual builds by stateless agents are useless — they cannot be reproduced, and if the agent crashes, no one can even know what was run or whether it completed. Worse, a *successful* manual build is a curse: it produces a real artifact and a success claim with no procedure attached, and agents invariably declare success without reducing the steps even to written documentation, let alone a reproducible script. The v1.6.55832 artifacts published in this very recovery commit embody that defect — they are real, verified artifacts whose build procedure survives nowhere. Committed scripts are the only durable record of how an artifact came to exist.

## Precipitated operating requirements (authority: KStore; mirrored to Admin-Manual backup)

1. **Commit and push every repository-work turn.** COMMIT the work of every turn, PUSH it before the turn ends and certainly before the session ends, and VERIFY the remote SHA readback. A push reporting "Everything up-to-date" publishes nothing. Never end a session with unpushed commits, uncommitted work, or a conflicted/unmerged working tree.
2. **Builds and release operations run by committed scripts only, never manual agent steps.** Every artifact must carry a reproducible provenance chain (script revision + commit) that survives agent crashes. This applies to successful manual builds too: an artifact whose procedure was never reduced to a committed script is unpublishable debt, no matter how well it works today. A crashed manual build leaves no record of what ran; a crashed scripted build leaves an exact one.
3. **Reconcile before publishing.** Before pushing new work, reconcile local `master` with `origin/master` (fetch; then merge or push) so divergence is resolved deliberately, never discovered after a crash.

## Recovery actions (executed)

1. `git fetch origin` — confirmed local ahead by exactly one commit (`59accbf`), no remote-only commits; the pending push is a fast-forward
2. Credential scan of every file to be published — matches in the design specs are security-design prose only; no literal credentials
3. Resolved `version.json` to the stash side (v1.6.55832, `buildDate` 2026-10-15T08:31:32−04:00) — it matches the `dist/` artifacts and the v1.6.55832 commit titles; `pnpm-workspace.yaml` `esbuild: true` was already present in HEAD
4. `git add version.json` — conflict cleared
5. Committed the resolution together with all untracked session work (v1.6.55832 artifacts, build/verification logs, design specs, this incident record) and pushed `master` to `origin`
6. Verified the remote SHA readback (below)
7. Dropped `stash@{0}` after verifying its content was fully captured by the commit

## Exit verification (mandatory)

- `git push origin master` → `4c2590d..3283fa2  master -> master` (fast-forward; includes previously unpushed `59accbf` and recovery commit `3283fa2`)
- `git rev-parse master` = `git ls-remote origin refs/heads/master` = `3283fa216f0afd9bf9167a0e5af1f6e0d6a24533` → **MATCH VERIFIED — remote equals local**
- GitHub advisory (non-blocking): the 71.58 MB APK exceeds the 50 MB recommended size; under the 100 MB hard limit and consistent with this repo's vendor-binaries-as-real-blobs convention
- `stash@{0}` dropped after verifying its content was fully captured by commit `3283fa2` (`pnpm-workspace.yaml` `esbuild: true` already in HEAD; `version.json` stash side committed)
