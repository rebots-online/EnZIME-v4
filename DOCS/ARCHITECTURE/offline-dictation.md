# Voice annotations and later transcription

User decision, 2026-10-12: the initial release records voice only; typing is a separate on-canvas annotation mode. Automatic transcription is deferred.

Voice annotations retain their durable annotation ID, article selector, actual audio MIME type, duration and asset reference. They must save, reopen after restart and play offline directly from their article anchor. Failed saves retain the recording for retry.

Later transcription adds editable text to the existing voice annotation while preserving its audio asset and article anchor. It must never be inferred from a voice recording alone or advertised before actual offline speech recognition is verified. Model provisioning is not part of the initial-release flow.

The prior RC-04 experiment is unverified and excluded from the application source/dependencies. Its design remains in Git history; local experimental source is preserved under Admin-Manual/.staging/enzime/deferred-transcription-2026-10-12.
