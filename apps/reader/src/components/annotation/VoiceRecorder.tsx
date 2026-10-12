import type { JSX } from "react";
import {
  useCallback,
  useEffect,
  useId,
  useRef,
  useState,
  type CSSProperties,
} from "react";
import { EzButton, EzIconButton, InlineStatus, useReducedMotion } from "./sharedUi";

export interface VoiceRecorderProps {
  state: "idle" | "recording" | "review" | "uploading";
  elapsedMs: number;
  playbackUrl?: string;
  onStart: () => void;
  onStop: () => void;
  onCommit: () => void;
  onDiscard: () => void;
}

type InternalState = "idle" | "recording" | "review" | "uploading";

interface RecorderBundle {
  readonly stream: MediaStream;
  readonly recorder: MediaRecorder;
  readonly chunks: BlobPart[];
}

export interface VoiceCapture { bytes: Uint8Array; durationMs: number; mimeType: string }
export const MAX_VOICE_BYTES = 32 * 1024 * 1024;
export function pickVoiceMimeType(): string | undefined {
  if (typeof MediaRecorder === "undefined" || !MediaRecorder.isTypeSupported) return;
  return ["audio/ogg;codecs=opus", "audio/webm;codecs=opus"].find(type => MediaRecorder.isTypeSupported(type));
}

export interface VoiceRecorderSelfManagedProps {
  /** Receives a notification once the recording is uploaded. */
  onStart?: () => void;
  onStop?: () => void;
  onCommit?: () => void;
  onDiscard?: () => void;
  /**
   * Called by the recorder once captured audio bytes have been produced.
   * The parent takes responsibility for the asset upload (e.g. via the
   * annotation store). The recorder treats a resolved promise as success
   * and a rejected promise as a recoverable error.
   */
  onCapture: (capture: VoiceCapture) => Promise<void>;
}

