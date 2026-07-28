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
  onCapture: (capture: { bytes: Uint8Array; durationMs: number }) => Promise<void>;
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
  const captureRef = useRef<{ bytes: Uint8Array; durationMs: number } | null>(null);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    return () => {
      stopTimer();
      if (playbackUrl) {
        URL.revokeObjectURL(playbackUrl);
      }
      teardownBundle(bundleRef.current);
      bundleRef.current = null;
    };
  }, [playbackUrl]);

  function stopTimer() {
    if (timerRef.current) {
      clearInterval(timerRef.current);
      timerRef.current = null;
    }
  }

  const captureFromBundle = useCallback(
    async (bundle: RecorderBundle, durationMs: number) => {
      const blob = new Blob(bundle.chunks, {
        type: bundle.recorder.mimeType || "audio/webm",
      });
      const arrayBuffer = await blob.arrayBuffer();
      const bytes = new Uint8Array(arrayBuffer);
      captureRef.current = { bytes, durationMs };
    },
    [],
  );

  const handleStart = useCallback(async () => {
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
      stream = await navigator.mediaDevices.getUserMedia({ audio: true });
    } catch (err) {
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
      recorder = new MediaRecorder(stream);
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
    recorder.ondataavailable = (event) => {
      if (event.data.size > 0) {
        chunks.push(event.data);
      }
    };
    const bundle: RecorderBundle = { stream, recorder, chunks };
    bundleRef.current = bundle;
    recorder.start();
    setElapsedMs(0);
    setState("recording");
    props.onStart?.();
    const startedAt = Date.now();
    timerRef.current = setInterval(() => {
      setElapsedMs(Date.now() - startedAt);
    }, 250);
  }, [props]);

  const handleStop = useCallback(async () => {
    const bundle = bundleRef.current;
    if (!bundle) {
      return;
    }
    stopTimer();
    const durationMs = elapsedMs;
    await new Promise<void>((resolve) => {
      bundle.recorder.onstop = () => resolve();
      if (bundle.recorder.state !== "inactive") {
        bundle.recorder.stop();
      } else {
        resolve();
      }
    });
    bundle.stream.getTracks().forEach((track) => track.stop());
    try {
      await captureFromBundle(bundle, durationMs);
      if (captureRef.current && typeof URL !== "undefined" && URL.createObjectURL) {
        const blob = new Blob([captureRef.current.bytes as BlobPart], {
          type: bundle.recorder.mimeType || "audio/webm",
        });
        const url = URL.createObjectURL(blob);
        setPlaybackUrl(url);
      }
      setState("review");
      props.onStop?.();
    } catch (err) {
      setErrorText(
        err instanceof Error
          ? `Could not capture audio: ${err.message}`
          : "Could not capture audio.",
      );
      setState("idle");
    } finally {
      bundleRef.current = null;
    }
  }, [captureFromBundle, elapsedMs, props]);

  const handleCommit = useCallback(async () => {
    if (!captureRef.current) {
      return;
    }
    setState("uploading");
    try {
      await props.onCapture({
        bytes: captureRef.current.bytes,
        durationMs: captureRef.current.durationMs,
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
    }
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
