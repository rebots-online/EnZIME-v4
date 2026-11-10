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

## 2026-11-10 — Operator's notes on build v1.20.05778

Recorded in the operator's words as they are given. Each note is filed by where the defect lives, not by the device
it was seen on: **All platforms** for anything in the shared app, a platform name only for something that exists on
that platform alone. First observed on `enzime-reader-v1.20.05778-android-universal-release-signed.apk`.

### All platforms

1. **"There is mock/placeholder content 'purchasing is not enabled'"**
   - Background: this build was produced with the billing settings unset (the build manifest records
     `monetization_env` as `unset`), so the app shows a not-enabled notice where purchasing belongs.
   - Standing rule it breaks: no mock or placeholder content, ever; every element is for end-user consumption and
     wired to real inputs (global rules, 2026-11-09). Release standard R1: no control that cannot succeed.
   - **Filing (operator):** "it should be in all versions the same as it is not a platform-dependent matter, but
     then the whole placeholder concept is not allowed for any platform any time except when specifically directed,
     so i would suggest filing it under all platform bugs, not only android". Filed under All platforms. The text is
     in the shared Settings screen (`apps/reader/src/routes/settings/SettingsView.tsx:222`) and the shared billing
     code (`apps/reader/src-tauri/src/billing.rs:101`), so every platform's build of 1.20.05778 carries it.
   - **Rule restated by the operator:** placeholder content is not allowed on any platform at any time, except when
     specifically directed.
   - **Rule (operator):** "Never, ever say 'purchasing is not available' when you're also upselling at every
     opportunity; it is confusing and frictional". The app must never show a purchasing-unavailable message
     alongside prompts to buy. Wherever the app invites a purchase, the purchase works.

### Android only

(none yet)

## 2026-11-10 — Operator instructions after the on-device look at v1.20.05778

1. **Fallbacks (rule).** Operator's words: "Fallbacks are to be functional and in a deliberate, authorized form, not
   ad hoc substituted". A fallback is a working, designed, approved behaviour; it is never a notice or stand-in
   improvised by whoever writes the code.

2. **Work in ThinkSpace Navigator v1.107 before any rewrite.** Operator's words: "ok before rewriting since it's
   already a dog's breakfast, work in the 1107 version pysanky I put in the vendored folder".
   - Source: `VENDOR/knowledge-pysanky/thinkspace-navigatorv1107.tsx` (2,742 lines, header "ThinkSpace Navigator —
     v1.107", one React component `ThinkSpace`). Committed as placed. An identical copy sits untracked at
     `DOCS/thinkspace-navigatorv1107.tsx`; left alone.
   - Read so far (lines 1-40, 447-459, 973-981, 1090-1290, 1872-1903): it has the Fabergé egg (16 % chance, lines
     973/981); the same charge code as v102 (power 0 at one second, `0.35 + 0.9*power`, overcharge scatter at
     random 0.6-1.4); the wheel adds forward velocity; idle recentre is on by default. It also contains its own
     reader tab, journey map, document import, embeddings through OpenAI/Ollama endpoints, Qdrant and Neo4j
     stores, an agent chat, and a seed pool of real Wikipedia topic clusters with en.wikipedia.org URLs.
   - Supersedes v102 as the reference for the reticle, overcharge glitch and eggs.

3. **Chargebee billing, prepper-first.** Operator's words: "add chargebee billing in vibe coder method, remembering
   this is to be a prepper positioned first release--so intended to be full offline functional" and "so the
   entitlements should be a token of some sort not requiring online beyond the one purchase time".
   - Already in the repository (commit `ea2056b`, `DOCS/ARCHITECTURE/billing-and-analytics.md`): Chargebee hosted
     checkout through a Netlify function (`services/billing`), which returns an Ed25519-signed entitlement bound to
     the device; the app verifies it offline against an embedded public key and stores it. Lifetime purchases are
     `perpetual` grants; subscriptions are `term` grants that end at the paid-through date.
   - Not done: Chargebee items and prices, the signing key, the Netlify deployment, and the five build settings.
     Without those the app shows the "purchasing isn't available" notice recorded above.
   - Consequence of the operator's second sentence: a subscription needs the app to go online again at each renewal
     to fetch a new token. Only a one-time purchase gives a token that never needs the network again.
   - **Decided (operator, 2026-11-10):** "there is no renewal", "it's a one-time activation i just said its to be
     offline capable", "offline-FIRST, in fact, being prepper-positioned". The first release sells one-time
     activation only. No subscriptions, no renewal, no call home after the purchase. The two subscription plans in
     `services/billing/catalog.json` (`pro_monthly`, `pro_yearly`) do not belong in this release.
   - **Chargebee site state (read through the Chargebee connector, 2026-11-10):** the site is `rochemedia`. Item
     families present: Natally, RobinsAIWorld, Brainbud, FindMyStuffcam, ReBotsOnline, default (Rochemedia). There
     is no EnZIME family and no EnZIME item. One feature exists, `query` (draft, "LLM Inference Credit"), which is
     not EnZIME's. The connector in this session offers list, retrieve and export only; it has no create action.

## 2026-11-10 — Billing provider direction and RevenueCat build-out

**Operator's words, in order:** "not using netlify"; "if you need a server, tell me what you need"; "we cant afford to
use all these non-scalable 3rd party tools"; "ok stop use our woocommerce" (credentials to be created by the operator;
not yet given; the existing `wordpress-roche-media` record is not it); "in the meantime build out revenuecat if you
can't do chargebee"; "but chargebee has a 250k promo limit but revenuecat only 2500"; "in revenuecat, put it in the
same project as natally and all the other apps, under Robins AI World, and use google play in-app billing as a primary
payment method".