function formatElapsed(ms: number): string {
  const totalSeconds = Math.max(0, Math.floor(ms / 1000));
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes.toString().padStart(2, "0")}:${seconds.toString().padStart(2, "0")}`;
}

export function VoiceRecorder(props: VoiceRecorderProps): JSX.Element {
  // The presentational variant is provided for §15.2 contract conformance.
  // It renders the externally driven state machine without touching the mic.
  const headingId = useId();
  return (
    <section
      aria-labelledby={headingId}
      style={panelStyle}
      data-ez-voice-recorder=""
    >
      <h3 id={headingId} style={headingStyle}>
        Voice annotation
      </h3>
      <p style={{ margin: 0, color: "var(--ez-color-text-secondary)" }}>
        Elapsed: <span style={monoStyle}>{formatElapsed(props.elapsedMs)}</span>
      </p>
      {props.playbackUrl ? (
        <audio
          controls
          src={props.playbackUrl}
          aria-label="Playback recorded voice annotation"
          style={{ width: "100%" }}
        />
      ) : null}
      <footer style={footerStyle}>
        {props.state === "idle" ? (
          <EzButton onClick={props.onStart} ariaLabel="Start recording">
            Start recording
          </EzButton>
        ) : null}
        {props.state === "recording" ? (
          <EzButton onClick={props.onStop} variant="danger" ariaLabel="Stop recording">
            Stop
          </EzButton>
        ) : null}
        {props.state === "review" ? (
          <>
            <EzButton
              variant="ghost"
              onClick={props.onDiscard}
              ariaLabel="Discard recording and start over"
            >
              Discard
            </EzButton>
            <EzButton
              onClick={props.onCommit}
              ariaLabel="Attach voice annotation"
            >
              Attach voice
            </EzButton>
          </>
        ) : null}
        {props.state === "uploading" ? (
          <InlineStatus tone="info" icon="↑">
            <span>Uploading voice annotation…</span>
          </InlineStatus>
        ) : null}
      </footer>
    </section>
  );
}

/**
 * Self-managed recorder used by AnnotationWorkspace. Manages the underlying
 * MediaRecorder, timer, and playback URL internally; notifies the parent via
 * callbacks. Provided as a separate export so the presentational contract in
 * §15.2 stays untouched.
 */
export function VoiceRecorderSelfManaged(
  props: VoiceRecorderSelfManagedProps,
): JSX.Element {
  const headingId = useId();
  const reduced = useReducedMotion();
  const [state, setState] = useState<InternalState>("idle");
  const [elapsedMs, setElapsedMs] = useState(0);
  const [playbackUrl, setPlaybackUrl] = useState<string | undefined>(undefined);
  const [errorText, setErrorText] = useState<string | undefined>(undefined);
  const [denied, setDenied] = useState(false);
  const bundleRef = useRef<RecorderBundle | null>(null);
  const captureRef = useRef<VoiceCapture | null>(null);
  const uploading = useRef(false);
  const starting = useRef(false);
  const mounted = useRef(false);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false; stopTimer();
      const bundle = bundleRef.current; bundleRef.current = null;
      teardownBundle(bundle); captureRef.current = null;
    };
  }, []);
  useEffect(() => () => { if (playbackUrl) URL.revokeObjectURL(playbackUrl); }, [playbackUrl]);

  function stopTimer() {
    if (timerRef.current) {
      clearInterval(timerRef.current);
      timerRef.current = null;
    }
  }

  const captureFromBundle = useCallback(
    async (bundle: RecorderBundle, durationMs: number) => {
      const blob = new Blob(bundle.chunks, {
        type: bundle.recorder.mimeType,
      });
      const arrayBuffer = await blob.arrayBuffer();
      const bytes = new Uint8Array(arrayBuffer);
      if (bytes.byteLength > MAX_VOICE_BYTES) throw new Error("Voice exceeds the 32 MiB limit.");
      if (mounted.current) captureRef.current = { bytes, durationMs, mimeType: bundle.recorder.mimeType };
    },
    [],
  );

  const handleStart = useCallback(async () => {
    if (starting.current || bundleRef.current || captureRef.current) return;
    const mimeType = pickVoiceMimeType();
    if (!mimeType) { setDenied(true); setErrorText("Opus recording in Ogg or WebM is not supported on this device."); return; }
    setErrorText(undefined);
    setDenied(false);
    if (typeof navigator === "undefined" || !navigator.mediaDevices?.getUserMedia) {
      setDenied(true);
      setErrorText(
        "Microphone capture is not available in this environment.",
      );
      return;
    }
    let stream: MediaStream;
    try {
      starting.current = true;
      stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      starting.current = false;
      if (!mounted.current) { stream.getTracks().forEach(track => track.stop()); return; }
    } catch (err) {
      starting.current = false;
      setDenied(true);
      setErrorText(
        err instanceof Error
          ? `Microphone permission denied: ${err.message}`
          : "Microphone permission denied.",
      );
      return;
    }
    if (typeof MediaRecorder === "undefined") {
      stream.getTracks().forEach((track) => track.stop());
      setDenied(true);
      setErrorText("MediaRecorder is not supported on this device.");
      return;
    }
    const chunks: BlobPart[] = [];
    let recorder: MediaRecorder;
    try {
      recorder = new MediaRecorder(stream, { mimeType, audioBitsPerSecond: 64_000 });
    } catch (err) {
      stream.getTracks().forEach((track) => track.stop());
      setDenied(true);
      setErrorText(
        err instanceof Error
          ? `MediaRecorder failed to start: ${err.message}`
          : "MediaRecorder failed to start.",
      );
      return;
    }
    let retainedBytes = 0;
    recorder.ondataavailable = (event) => {
      if (event.data.size <= 0) return;
      if (retainedBytes + event.data.size > MAX_VOICE_BYTES) {
        // Never retain an over-budget chunk or offer a truncated container as
        // valid audio. Stop devices and release the bounded pending capture.
        chunks.length = 0; retainedBytes = MAX_VOICE_BYTES;
        bundleRef.current = null; captureRef.current = null;
        if (recorder.state !== "inactive") recorder.stop();
        stream.getTracks().forEach(track => track.stop()); stopTimer();
        setErrorText("Recording reached the 32 MiB limit. Record a shorter voice note."); setState("idle");
        return;
      }
      retainedBytes += event.data.size; chunks.push(event.data);
      if (retainedBytes >= MAX_VOICE_BYTES - 1024 * 1024 && recorder.state !== "inactive") {
        // Reserve headroom for the final encoder chunk and finalize normally.
        recorder.stop();
      }
    };
    const bundle: RecorderBundle = { stream, recorder, chunks };
    bundleRef.current = bundle;
    const startedAt = Date.now();
    recorder.onstop = () => {
      if (bundleRef.current !== bundle) return;
      stopTimer(); stream.getTracks().forEach(track => track.stop());
      void captureFromBundle(bundle, Date.now() - startedAt).then(() => {
        if (!mounted.current) return;
        const capture = captureRef.current;
        if (capture && URL.createObjectURL) setPlaybackUrl(URL.createObjectURL(new Blob([capture.bytes as BlobPart], { type: capture.mimeType })));
        setState("review"); props.onStop?.();
      }).catch(err => { setErrorText(String(err)); setState("idle"); }).finally(() => { bundleRef.current = null; });
    };
    recorder.start(1000);
    setElapsedMs(0);
    setState("recording");
    props.onStart?.();
    timerRef.current = setInterval(() => {
      setElapsedMs(Date.now() - startedAt);
    }, 250);
  }, [props, captureFromBundle]);

  const handleStop = useCallback(() => {
    const bundle = bundleRef.current;
    if (bundle && bundle.recorder.state !== "inactive") bundle.recorder.stop();
  }, []);

  const handleCommit = useCallback(async () => {
    if (!captureRef.current || uploading.current) {
      return;
    }
    uploading.current = true;
    setErrorText(undefined);
    setState("uploading");
    try {
      await props.onCapture({
        bytes: captureRef.current.bytes,
        durationMs: captureRef.current.durationMs,
        mimeType: captureRef.current.mimeType,
      });
      props.onCommit?.();
      captureRef.current = null;
      if (playbackUrl) {
        URL.revokeObjectURL(playbackUrl);
      }
      setPlaybackUrl(undefined);
      setElapsedMs(0);
      setState("idle");
    } catch (err) {
      setErrorText(
        err instanceof Error
          ? `Upload failed: ${err.message}`
          : "Upload failed.",
      );
      setState("review");
    } finally { uploading.current = false; }
  }, [playbackUrl, props]);

  const handleDiscard = useCallback(() => {
    captureRef.current = null;
    if (playbackUrl) {
      URL.revokeObjectURL(playbackUrl);
    }
    setPlaybackUrl(undefined);
    setElapsedMs(0);
    setState("idle");
    setErrorText(undefined);
    props.onDiscard?.();
  }, [playbackUrl, props]);

  return (
    <section
      aria-labelledby={headingId}
      style={panelStyle}
      data-ez-voice-recorder=""
    >
      <header style={headerStyle}>
        <h3 id={headingId} style={headingStyle}>
          Voice annotation
        </h3>
        <span
          aria-live="polite"
          style={{
            color: denied
              ? "var(--ez-color-danger)"
              : state === "recording"
                ? "var(--ez-color-danger)"
                : "var(--ez-color-text-muted)",
            fontFamily: "var(--ez-font-mono)",
            fontSize: "0.8rem",
          }}
        >
          {state === "recording" ? "● Recording" : state === "uploading" ? "↑ Uploading" : "○ Idle"}
        </span>
      </header>
      <p style={{ margin: 0, color: "var(--ez-color-text-secondary)" }}>
        Elapsed: <span style={monoStyle}>{formatElapsed(elapsedMs)}</span>
      </p>
      {errorText ? (
        <InlineStatus tone={denied ? "warning" : "danger"} icon="!">
          <span>{errorText}</span>
        </InlineStatus>
      ) : null}
      {playbackUrl ? (
        <audio
          controls
          src={playbackUrl}
          aria-label="Playback captured voice annotation"
          style={{ width: "100%" }}
        />
      ) : null}
      <footer style={footerStyle}>
        {state === "idle" ? (
          <EzButton onClick={handleStart} ariaLabel="Start recording" disabled={denied}>
            Start recording
          </EzButton>
        ) : null}
        {state === "recording" ? (
          <>
            <EzButton
              onClick={handleStop}
              variant="danger"
              ariaLabel="Stop recording"
            >
              Stop
            </EzButton>
            <span
              style={{
                width: "0.6rem",
                height: "0.6rem",
                borderRadius: "50%",
                background: "var(--ez-color-danger)",
                alignSelf: "center",
                animation: reduced ? "none" : undefined,
              }}
              aria-hidden="true"
            />
          </>
        ) : null}
        {state === "review" ? (
          <>
            <EzIconButton
              ariaLabel="Discard recording"
              onClick={handleDiscard}
              title="Discard"
            >
              ✕
            </EzIconButton>
            <span style={{ flex: 1 }} />
            <EzButton
              onClick={handleCommit}
              ariaLabel="Attach voice annotation"
            >
              Attach voice
            </EzButton>
          </>
        ) : null}
        {state === "uploading" ? (
          <InlineStatus tone="info" icon="↑">
            <span>Uploading voice annotation…</span>
          </InlineStatus>
        ) : null}
      </footer>
    </section>
  );
}

function teardownBundle(bundle: RecorderBundle | null): void {
  if (!bundle) {
    return;
  }
  try {
    if (bundle.recorder.state !== "inactive") {
      bundle.recorder.stop();
    }
  } catch {
    // swallow; nothing the caller can do
  }
  bundle.stream.getTracks().forEach((track) => track.stop());
}

const panelStyle: CSSProperties = {
  background: "var(--ez-color-bg-glass)",
  borderRadius: "var(--ez-radius-md)",
  padding: "var(--ez-space-4)",
  display: "flex",
  flexDirection: "column",
  gap: "var(--ez-space-3)",
};

const headerStyle: CSSProperties = {
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  gap: "var(--ez-space-3)",
};

const headingStyle: CSSProperties = {
  margin: 0,
  fontSize: "0.95rem",
  letterSpacing: "0.04em",
  textTransform: "uppercase",
  color: "var(--ez-color-text-secondary)",
};

const footerStyle: CSSProperties = {
  display: "flex",
  alignItems: "center",
  gap: "var(--ez-space-2)",
  flexWrap: "wrap",
};

const monoStyle: CSSProperties = {
  fontFamily: "var(--ez-font-mono)",
};
