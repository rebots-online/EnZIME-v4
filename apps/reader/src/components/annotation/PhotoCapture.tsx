import { useEffect, useRef, useState } from "react";

export interface PhotoCapture { bytes: Uint8Array; mimeType: "image/jpeg"; width: number; height: number }
export interface PhotoCaptureSelfManagedProps { onCapture: (capture: PhotoCapture) => Promise<void>; onDiscard: () => void }
export function PhotoCaptureSelfManaged({ onCapture, onDiscard }: PhotoCaptureSelfManagedProps) {
  const video = useRef<HTMLVideoElement>(null);
  const stream = useRef<MediaStream | undefined>(undefined);
  const mounted = useRef(false);
  const busy = useRef(false);
  const capture = useRef<PhotoCapture | undefined>(undefined);
  const [state, setState] = useState<"idle" | "camera" | "review" | "saving">("idle");
  const [error, setError] = useState<string>();
  const [url, setUrl] = useState<string>();
  const [failed, setFailed] = useState(false);
  function stop() { stream.current?.getTracks().forEach(track => track.stop()); stream.current = undefined; }
  useEffect(() => { mounted.current = true; return () => { mounted.current = false; stop(); capture.current = undefined; }; }, []);
  useEffect(() => () => { if (url) URL.revokeObjectURL(url); }, [url]);
  useEffect(() => {
    if (state === "camera" && video.current && stream.current) {
      video.current.srcObject = stream.current;
      void video.current.play()?.catch(() => undefined);
    }
  }, [state]);
  async function start() {
    if (busy.current) return;
    busy.current = true; setError(undefined);
    try {
      if (!navigator.mediaDevices?.getUserMedia) throw new Error("Camera capture is unavailable on this device.");
      const next = await navigator.mediaDevices.getUserMedia({ video: { facingMode: { ideal: "environment" }, width: { ideal: 1920, max: 4096 }, height: { ideal: 1080, max: 4096 } }, audio: false });
      if (!mounted.current) { next.getTracks().forEach(track => track.stop()); return; }
      stop(); stream.current = next; setState("camera");
    } catch (cause) { if (mounted.current) setError(`Camera unavailable: ${cause instanceof Error ? cause.message : String(cause)}. Retry or discard this annotation.`); }
    finally { busy.current = false; }
  }
  async function shutter() {
    if (busy.current) return;
    const source = video.current;
    if (!source?.videoWidth || !source.videoHeight) { setError("Camera preview is not ready. Wait a moment and try again."); return; }
    busy.current = true; setError(undefined);
    try {
      const scale = Math.min(1, 4096 / Math.max(source.videoWidth, source.videoHeight));
      const width = Math.max(1, Math.round(source.videoWidth * scale)), height = Math.max(1, Math.round(source.videoHeight * scale));
      const canvas = document.createElement("canvas"); canvas.width = width; canvas.height = height;
      const context = canvas.getContext("2d"); if (!context) throw new Error("Photo encoding is unavailable.");
      context.drawImage(source, 0, 0, width, height);
      const blob = await new Promise<Blob>((resolve, reject) => canvas.toBlob(value => value ? resolve(value) : reject(new Error("JPEG encoding failed.")), "image/jpeg", 0.9));
      if (blob.type !== "image/jpeg" || blob.size > 16 * 1024 * 1024) throw new Error("Photo must be JPEG and smaller than 16 MiB.");
      const bytes = new Uint8Array(await blob.arrayBuffer());
      if (!mounted.current) return;
      capture.current = { bytes, mimeType: "image/jpeg", width, height }; stop(); setUrl(URL.createObjectURL(blob)); setState("review");
    } catch (cause) { if (mounted.current) setError(cause instanceof Error ? cause.message : String(cause)); }
    finally { busy.current = false; }
  }
  async function save() {
    if (busy.current || !capture.current) return;
    busy.current = true; setState("saving"); setError(undefined);
    try {
      await onCapture(capture.current);
      capture.current = undefined;
      if (mounted.current) { setUrl(undefined); setState("idle"); setFailed(false); }
    } catch (cause) { if (mounted.current) { setState("review"); setFailed(true); setError(`Photo was not saved: ${cause instanceof Error ? cause.message : String(cause)}`); } }
    finally { busy.current = false; }
  }
  return <section aria-label="Photograph annotation">
    <h3>Photograph this note</h3>
    {error ? <p role="alert">{error}</p> : null}
    {state === "camera" ? <video ref={video} autoPlay muted playsInline aria-label="Live camera preview" style={{ width: "100%", maxHeight: "45vh" }} /> : null}
    {url ? <img src={url} alt="Captured annotation photograph" style={{ width: "100%", maxHeight: "45vh", objectFit: "contain" }} /> : null}
    <div style={{ display: "flex", flexWrap: "wrap", gap: 8 }}>
      {state === "idle" ? <button type="button" className="reader-chip" onClick={() => void start()}>Start camera</button> : null}
      {state === "camera" ? <button type="button" className="reader-chip" onClick={() => void shutter()}>Take photograph</button> : null}
      {state === "review" ? <><button type="button" className="reader-chip" onClick={() => void save()}>Save photograph</button><button type="button" disabled={failed} onClick={() => { capture.current = undefined; setUrl(undefined); void start(); }}>Retake</button></> : null}
      {state === "saving" ? <p role="status">Saving photograph…</p> : null}
      <button type="button" disabled={state === "saving"} onClick={() => { stop(); onDiscard(); }}>Discard photograph</button>
    </div>
  </section>;
}
