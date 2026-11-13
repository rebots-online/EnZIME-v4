# EnZIME Stitch review — 2026-11-13

This is a review of existing design sources, not a claim that generated designs are implemented. `screen-inventory.json` records every returned resource and four individually recovered activation screens. HTML and available screenshots were read. No application tests were run.

## Sources and disposition

| Project | Reviewed material | Retained contribution | Reconciled against product decisions |
|---|---|---|---|
| 9898008898121608042, EnZIME Reader | HomeDashboard; LibraryPackManager; DownloadPlan; ClassicReader; AnnotationWorkspace; NotesBrowser; AIConversation; MemoryLoreBrowser; SetupWizard; SystemSettings; ThinkSpaceNavigator; Three.js and Shader exports | Aurora Glass identity; six destinations; focused reader; contextual annotations; scoped local Ask; reviewable Memory; storage-fit planning; spatial exploration | No healthy-readiness dashboard, terminal launch, enterprise profile, invented pack data or hardware readings. Decorative code is not a capability. |
| 7808089884213645532, Reader/Annotator | GlobalSearch; DeepReaderAINotebook; LibraryHome; Settings; DownloadCatalog and v2; MobileHome; MobileReader; ProPaywall; AISetup; Notebook; ShareSync; PRD brief | Search groups and scope; narrow layouts; notebook/source relationship; catalog; explicit AI setup; sidecar preview and authorship | One-time activation supersedes subscription offers. No mandatory account, uncontracted mesh service or invented model response. Mobile variants are responsive states of the same functions. |
| 10341950726478924497, EnZIME | ReadingAnnotation; Landing; ArchiveLibrary; Navigator; PRD document | Reading/annotation pairing; offline ownership promise; library entry; selected-node detail and dive | The document is an unfilled template, not requirements. Arbitrary format support, fake security claims, coordinates and marketing telemetry do not become scope. Landing concepts inform setup; no new marketing route is required. |
| 9609690129861232556, EnZIME-Lite | Four ReaderOverlay variants; five FieldLibrary variants; SettingsStorage; MyMarks; KiwixCatalog; SynthesisGraph; SynthesisConfigurator; DynamicDepth; SplashHybridUpsell; SplashLogin; prototype | Reader/TOC/notes organization; downloads and retained coverage; storage budget; source-linked marks; graph selection; import/export | Duplicate layouts map to one final surface each. Catalog branding is neutral. Private resource storage, mandatory login, DEFCON/security theatre, emergency bypass and new subscription tiers contradict the product. Graph/synthesis controls map to existing Navigator, Ask and Memory contracts; they do not authorize an additional engine. |
| 13978455034360167310, Reader v5 | Four recovered Activation steps, HTML and screenshots; navigator project thumbnail | Four-step activation; reader/navigator pairing; journey; real preparation progress; established Pysanky visual lineage | Literal price interpolation and fabricated receipt/key cannot ship. Use one consistent Z mark and typography. Navigator telemetry stays only where it explains a real interaction; no false status. |

## Enumeration gap

The v5 project's list_screens endpoint returned an empty object twice. Individually fetching the four IDs recorded in `LIBS/UI/STITCH/activation/STITCH-SCREENS.md` succeeded. The project thumbnail shows another navigator design. It has been visually reviewed, but its screen ID/export has not yet been recovered. The retained native session `92cd0a17-467a-46c2-ae13-5e44c4fce47e` records its generation as background task `k0h2ujmt1`, which timed out without returning a screen ID. The paired Reader generation was declined, not completed. Four later activation responses supplied the recovered IDs. This establishes the known source lineage but does not prove the service has no additional screens. The API enumeration gap remains explicit.

## Current prototype revision

The new project contains drafts for all S01–S17 functions, including separate desktop/mobile Home. Exports under `LIBS/UI/STITCH/product/drafts/` are provenance material, not approved shipping assets. `draft-manifest.json` marks them unfrozen. Generated descriptions have repeatedly claimed removal of sample data while the HTML retained invented content or simulated saves; source and screenshot review, rather than those descriptions, determines disposition. Corrections are in progress before connection and freeze. No application tests or runtime acceptance checks have been run.

Build-time `.env` governs all configurable values, including displayed identity and pricing. Design text is the named EnZIME product's visual reference; implementation binds the configured identity and offer without hard-coded commercial values. Store responses must correspond to that configuration. No administrative key is a client-visible design field.

## Principles applied to every source

Preserve every intended user function, not every generated decoration. Repeated variants are mapped, not silently dropped. A generated example of a new business model, unsupported security promise, private-storage policy or fictional data cannot override explicit product decisions. The final inventory must map all retained functions to actual exported Stitch screens and bind their controls in architecture.

The new prototype project is `17076380137565911053`. Its freeze manifest must distinguish ready, empty, in-progress and actionable-failure states; a screenshot of a happy path alone is not a complete interaction design. Prototype content must not be promoted into production data. No customer-visible placeholder tokens, invented prices, fake receipts or fake knowledge are permitted.
