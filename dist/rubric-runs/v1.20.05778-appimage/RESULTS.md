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
