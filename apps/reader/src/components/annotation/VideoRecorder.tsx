import { useEffect, useRef, useState } from "react";
import { EzButton } from "./sharedUi";
export interface VideoCapture { readonly bytes: Uint8Array; readonly mimeType: string; readonly durationMs: number }
export interface VideoRecorderSelfManagedProps {
  onCapture: (capture: VideoCapture) => Promise<void>; onStart?: () => void; onStop?: () => void;
  onCommit?: () => void; onDiscard?: () => void; disabled?: boolean;
}
export const MAX_VIDEO_BYTES = 128 * 1024 * 1024;
export const MAX_VIDEO_DURATION_MS = 120_000;
export function pickVideoMimeType(): string | undefined {
  if (typeof MediaRecorder === "undefined" || !MediaRecorder.isTypeSupported) return;
  return ["video/webm;codecs=vp9,opus", "video/webm;codecs=vp8,opus", "video/webm", "video/mp4"].find(type => MediaRecorder.isTypeSupported(type));
}
export function VideoRecorderSelfManaged(props: VideoRecorderSelfManagedProps) {
  const [state, setState] = useState<"idle" | "recording" | "review" | "uploading">("idle");
  const [elapsedMs, setElapsedMs] = useState(0);
  const [playbackUrl, setPlaybackUrl] = useState<string>();
  const [errorText, setErrorText] = useState<string>();
  const preview = useRef<HTMLVideoElement>(null);
  const bundle = useRef<{ stream: MediaStream; recorder: MediaRecorder; chunks: BlobPart[]; bytes: number; started: number } | null>(null);
  const capture = useRef<VideoCapture | null>(null);
  const timer = useRef<ReturnType<typeof setInterval> | undefined>(undefined);
  const busy = useRef(false), mounted = useRef(false);
  function stopTimer() { if (timer.current !== undefined) clearInterval(timer.current); timer.current = undefined; }
  function release() {
    const active = bundle.current; bundle.current = null; stopTimer();
    if (active) { if (active.recorder.state !== "inactive") active.recorder.stop(); active.stream.getTracks().forEach(track => track.stop()); active.chunks.length = 0; }
  }
  useEffect(() => { mounted.current = true; return () => { mounted.current = false; release(); capture.current = null; }; }, []);
  useEffect(() => () => { if (playbackUrl) URL.revokeObjectURL(playbackUrl); }, [playbackUrl]);
  useEffect(() => {
    if (state === "recording" && preview.current && bundle.current) { preview.current.srcObject = bundle.current.stream; void preview.current.play()?.catch(() => undefined); }
  }, [state]);
  async function start() {
    if (busy.current || bundle.current || capture.current) return;
    busy.current = true; setErrorText(undefined);
    let stream: MediaStream | undefined;
    try {
      const mimeType = pickVideoMimeType();
      if (!mimeType || !navigator.mediaDevices?.getUserMedia) throw new Error("Supported camera/microphone recording is unavailable.");
      stream = await navigator.mediaDevices.getUserMedia({ video: { width: { ideal: 1280 }, height: { ideal: 720 } }, audio: true });
      if (!mounted.current) { stream.getTracks().forEach(track => track.stop()); return; }
      const recorder = new MediaRecorder(stream, { mimeType, videoBitsPerSecond: 2_000_000, audioBitsPerSecond: 64_000 });
      const active = { stream, recorder, chunks: [] as BlobPart[], bytes: 0, started: Date.now() };
      bundle.current = active;
      recorder.ondataavailable = event => {
        if (bundle.current !== active || !event.data.size) return;
        if (active.bytes + event.data.size > MAX_VIDEO_BYTES) {
          release(); setState("idle"); setErrorText("Video reached the 128 MiB limit, including the final encoder chunk. Record a shorter clip."); return;
        }
        active.bytes += event.data.size; active.chunks.push(event.data);
        if (active.bytes >= MAX_VIDEO_BYTES - 2 * 1024 * 1024 && recorder.state !== "inactive") recorder.stop();
      };
      recorder.onerror = () => { if (bundle.current === active) { release(); setState("idle"); setErrorText("Camera recording failed. Retry with a shorter clip."); } };
      recorder.onstop = () => {
        if (bundle.current !== active) return;
        stopTimer(); active.stream.getTracks().forEach(track => track.stop());
        const durationMs = Date.now() - active.started;
        void (async () => {
          if (durationMs > MAX_VIDEO_DURATION_MS) throw new Error("Video exceeded two minutes. Record a shorter clip.");
          if (!/^video\/(webm|mp4)(?:;|$)/i.test(recorder.mimeType)) throw new Error("Recorder did not identify a supported video container.");
          const blob = new Blob(active.chunks, { type: recorder.mimeType });
          const bytes = new Uint8Array(await blob.arrayBuffer());
          if (bytes.byteLength > MAX_VIDEO_BYTES) throw new Error("Video exceeds 128 MiB.");
          if (!mounted.current) return;
          capture.current = { bytes, mimeType: recorder.mimeType, durationMs };
          setPlaybackUrl(URL.createObjectURL(blob)); setState("review"); props.onStop?.();
        })().catch(cause => { if (mounted.current) { setErrorText(String(cause)); setState("idle"); } }).finally(() => { active.chunks.length = 0; if (bundle.current === active) bundle.current = null; });
      };
      recorder.start(1000); setState("recording"); setElapsedMs(0); props.onStart?.();
      timer.current = setInterval(() => {
        const elapsed = Date.now() - active.started; setElapsedMs(elapsed);
        if (elapsed >= MAX_VIDEO_DURATION_MS - 1000 && recorder.state !== "inactive") recorder.stop();
      }, 250);
    } catch (cause) { release(); stream?.getTracks().forEach(track => track.stop()); if (mounted.current) setErrorText(`Camera/microphone unavailable: ${cause instanceof Error ? cause.message : String(cause)}`); }
    finally { busy.current = false; }
  }
  function stop() { const recorder = bundle.current?.recorder; if (recorder && recorder.state !== "inactive") recorder.stop(); }
  async function save() {
    if (busy.current || !capture.current) return;
    busy.current = true; setState("uploading"); setErrorText(undefined);
    try { await props.onCapture(capture.current); capture.current = null; if (mounted.current) { setPlaybackUrl(undefined); setElapsedMs(0); setState("idle"); props.onCommit?.(); } }
    catch (cause) { if (mounted.current) { setErrorText(`Upload failed: ${cause instanceof Error ? cause.message : String(cause)}`); setState("review"); } }
    finally { busy.current = false; }
  }
  function discard() { if (busy.current) return; release(); capture.current = null; setPlaybackUrl(undefined); setElapsedMs(0); setState("idle"); setErrorText(undefined); props.onDiscard?.(); }
  return <section aria-label="Video note recorder" data-ez-video-recorder="" style={{ display: "grid", gap: "0.75rem" }}>
    <h3>Video annotation</h3>
    <p role="status">{state === "recording" ? `Recording ${(elapsedMs / 1000).toFixed(1)}s / 120s` : state === "uploading" ? "Saving video…" : state === "review" ? "Review ready" : "Camera and microphone stay on this device."}</p>
    {errorText ? <p role="alert">{errorText}</p> : null}
    {state === "recording" ? <video ref={preview} muted autoPlay playsInline aria-label="Live video preview" style={{ width: "100%", maxHeight: "45vh" }} /> : null}
    {playbackUrl ? <video src={playbackUrl} controls aria-label="Captured video preview" style={{ width: "100%", maxHeight: "45vh" }} /> : null}
    <div style={{ display: "flex", gap: 8 }}>
      {state === "idle" ? <EzButton onClick={() => void start()} disabled={props.disabled} ariaLabel="Start filming">Start filming</EzButton> : null}
      {state === "recording" ? <EzButton onClick={stop} ariaLabel="Stop filming">Stop filming</EzButton> : null}
      {state === "review" || state === "uploading" ? <><EzButton onClick={() => void save()} disabled={state === "uploading"} ariaLabel="Attach film note">Attach film</EzButton><EzButton onClick={discard} disabled={state === "uploading"} ariaLabel="Discard film note">Discard</EzButton></> : null}
    </div>
    <p>Clips are limited to two minutes and 128 MiB.</p>
  </section>;
}
