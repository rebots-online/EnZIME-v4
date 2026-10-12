import type { JSX } from "react";
import { useCallback, useEffect, useRef, useState } from "react";
import { EzButton, useReducedMotion } from "./sharedUi";

/// A captured video clip handed to the parent for upload. `mimeType` is the
/// exact MediaRecorder container codec string so the backend stores a
/// truthful MIME alongside the asset bytes.
export interface VideoCapture {
  readonly bytes: Uint8Array;
  readonly mimeType: string;
  readonly durationMs: number;
}

export interface VideoRecorderSelfManagedProps {
  onCapture: (capture: VideoCapture) => Promise<void>;
  onStart?: () => void;
  onStop?: () => void;
  onCommit?: () => void;
  onDiscard?: () => void;
  disabled?: boolean;
}

type InternalState = "idle" | "recording" | "review" | "uploading";

interface RecorderBundle {
  stream: MediaStream;
  recorder: MediaRecorder;
  startedAtMs: number;
}

function pickVideoMimeType(): string {
  if (typeof MediaRecorder === "undefined" || !MediaRecorder.isTypeSupported) {
    return "video/webm";
  }
  for (const candidate of [
    "video/webm;codecs=vp9,opus",
    "video/webm;codecs=vp8,opus",
    "video/webm",
    "video/mp4",
  ]) {
    if (MediaRecorder.isTypeSupported(candidate)) {
      return candidate;
    }
  }
  return "video/webm";
}