**Chargebee:** nothing created. The connector in this session has no write action. EnZIME must not use Natally's
family, credentials or catalog server.

**RevenueCat — created 2026-11-10 in project "Robin's AI World" (`projaba0bd94`), additive only:**

| Object | Identifier | RevenueCat id |
|---|---|---|
| App, Google Play, package `mba.robin.enzime` | EnZIME Reader (Google Play) | `app3949b838c8` |
| Entitlement | `enzime_pro` — "EnZIME Reader — one-time activation" | `entl90e87dac86` |
| Product, one-time, non-consumable | `enzime_lifetime` | `prod23a256bbd2` |
| Offering (not current) | `enzime_default` — "EnZIME Reader Packages" | `ofrngf028591509` |
| Package | `$rc_lifetime` — "EnZIME Reader Lifetime" | `pkge2371829a34` |

The product is attached to the entitlement and to the package. Naming follows the other apps in that project
(`kintsugi_pro`, `prefetch_lifetime`, `kintsugi_default`). No existing app, entitlement, product or offering in the
project was changed. The older RevenueCat projects named "EnZIME" (`proj4170079b`, test store only, with monthly and
yearly subscriptions) and "EnZIME Mobile" (`projd31373e0`) were read and left as they are.

**Not done, and required before a purchase can happen:**
- The in-app product `enzime_lifetime` does not exist in Google Play Console, and the app is not on Play.
- The Play service-account credentials are not configured on the RevenueCat app (true of every Play app in that
  project).
- No price is set.
- The EnZIME app contains no RevenueCat or Play Billing code. Its billing code talks to the Chargebee function.
- Linux and Windows have no payment path under Play billing.
- How a Play purchase becomes the offline activation token is not designed.

## 2026-11-10 — ThinkSpace Navigator v1.107: Ollama connection (operator observation)

**Operator's words:** "there is a problem in the 1107tsx connecting to ollama localhost or on lan... but it is somewhat
immaterial".

Recorded, not investigated. In v1.107 the embedding and model calls go to an OpenAI-style or Ollama endpoint chosen in
its settings (`/v1/models`, `/api/tags`, `/v1/embeddings`, `/api/embeddings`, lines 299-348). The operator asked
instead for research into on-device options (report to follow in `DOCS/`).

## 2026-11-10 — Local inference requirement (operator)

**Operator's words:** "as a prepp-oriented offline first platform we need to build in the local inference to work with
as many platforms as possible with strong gpu, with no gpu,"

**Recorded as a requirement:** local inference is built into the app and must work on as many platforms and hardware
levels as possible, from machines with a strong GPU to machines with no GPU at all, with no network. The research
report on backends (in progress) is to be organised by that range of hardware as well as by platform.
