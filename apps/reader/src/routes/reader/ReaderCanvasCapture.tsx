import { VoiceRecorderSelfManaged } from "../../components/annotation/VoiceRecorder";
import { useRef } from "react";
import type { AnnotationRecord, SelectorBundle } from "../../services/annotationService";
import { PhotoCaptureSelfManaged } from "../../components/annotation/PhotoCapture";
import { VideoRecorderSelfManaged } from "../../components/annotation/VideoRecorder";
import { saveCanvasDraft, type CanvasSaveSession } from "./canvasPersistence";
export interface ReaderCanvasCaptureProps {
  kind: "photo" | "video" | "voice"; selector: SelectorBundle; draftId: string;
  onSaved: (annotation: AnnotationRecord) => void; onCancel: () => void;
  onBusyChange?: (busy: boolean) => void;
}
export function ReaderCanvasCapture(props: ReaderCanvasCaptureProps) {
  const session = useRef<CanvasSaveSession>({ pending: new Map(), busy: new Set() });
  async function save(capture: { bytes: Uint8Array; mimeType: string; durationMs?: number }) {
    props.onBusyChange?.(true);
    try { props.onSaved(await saveCanvasDraft(session.current, { draftId: props.draftId, kind: props.kind, selector: props.selector, capture })); }
    finally { props.onBusyChange?.(false); }
  }
  return <aside role="dialog" aria-modal="true" aria-label={props.kind === "photo" ? "Capture photograph annotation" : props.kind === "voice" ? "Capture voice annotation" : "Capture video annotation"} style={{ position: "fixed", zIndex: 30, inset: "10vh max(1rem, calc((100vw - 40rem) / 2)) auto", maxHeight: "80vh", overflowY: "auto", padding: "1.5rem", borderRadius: "1rem", background: "var(--ez-color-bg-elevated)", color: "var(--ez-color-text-primary)", boxShadow: "0 1rem 4rem #0008" }}>
    <p>Attached to: {props.selector.textQuote?.exact.slice(0, 120) || "Selected article image"}</p>
    {props.kind === "photo" ? <PhotoCaptureSelfManaged onCapture={save} onDiscard={props.onCancel} /> : props.kind === "voice" ? <><VoiceRecorderSelfManaged onCapture={save} onDiscard={props.onCancel} /><button type="button" onClick={props.onCancel} disabled={session.current.busy.size > 0}>Cancel voice annotation</button></> : <><VideoRecorderSelfManaged onCapture={save} onDiscard={props.onCancel} /><button type="button" onClick={props.onCancel} disabled={session.current.busy.size > 0}>Cancel video annotation</button></>}
  </aside>;
}