/// Film a short video note from the camera (with microphone). Mirrors the
/// VoiceRecorder self-managed pattern: the component owns the MediaRecorder
/// lifecycle and hands completed bytes to `onCapture`, which performs the
/// upload.
export function VideoRecorderSelfManaged(
  props: VideoRecorderSelfManagedProps,
): JSX.Element {
  const [state, setState] = useState<InternalState>("idle");
  const [elapsedMs, setElapsedMs] = useState(0);
  const [playbackUrl, setPlaybackUrl] = useState<string | undefined>();
  const [errorText, setErrorText] = useState<string | undefined>();
  const bundleRef = useRef<RecorderBundle | null>(null);
  const uploading = useRef(false);
  const captureRef = useRef<VideoCapture | null>(null);
  const chunksRef = useRef<BlobPart[]>([]);
  const timerRef = useRef<number | undefined>(undefined);
  const reduced = useReducedMotion();

  useEffect(
    () => () => {
      if (timerRef.current !== undefined) {
        window.clearInterval(timerRef.current);
      }
      bundleRef.current?.stream.getTracks().forEach((track) => track.stop());
      if (playbackUrl && typeof URL !== "undefined" && URL.revokeObjectURL) {
        URL.revokeObjectURL(playbackUrl);
      }
    },
    [playbackUrl],
  );

  const startTimer = useCallback(() => {
    const startedAt = Date.now();
    setElapsedMs(0);
    timerRef.current = window.setInterval(() => {
      setElapsedMs(Date.now() - startedAt);
    }, 250);
  }, []);

  const stopTimer = useCallback(() => {
    if (timerRef.current !== undefined) {
      window.clearInterval(timerRef.current);
      timerRef.current = undefined;
    }
  }, []);

  const handleStart = useCallback(async () => {
    setErrorText(undefined);
    if (
      typeof navigator === "undefined" ||
      !navigator.mediaDevices?.getUserMedia
    ) {
      setErrorText("Video capture is not available in this environment.");
      return;
    }
    if (typeof MediaRecorder === "undefined") {
      setErrorText("MediaRecorder is not available in this environment.");
      return;
    }
    let stream: MediaStream;
    try {
      stream = await navigator.mediaDevices.getUserMedia({
        audio: true,
        video: true,
      });
    } catch (err) {
      setErrorText(
        err instanceof Error
          ? `Camera/microphone unavailable: ${err.message}`
          : "Camera/microphone unavailable.",
      );
      return;
    }
    const mimeType = pickVideoMimeType();
    let recorder: MediaRecorder;
    try {
      recorder = new MediaRecorder(stream, { mimeType });
    } catch {
      try {
        recorder = new MediaRecorder(stream);
      } catch (err) {
        stream.getTracks().forEach((track) => track.stop());
        setErrorText(
          err instanceof Error ? `Recorder failed: ${err.message}` : "Recorder failed.",
        );
        return;
      }
    }
    const chunks: BlobPart[] = [];
    chunksRef.current = chunks;
    recorder.ondataavailable = (event) => {
      if (event.data.size > 0) {
        chunks.push(event.data);
      }
    };
    const startedAtMs = Date.now();
    bundleRef.current = { stream, recorder, startedAtMs };
    recorder.start(1_000);
    startTimer();
    setState("recording");
    props.onStart?.();
  }, [props, startTimer]);

  const handleStop = useCallback(async () => {
    const bundle = bundleRef.current;
    if (!bundle) {
      return;
    }
    stopTimer();
    const durationMs = Date.now() - bundle.startedAtMs;
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
      const blob = new Blob(chunksRef.current, {
        type: bundle.recorder.mimeType || "video/webm",
      });
      const bytes = new Uint8Array(await blob.arrayBuffer());
      captureRef.current = {
        bytes,
        mimeType: bundle.recorder.mimeType || "video/webm",
        durationMs,
      };
      if (typeof URL !== "undefined" && URL.createObjectURL) {
        setPlaybackUrl(URL.createObjectURL(blob));
      }
      setState("review");
      props.onStop?.();
    } catch (err) {
      setErrorText(
        err instanceof Error
          ? `Could not capture video: ${err.message}`
          : "Could not capture video.",
      );
      setState("idle");
    } finally {
      bundleRef.current = null;
    }
  }, [props, stopTimer]);

  const handleCommit = useCallback(async () => {
    if (!captureRef.current || uploading.current) {
      return;
    }
    uploading.current = true;
    setErrorText(undefined);
    setState("uploading");
    try {
      await props.onCapture(captureRef.current);
      props.onCommit?.();
      captureRef.current = null;
      if (playbackUrl && typeof URL !== "undefined" && URL.revokeObjectURL) {
        URL.revokeObjectURL(playbackUrl);
      }
      setPlaybackUrl(undefined);
      setElapsedMs(0);
      setState("idle");
    } catch (err) {
      setErrorText(
        err instanceof Error ? `Upload failed: ${err.message}` : "Upload failed.",
      );
      setState("review");
    } finally { uploading.current = false; }
  }, [playbackUrl, props]);

  const handleDiscard = useCallback(() => {
    captureRef.current = null;
    if (playbackUrl && typeof URL !== "undefined" && URL.revokeObjectURL) {
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
      aria-label="Video note recorder"
      style={{
        background: "var(--ez-color-bg-glass)",
        borderRadius: "var(--ez-radius-md)",
        padding: "var(--ez-space-4)",
        display: "flex",
        flexDirection: "column",
        gap: "var(--ez-space-3)",
      }}
      data-ez-video-recorder=""
    >
      <header
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
        }}
      >
        <h3
          style={{
            margin: 0,
            fontSize: "0.95rem",
            letterSpacing: "0.04em",
            textTransform: "uppercase",
            color: "var(--ez-color-text-secondary)",
          }}
        >
          Film note
        </h3>
        <span
          style={{
            color:
              state === "recording"
                ? "var(--ez-color-danger)"
                : "var(--ez-color-text-muted)",
            fontFamily: "var(--ez-font-mono)",
            fontSize: "0.8rem",
          }}
        >
          {state === "recording"
            ? `● REC ${(elapsedMs / 1000).toFixed(1)}s`
            : state === "review"
              ? "Review ready"
              : state === "uploading"
                ? "Uploading…"
                : "○ Idle"}
        </span>
      </header>
      {errorText ? (
        <p role="alert" style={{ margin: 0, color: "var(--ez-color-danger)" }}>
          {errorText}
        </p>
      ) : null}
      {state === "review" && playbackUrl ? (
        <video
          src={playbackUrl}
          controls
          style={{
            width: "100%",
            borderRadius: "var(--ez-radius-sm)",
            background: "var(--ez-color-bg-elevated)",
          }}
        />
      ) : null}
      <div style={{ display: "flex", gap: "var(--ez-space-2)" }}>
        {state === "idle" ? (
          <EzButton
            onClick={() => void handleStart()}
            disabled={props.disabled}
            ariaLabel="Start filming"
          >
            ● Film
          </EzButton>
        ) : null}
        {state === "recording" ? (
          <EzButton
            variant="primary"
            onClick={() => void handleStop()}
            ariaLabel="Stop filming"
          >
            ■ Stop
          </EzButton>
        ) : null}
        {state === "review" || state === "uploading" ? (
          <>
            <EzButton
              onClick={handleCommit}
              disabled={state === "uploading"}
              ariaLabel="Attach film note"
            >
              Attach film
            </EzButton>
            <EzButton
              variant="ghost"
              onClick={handleDiscard}
              disabled={state === "uploading"}
              ariaLabel="Discard film note"
            >
              Discard
            </EzButton>
          </>
        ) : null}
      </div>
      <p
        style={{
          margin: 0,
          color: "var(--ez-color-text-muted)",
          fontSize: "0.875rem",
          transition: reduced ? "none" : undefined,
        }}
      >
        Records camera + microphone to a WebM clip attached to this article.
        Clips stay on this device.
      </p>
    </section>
  );
}

/// MediaRecorder collects BlobParts via ondataavailable into the ref set at
/// recording start; nothing further is needed at unmount.
