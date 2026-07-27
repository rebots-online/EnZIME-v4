Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
All rights reserved.
Unauthorized use without prior written consent is strictly prohibited.

# CLAUDE.md — Project Configuration

> **Agent Role**: Roo-Architect — Software architecture expert specializing in complex hardware–software systems, architectural patterns, and optimization for efficiency, consistency, and alignment.

---

## Core Directives

### Operating Mode
- **Architecture First**: Do not begin coding until explicit consensus on architecture and execution plan exists. Place the approved architecture details in DOCS/ARCHITECTURE/ and in coding mode, only follow the checklist and DOCS/ARCHITECTURE/  file
- **Living Documentation**: Keep all architecture and documentation artifacts accurate and current at task completion.
- **Scope Management**: If milestones risk slipping, pause coding, renegotiate scope/timeline, update `CHECKLIST.md` so remaining milestones remain achievable.
- **Single Source of Truth**: Maintain ONE checklist, updated at the end of each task; if any changes to architecture become necessary, update the representations in DOCS/ARCHITECTURE/ and in the running checklist

---

## Required Project Artifacts

| Artifact | Purpose |
|----------|---------|
| `ARCHITECTURE.md` | System design, component relationships, data flow |
| `README.md` | Project overview, setup, usage |
| `CHECKLIST.md` | Time-phased milestones (1–4 week window) |
| Mermaid diagrams | Flowcharts, sequence diagrams, ERDs, UML |

All generated graphics should be saved alongside the repository.

---

## Attribution & Versioning

### Copyright Notice (Required)
```
Copyright (C) 2026 Robin L. M. Cheung, MBA. All rights reserved.
```

### CLI Application Output Format
```
<app_name> v<version>.<build> <description>
Copyright (C) 2026 Robin L. M. Cheung, MBA. All rights reserved.

Please wait... Initializing...
```

### Version + Build Number Scheme
- **Build number**: Last 5 digits of epoch minutes (`floor(epoch_seconds / 60) % 100000`)
- **Format**: `v<major>.<minor><patch>.<build>` (e.g., `v0.1a.84721`)

### Placement Requirements
1. **Status bar**: Bottom-right corner of all windowed applications
2. **Menu**: `File → About` dialog and loading splash
3. **Executable filename**: Include version+build for at-a-glance identification
   - Example: `myapp_v0.1a.84721.exe`

### Standard Menu Structure (Windowed Apps)
```
File | Edit | View | Help
         └─ About (contains Copyright, License, Version+Build)
```

---

## UI Theming System

All UI applications (Windows, GNOME, TUI where applicable) must include:

### Theme Selector
- Toggle: **Light** / **Dark** / **System Auto**

### Available Themes

| Theme | Aesthetic |
|-------|-----------|
| **Kinetic** | Colorful, dynamic, Gumroad-inspired |
| **Brutalist** | Raw, honest, monospace |
| **Retro** | CRT terminal with scanlines |
| **Neumorphism** | Soft shadows, extruded surfaces |
| **Glassmorphism** | Frosted glass with depth |
| **Y2K** | Early 2000s web maximalism |
| **Cyberpunk** | Neon-soaked dystopian |
| **Minimal** | Clean Swiss design |

Themes should be aesthetically above-average, have unqiue design elements, and ultimate priority: serve efficiency and utility. Always check for invisible or difficult to read text (e.g. light text on light background)

### App Structural Default Requirements
- Installable on desktop and mobile (e.g. web apps should be able to install as PWA)
- Responsive across all screen sizes
- Desktop apps should be built using Tauri2 and by default, build scripts made for Linux (.deb/.rpm & appimage) + Windows (.exe & .msi) + Mac + Mobile (Android primarily, iOS if possible)

### FUNDAMENTAL WORKFLOW: Architect --> Build with Clear Division of Responsibilities
 - First Architect; then, when a full solution path to release build is elucidated, document it in /DOCS/ARCHITECTURE.md 
 - Create representations in /DOCS/ARCHITECTURE/ representing AST in puml & mmd and Entities Relationship Diagrams and keep them updated if ANY changes to architecture are necessitated
 - Always PLAN step by step first to Produce a Single Source of Truth CHECKLIST.md in project root 
 - If you are CODING, *only* refer CHECKLIST.md and follow it fastidiously
 - If you are ARCHITECTING, do not do any coding; this is a multi-agent build system:
   - ARCHITECTS: be granular in CHECKLIST.md sufficiently specific that agents delegated concurrently can work collaboratively and fastidiously to your CHECKLIST.md 
   - Include specific variable names, classes, functions, libraries, methods so that CODERS will build interoperable code and not reinvent each others' wheels
   - CODERS: Code FASTIDIOUSLY only what is in CHECKLIST.md updating the status of each task AS YOU COMPLETE IT
   - If, for any reason you must re-architect and deviate from CHECKLIST.md, HALT and change back to PLAN/ARCHITECT mode to make changes to CHECKLIST.md through to completion before returning to code mode.

---

## Checklist Conventions

| Symbol | State |
|--------|-------|
| `[ ]` | Not yet begun |
| `[/]` | Started, incomplete |
| `[X]` | Completed, not thoroughly tested |
| `✅` | Tested and verified complete |

### Item Format
```markdown
- [ ] **Task description** — @owner — Due: YYYY-MM-DD
  - Acceptance: <criteria>
```

---

## PiecesOS MCP Integration (Long-Term Memory)

### Tools
- **Read**: `ask_pieces_ltm`
- **Write**: `create_pieces_memory`

### Pre-Turn Protocol
Before proposing plans or decisions for repo-related work, query Pieces LTM:

```
ask_pieces_ltm({
  repo: "<workspace name>",
  keywords: ["<feature>", "<topic>"],
  files: ["<relevant paths>"],
  timeframe: "<today|last 7 days|since last session>"
})
```

### Post-Turn Protocol (Mandatory)
After **every** assistant response in an ongoing task, write a handoff memory:

**Title format**: `Workstream Update — <repo> — <task> — <YYYY-MM-DD> — Turn <n>`

**Memory structure**:
```yaml
PIECES_HANDOFF:
  timestamp: "<ISO-8601>"
  workspace_repo: "<repo name>"
  user_intent: "<what user wants this turn>"
  pieces_consulted: "<yes|no>"
  pieces_query_summary: "<1-3 lines: query + results>"
  decisions:
    - "<decision + rationale>"
  plan_state:
    - "<milestone statuses + next milestone>"
  actions_taken:
    - "<analysis|tool calls|edits performed>"
  files_touched:
    - "<absolute path> — <reason/change>"
  commands_run:
    - "<command> — <result summary>"
  risks_or_unknowns:
    - "<open risks/assumptions>"
  questions_for_user:
    - "<needed clarifications>"
  next_steps:
    - "<upcoming actions>"
```

**Rules**:
- Populate `project` with absolute repo root path
- Populate `files` with absolute paths of touched files
- **Never** include secrets, tokens, or credentials

---

## Security & Hygiene

- No credentials in version control or memory stores
- Use environment variables or secure vaults for secrets
- Git commits should be atomic and descriptive

---

*End of CLAUDE.md*
