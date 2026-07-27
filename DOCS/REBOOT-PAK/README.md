# EnZIME Reboot Pak

## Start here

1. Read `SOURCE-GAPS.md`.
2. Complete and verify `VENDOR-MANIFEST.md`.
3. Run `audits/cold-start-audit.md`.
4. Run the Stitch workstation workflow in `STITCH/STITCH-MCP-RUNBOOK.md`.
5. Update remote evidence only from successful Stitch tool results.
6. Rebuild `PAK-MANIFEST.sha256`.
7. Give the coding agent `KICKOFF-PROMPT.md`.

## Authority

| File | Purpose |
|---|---|
| `PRD.md` | complete product intent and acceptance |
| `ARCHITECTURE.md` | complete implementation contract |
| `CHECKLIST.md` | fresh executable transform |
| `KICKOFF-PROMPT.md` | cold-start coding instructions/failure guards |
| `VENDOR-MANIFEST.md` | exact permitted source inputs |
| `SOURCE-GAPS.md` | evidence not available during pak generation |
| `STITCH/` | design tokens, screen manifest, prompts and remote evidence |
| `audits/` | launch audit |

## Closed-world rule

Application implementation may use only:

- this pak;
- the exact verified vendor tree;
- ordinary package registries.

No previous EnZIME repository or conversation is an implementation input.

## Workstation plan copy

The architecture file intended for the optional Windsurf plan location is:

```text
DOCS/REBOOT-PAK/ARCHITECTURE.md
```

On the workstation, after final gap closure:

```bash
install -D -m 0644 \
  DOCS/REBOOT-PAK/ARCHITECTURE.md \
  /home/robin/.windsurf/plans/enzim-complete-architecture-a8a84d.md
```

That workstation copy is convenience only.
The pak-relative architecture remains authoritative.

## Current generation status

The architecture, PRD, fresh checklist, kickoff and Stitch execution contract
were generated and locally validated.

This pak is not cold-start-authorized until:

- unavailable source attachments intended as authority are re-supplied;
- actual vendor trees and digests are included;
- Stitch remote evidence is executed or explicitly waived;
- `SOURCE-GAPS.md` is resolved;
- cold-start audit passes.

