# EnZIME Reader — to-do ledger

Items are recorded as stated by the operator. Nothing here is acted on until the operator moves it to action.

## 2026-11-10 — Knowledge Pysanky 6-DOF navigator (operator, during build 1.20; version files at 1.19.05769)

**Operator's words:** "I have placed the original tsx artifact of the knowledge pysanky 6-dof navigator in
@VENDOR/knowledge-pysanky/knowledgepysanky-v102.tsx the most faithful implementation of which centres on the
pysanky egg (occasional ~10% Faberge egg) launcher, which is erroneously implemented so the longer the 'e' is held
down, the less energy it is launched with--but the targeting reticle and its 'overcharge glitch' effect is the
central easter egg focus--implement the exact reticle and overcharge glitch effect but with corrected (non-inverse
proportion) energy launch charging effect in the 6-dof navigation which is to be calibrated by semanticity/vector
embed/hypergraph as discussed and decided when we discuss and decide so it can be an accurate representation of the
information originally downloaded in-ZIM and either vector embedded in background gradually over time or some other
means (as we discuss details of implementation when you know the data structure shapes and what would work for the
use case); the 6 dof navigator is intended to assist the user in uncovering otherwise-latent usages or concepts
otherwise un-thought-of amongst the unbounded set of things to compare concepts to"

**Source artifact:** `VENDOR/knowledge-pysanky/knowledgepysanky-v102.tsx` (1,203 lines, header "ThinkSpace — 6DoF
semantic concept navigator (mockup)"), committed as placed.

**What the artifact does today (read 2026-11-10, lines 391-403, 506-513, 555-580, 887-906):**
- Tap E (< 1 s): homing egg to a random node, always arrives in about 22 frames.
- Hold E ≥ 1 s: aimed ballistic egg. `power = min(1,(held-1000)/1400)`; launch speed `(0.35 + 0.9*power) * 3.4`
  px/frame, so within 1.0-2.4 s speed rises with hold time.
- Hold > 2.4 s: `over = (held-2400)/800` (shown capped at 1.4): reticle jitters by `over*6` px. `over > 1`
  (held > 3.2 s) is the glitch: random-hue ring, five random bars, flickering opacity, text alternating
  "⚠ OVERCHARGE" / "█▓▒ T4RG3T_L0$T"; release fires 2-4 eggs scattered ±90 px at random power 0.6-1.4.
- Reticle: dashed line from launcher (W/2, H-30) to cursor, ring radius `16 + 10*power`, hue `90 - 90*power`
  (green to red), white centre dot, four white ticks, eight-segment battery to the upper right.
- Where "longer hold, less energy" shows: a hold just past 1 s launches at the minimum (0.35) and is far weaker
  than a tap; an overcharged release averages below a full charge. The formula itself is not inverted.
- No Fabergé egg exists in this file (no match for "faberg"); every egg is a banded pysanky from a random seed.

**Not decided (operator: "as discussed and decided when we discuss and decide"):** what the corrected charge curve
is; whether tap stays homing; the Fabergé variant's look and its ~10 % rule; where the navigator lives in the app;
how node positions are derived from ZIM content (embedding, hypergraph or other) and when that is computed.
No architecture, checklist task or code exists for this item.

### Addition, same item (operator, minutes later)

**Operator's words:** "the movement within it is also not correct, I just noticed--it should move like a space ship
through 3-space, with the zoom wheel corresponding to a fwd thruster"

**What the artifact does today (header comment and line 603):** dragging empty space pivots/orbits the view about a
vertical hinge in front of the camera; the wheel adds to a forward velocity (`c.vFwd += -e.deltaY*0.05`) that "banks
toward the focused node"; the view recentres itself after 0.9 s idle (`recenterIdle`). So the wheel already feeds a
forward velocity, but steering is an orbit around a point, not the craft's own pitch/yaw/roll, and the idle recentre
and node-banking move the view without input.

**Not decided:** which inputs give pitch, yaw, roll and lateral/vertical thrust; whether velocity persists (drift)
or damps; whether auto-recentre and banking are removed.

### Decision, same item (operator, 2026-11-10)

**Operator's words:** "the navigator is selectable as one of two main viewer modes for the downloaded/generated zim
content"

**Recorded as decided:** the navigator is not a separate route or an easter-egg screen. Reading installed ZIM content
has two main viewer modes the user selects between: the existing article reader and the 6-DOF navigator. This closes
the open point "where the navigator lives in the app".

**Still open:** where the mode switch sits and whether the choice is remembered per pack; what "generated" ZIM content
covers; how selecting a node in the navigator opens its article in the reader mode (the artifact's double-click "dive").

### Decision, same item (operator, 2026-11-10) — mode switch and presentation

**Operator's words:** "it is a toggle. Since it's a USP, it would ideally present either in conjunction with the
traditional reader shape or inset in a windowlet to navigate and swap focus with the main window, like the natal
chart inset windowlet in Natally-ai-studio's main atelier view"

**Recorded as decided:** the mode switch is a toggle. Because the navigator is a selling point, it is visible
together with the reader, not hidden behind it: either alongside the traditional reader layout, or as an inset
windowlet that can be navigated in place and swapped with the main pane (inset becomes main, main becomes inset).
Reference behaviour: the natal chart inset windowlet in the main atelier view of `~/CascadeProjects/natally-ai-studio`.

**Not yet read:** the Natally inset implementation (click/swap, docking, resizing). It is read when this item is
architected, not before.

**Still open:** side-by-side versus inset as the default, or both offered; inset size, position and behaviour on
narrow and Android screens; whether the toggle state is remembered per pack.

## 2026-11-10 — Storage layout: observations on build 1.20.05778 against the fixed architecture

**Operator's words:** "is a single monolithic db going to be a good idea for multiple hundred gb imports?" and, to my
answer, "what do you mean? thats an architectural decision fixed years ago".

**Standing:** the storage design is fixed by `DOCS/ARCHITECTURE.md` §1.2 (replaceable archives never coupled to user
data), §6.1 (one metadata database) and §6.3 (file object store under `<storage-root>/EnZIME/`). It is not open for
redesign. My reply wrongly presented layout points as things "to be decided"; that is withdrawn.

**Observed on the 1.20.05778 AppImage profile (facts only, no action taken):**
- Metadata database is `<app-data>/enzime.db` (0.6 MB after one install); §6.1 names
  `<app-data>/EnZIME/metadata/enzime.sqlite3`.
- Installed pack content is under `<app-data>/packs/version-<hash>/` (`pack-index.sqlite`, `clusters/<n>.bin`,
  `pages/`, `workspace/`); §6.3 names `<storage-root>/EnZIME/packs/<pack-id>/<pack-version-id>/`. The directory
  `EnZIME/packs/` exists and is empty.
- Article and media bytes are in cluster files, not in SQLite. Each pack has its own index database.
- Planning a 125 MB Wikipedia pack ran more than ten minutes without a progress figure.
- Not checked: whether `DOCS/ARCHITECTURE/dyndon-range-acquisition.md` amends §6.1/§6.3 for these paths.
