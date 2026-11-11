# Rubric run — EnZIME Reader v1.20.05778 Linux AppImage (in progress)

Rubric: `DOCS/TEST_RUBRIC.md`, "Build-set and read-path iteration — preauthored 2026-11-10".
Artifact: `dist/enzime-reader-v1.20.05778-linux-amd64.AppImage`, SHA-256 in `artifact-sha256.txt`.
Method: launched on a private X display (Xvfb :99, 1400x900) with an isolated profile
(`XDG_DATA_HOME`/`XDG_CONFIG_HOME`/`XDG_CACHE_HOME` under `.tmp/gauntlet-v1.20.05778/`), driven with xdotool,
observed through the numbered screenshots in this folder. No continuous recording (waived for this iteration).
Observer: the orchestrating agent, reading each screenshot. Network: on.

| Row | Result | Observation (screenshot) |
|---|---|---|
| R-SET | observed | `verify-build-set.py dist/current-build-manifest.json --require-complete` exit 0: 1.20.05778 complete, 11/11, Android lane success. |
| R-APK-SIZE | observed | APK 74,508,215 bytes; four `libenzime_reader_lib.so` of 12.9-20.6 MB; symbols ZIP holds four `.dbg` files (246-312 MB) plus `mapping.txt`; provenance records `all-shf-alloc-sections-sha256` for all eight packaged libraries; signer certificate SHA-256 begins `56c13674ef22df95`. |
| R-BOOT | observed | Styled Home; bottom-right label "EnZIME Reader v1.20.05778" (02). |
| R-LIBRARY | observed | Library lists live Kiwix catalog entries; catalog search for "Mathematik" returns three matching packs (03, 13). |
| R-PLAN | observed (one pack) | Zapping Sauvage, 357 MB, budget 0.125 GiB: "plan downloads 80 MB · subset · ranged · FITS", 96 MB kept, 32 MB temp (06, 07). |
| R-DOWNLOAD | observed (one pack) | Job "completed · 80 MB retained locally · Partial pack"; pack listed under Installed packs (10). |
| R-READ | NOT observed | Opening that pack shows `C/index.html` with a blank article area (11, 12). The pack is a video collection whose index page is script-built; the reader blocks scripts. No text article has been seen rendering in this run. |
| R-REOPEN | not yet run | |
| R-OFFLINE | not yet run | |

## Defects seen on this artifact

1. Plan for "Mathematik von Wikipedia" (125 MB, budget 64 MiB) is still "Computing…" after more than 3.5 minutes
   with no progress figure (14, 15). Release standard R3 asks for a plan or visible progress within 3 s.
2. A budget of 0.1 GiB is rejected: "Enter a positive storage budget that resolves to a safe whole byte count" (05).
3. Reader shows a blank page for a script-built main page, with no message saying why (11, 12).
4. Home "Continue reading" shows loading bars indefinitely when there is nothing to continue (02).
5. Installed pack still reads "not opened" after it was opened in the reader (13).
6. Home "Task inbox" text mentions asking a question and running a backup; Ask and Memory remain in the side
   rail (greyed). Neither feature is in this build (02).
7. Home still offers "Open a ZIM file", which the M1 rubric lists as a void path (02).

## Second session, 2026-11-10 (same artifact, same profile; screenshots 18-28)

| Row | Result | Evidence |
|---|---|---|
| R-REOPEN | Observed. After a full stop and restart the installed pack and the completed download are listed, status "Ready" | 19 |
| R-PLAN (text pack) | Observed. "InfluxData Docs" (1.8 MB): plan shown within 45 s of "Compute plan": 157 articles, 19 resources, FITS | 22, 23 |
| R-DOWNLOAD (text pack) | Observed. Completed within 40 s; listed as installed, 157 articles | 24 |
| R-READ | Observed. Main page renders as text; title search for "influx" returns 10 matches; a linked article opens with headings and links | 25, 27, 28 |
| R-OFFLINE | Observed, with a limit. App restarted with every proxy variable pointed at a closed local port. Library shows "Kiwix unavailable (offline); showing cached entries"; both packs listed; text pack opens, title search works, article opens | 26, 27, 28 |

Limit of the offline run: a true no-interface run was not possible without root on this machine (`unshare -rn`,
`bwrap --unshare-net` and user `PrivateNetwork=yes` were all refused or ineffective). The dead-proxy method blocks
the app's HTTP traffic, which the app's own "unavailable (offline)" message confirms, but it is not the same as
airplane mode.

Defects added in this session:

9. Below the end of a short article the reader frame is white while the article area is dark (28).
10. The plan for a 1.8 MB pack states "66 MB kept" and "Final footprint 66 MB" (22, 23).
11. Defect 5 confirmed again: "InfluxData Docs" still reads "not opened" after being opened and after a restart (26).

Still not run: defects 1-4 and 6-7 were not re-tested; no row was run on Android or Windows.
