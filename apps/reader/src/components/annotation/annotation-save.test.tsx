import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { AnnotationWorkspace } from "./AnnotationWorkspace";
import { AnnotationPanel } from "./AnnotationPanel";
import { InkCanvas } from "./InkCanvas";
import { MAX_VOICE_BYTES, VoiceRecorderSelfManaged, pickVoiceMimeType } from "./VoiceRecorder";
import { VideoRecorderSelfManaged } from "./VideoRecorder";
import { useAnnotationStore } from "../../stores/annotationStore";
import { annotationService, uploadAnnotationAsset, type AnnotationDraft, type AnnotationRecord, type SelectorBundle } from "../../services/annotationService";

vi.mock("../../services/annotationService", async importOriginal => ({
  ...await importOriginal<typeof import("../../services/annotationService")>(),
  sha256Hex: vi.fn(async () => "a".repeat(64)),
}));
const selector: SelectorBundle = { article: { packId: "pack", articleKey: "A/Home" }, textQuote: { exact: "Hello", prefix: "", suffix: "" }, textPosition: { normalizedStart: 0, normalizedEnd: 5 }, capturedContext: "Hello" };
const draft: AnnotationDraft = { annotationType: "comment", selector, bodyText: "Note", style: { colorToken: "highlight-yellow", emphasis: "solid" }, tagIds: [], sidecarLayerId: "layer" };
const record: AnnotationRecord = { ...draft, annotationId: "note", tags: [], resolution: { state: "resolved", confidence: 1, explanation: "Exact" }, pinned: false, revision: 1, createdAtMs: 1, updatedAtMs: 1 };
const initial = useAnnotationStore.getState();
const workspaceProps = { article: { ...selector.article, title: "Home" }, selection: selector, annotations: [], onToolChange: vi.fn(), onCreate: vi.fn(), onUpdate: vi.fn(), onClose: vi.fn() };
class ReadableBlob extends Blob {
  arrayBuffer(): Promise<ArrayBuffer> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader(); reader.onload = () => resolve(reader.result as ArrayBuffer); reader.onerror = () => reject(reader.error); reader.readAsArrayBuffer(this);
    });
  }
}
let stopTrack: ReturnType<typeof vi.fn>;

class FakeRecorder {
  static last: FakeRecorder;
  static isTypeSupported = vi.fn((mime: string) => mime.includes("opus"));
  mimeType: string;
  state = "inactive";
  ondataavailable: ((event: { data: Blob }) => void) | null = null;
  onstop: (() => void) | null = null;
  constructor(_stream: MediaStream, options?: MediaRecorderOptions) { this.mimeType = options?.mimeType ?? "video/webm"; FakeRecorder.last = this; }
  start() { this.state = "recording"; }
  stop() { this.state = "inactive"; this.ondataavailable?.({ data: new Blob([new Uint8Array([1, 2, 3])], { type: this.mimeType }) }); this.onstop?.(); }
}
beforeEach(() => {
  vi.clearAllMocks();
  useAnnotationStore.setState({ ...initial, annotations: [], status: { kind: "loaded" }, mutation: { kind: "idle" }, layers: [{ sidecarLayerId: "layer", name: "Local", visibility: "private", trustState: "local", enabled: true, annotationCount: 0 }] }, true);
  vi.stubGlobal("Blob", ReadableBlob);
  vi.stubGlobal("PointerEvent", MouseEvent);
  vi.stubGlobal("MediaRecorder", FakeRecorder);
  FakeRecorder.isTypeSupported.mockImplementation(mime => mime.includes("opus"));
  stopTrack = vi.fn();
  Object.defineProperty(navigator, "mediaDevices", { configurable: true, value: { getUserMedia: vi.fn(async () => ({ getTracks: () => [{ stop: stopTrack }] })) } });
  URL.createObjectURL = vi.fn(() => "blob:recording"); URL.revokeObjectURL = vi.fn();
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockReturnValue({ clearRect: vi.fn(), beginPath: vi.fn(), moveTo: vi.fn(), lineTo: vi.fn(), stroke: vi.fn() } as unknown as CanvasRenderingContext2D);
});
afterEach(() => { cleanup(); vi.unstubAllGlobals(); });

async function recordVoice() {
  fireEvent.click(screen.getByRole("button", { name: "Start recording" }));
  await screen.findByRole("button", { name: "Stop recording" });
  fireEvent.click(screen.getByRole("button", { name: "Stop recording" }));
  await screen.findByRole("button", { name: "Attach voice annotation" });
}

describe("committed store results", () => {
  it("returns and applies a created record despite count refresh failure", async () => {
    vi.spyOn(annotationService, "create").mockResolvedValue(record);
    vi.spyOn(annotationService, "notebookTree").mockRejectedValue(new Error("refresh failed"));
    const result = await useAnnotationStore.getState().create(draft);
    expect(result).toEqual(record); expect(useAnnotationStore.getState().annotations).toEqual([record]);
    expect(useAnnotationStore.getState().mutation).toEqual({ kind: "idle" });
  });
  it("applies committed deletion even when optional count refresh fails", async () => {
    useAnnotationStore.setState({ annotations: [record] });
    vi.spyOn(annotationService, "delete").mockResolvedValue(undefined);
    vi.spyOn(annotationService, "notebookTree").mockRejectedValue(new Error("refresh failed"));
    expect(await useAnnotationStore.getState().remove("note", 1)).toBe(true);
    expect(useAnnotationStore.getState().annotations).toEqual([]);
    expect(useAnnotationStore.getState().mutation).toEqual({ kind: "idle" });
  });
  it("passes truthful Opus MIME and exact bytes through assetBegin/chunk/commit", async () => {
    const begin = vi.spyOn(annotationService, "assetBegin").mockResolvedValue({ uploadToken: "upload", maximumChunkBytes: 1024 });
    const chunk = vi.spyOn(annotationService, "assetChunk").mockResolvedValue(undefined);
    vi.spyOn(annotationService, "assetCommit").mockResolvedValue(record);
    vi.spyOn(annotationService, "list").mockRejectedValue(new Error("optional refresh unavailable"));
    const result = await useAnnotationStore.getState().uploadVoice("note", new Uint8Array([1, 2, 3]), 1200, "audio/webm;codecs=opus");
    expect(result).toEqual(record);
    expect(begin).toHaveBeenCalledWith({ annotationId: "note", assetKind: "voice_opus", mimeType: "audio/webm;codecs=opus", expectedBytes: 3, durationMs: 1200 });
    expect(chunk).toHaveBeenCalledWith({ uploadToken: "upload", sequenceNo: 1, bytesBase64: "AQID" });
    expect(useAnnotationStore.getState().annotations).toEqual([record]);
  });
  it.each([1, 2, 4, 5, 1024])("chunks raw bytes independently at a %i byte transport boundary", async limit => {
    vi.spyOn(annotationService, "assetBegin").mockResolvedValue({ uploadToken: "upload", maximumChunkBytes: limit });
    const chunk = vi.spyOn(annotationService, "assetChunk").mockResolvedValue(undefined);
    const commit = vi.spyOn(annotationService, "assetCommit").mockResolvedValue(record);
    const bytes = Uint8Array.from({ length: 4099 }, (_, i) => i % 256);
    await uploadAnnotationAsset({ annotationId: "note", assetKind: "voice_opus", mimeType: "audio/webm;codecs=opus", bytes, sha256Hex: "a".repeat(64) });
    const reconstructed: number[] = [];
    for (const [index, [part]] of chunk.mock.calls.entries()) {
      expect(part.sequenceNo).toBe(index + 1);
      const raw = atob(part.bytesBase64); expect(raw.length).toBeLessThanOrEqual(limit);
      reconstructed.push(...Array.from(raw, c => c.charCodeAt(0)));
    }
    expect(Uint8Array.from(reconstructed)).toEqual(bytes);
    expect(commit).toHaveBeenCalledWith({ uploadToken: "upload", expectedSha256: "a".repeat(64) });
  });
  it.each([0, -1, 1.5, Number.NaN, Number.POSITIVE_INFINITY])("rejects an invalid upload chunk limit %s", async limit => {
    vi.spyOn(annotationService, "assetBegin").mockResolvedValue({ uploadToken: "upload", maximumChunkBytes: limit });
    const chunk = vi.spyOn(annotationService, "assetChunk");
    await expect(uploadAnnotationAsset({ annotationId: "note", assetKind: "file", mimeType: "video/webm", bytes: new Uint8Array([1]), sha256Hex: "a".repeat(64) })).rejects.toThrow("invalid chunk size");
    expect(chunk).not.toHaveBeenCalled();
  });
  it("rejects unsupported voice MIME before beginning an upload", async () => {
    const begin = vi.spyOn(annotationService, "assetBegin");
    expect(await useAnnotationStore.getState().uploadVoice("note", new Uint8Array([1]), 1, "audio/mp3")).toBeUndefined();
    expect(begin).not.toHaveBeenCalled();
    expect(useAnnotationStore.getState().mutation.kind).toBe("error");
  });
});

describe("draft retention and retry", () => {
  it("retains comments on create failure and suppresses duplicate clicks until completion", async () => {
    let release!: (record: AnnotationRecord | undefined) => void;
    const create = vi.fn(() => new Promise<AnnotationRecord | undefined>(resolve => { release = resolve; }));
    useAnnotationStore.setState({ create });
    render(<AnnotationWorkspace {...workspaceProps} activeTool="comment" />);
    const text = screen.getByRole("textbox") as HTMLTextAreaElement;
    fireEvent.change(text, { target: { value: "Keep this note" } });
    const save = screen.getByRole("button", { name: "Attach comment" });
    fireEvent.click(save); fireEvent.click(save);
    expect(create).toHaveBeenCalledTimes(1);
    await act(async () => release(undefined));
    expect(text.value).toBe("Keep this note"); expect(workspaceProps.onCreate).not.toHaveBeenCalled();
    fireEvent.click(save); await act(async () => release(record));
    expect(text.value).toBe(""); expect(workspaceProps.onCreate).toHaveBeenCalledTimes(1);
  });
  it("retains drawing strokes on save rejection, locks editing and clears after retry", async () => {
    const save = vi.fn().mockRejectedValueOnce(new Error("Disk full")).mockResolvedValue(undefined);
    render(<InkCanvas enabled colorToken="highlight-blue" strokeWidth={3} onCommitSvg={save} onCancel={vi.fn()} />);
    const canvas = screen.getByRole("img", { name: "Ink drawing surface" });
    fireEvent.pointerDown(canvas, { clientX: 1, clientY: 2 }); fireEvent.pointerUp(canvas, { clientX: 3, clientY: 4 });
    fireEvent.click(screen.getByRole("button", { name: "Save ink annotation" }));
    await screen.findByRole("alert");
    expect(screen.getByText(/1 stroke ready/)).toBeTruthy();
    expect((screen.getByRole("button", { name: "Clear all strokes" }) as HTMLButtonElement).disabled).toBe(true);
    fireEvent.pointerDown(canvas, { clientX: 5, clientY: 5 }); fireEvent.pointerUp(canvas);
    fireEvent.click(screen.getByRole("button", { name: "Save ink annotation" }));
    await waitFor(() => expect(screen.queryByText(/1 stroke ready/)).toBeNull());
    expect(save.mock.calls[0]![0]).toBe(save.mock.calls[1]![0]);
  });
  it("retries voice attachment with the same annotation ID and actual capture MIME", async () => {
    const create = vi.fn().mockResolvedValue(record);
    const uploadVoice = vi.fn().mockResolvedValueOnce(undefined).mockResolvedValue(record);
    useAnnotationStore.setState({ create, uploadVoice });
    render(<AnnotationWorkspace {...workspaceProps} activeTool="voice" />);
    await recordVoice();
    fireEvent.click(screen.getByRole("button", { name: "Attach voice annotation" }));
    await screen.findByText(/Upload failed:/);
    expect(workspaceProps.onCreate).not.toHaveBeenCalled();
    expect(screen.getByLabelText("Playback captured voice annotation")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Attach voice annotation" }));
    await screen.findByRole("button", { name: "Start recording" });
    expect(create).toHaveBeenCalledTimes(1);
    expect(uploadVoice).toHaveBeenCalledTimes(2);
    expect(uploadVoice.mock.calls[0]![0]).toBe("note"); expect(uploadVoice.mock.calls[1]![0]).toBe("note");
    expect(uploadVoice.mock.calls[0]![1]).toEqual(new Uint8Array([1, 2, 3]));
    expect(uploadVoice.mock.calls[0]![3]).toBe("audio/ogg;codecs=opus");
    expect(workspaceProps.onCreate).toHaveBeenCalledTimes(1);
  });
  it("retries ink upload against its existing annotation and leaves failed strokes in place", async () => {
    const create = vi.fn().mockResolvedValue(record);
    const uploadInk = vi.fn().mockResolvedValueOnce(undefined).mockResolvedValue(record);
    useAnnotationStore.setState({ create, uploadInk });
    render(<AnnotationWorkspace {...workspaceProps} activeTool="ink" />);
    const canvas = screen.getByRole("img", { name: "Ink drawing surface" });
    fireEvent.pointerDown(canvas, { clientX: 4, clientY: 5 }); fireEvent.pointerUp(canvas);
    fireEvent.click(screen.getByRole("button", { name: "Save ink annotation" }));
    await screen.findByRole("alert");
    expect(screen.getByText(/1 stroke ready/)).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Save ink annotation" }));
    await waitFor(() => expect(screen.queryByText(/1 stroke ready/)).toBeNull());
    expect(create).toHaveBeenCalledTimes(1); expect(uploadInk).toHaveBeenCalledTimes(2);
    expect(uploadInk.mock.calls[0]).toEqual(uploadInk.mock.calls[1]);
  });
  it("retains a capture after create failure and creates only on the subsequent attempt", async () => {
    const create = vi.fn().mockResolvedValueOnce(undefined).mockResolvedValue(record);
    const uploadVoice = vi.fn().mockResolvedValue(record);
    useAnnotationStore.setState({ create, uploadVoice });
    render(<AnnotationWorkspace {...workspaceProps} activeTool="voice" />);
    await recordVoice();
    fireEvent.click(screen.getByRole("button", { name: "Attach voice annotation" }));
    await screen.findByText(/Upload failed:/);
    expect(uploadVoice).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Attach voice annotation" }));
    await screen.findByRole("button", { name: "Start recording" });
    expect(uploadVoice).toHaveBeenCalledTimes(1); expect(create).toHaveBeenCalledTimes(2);
  });
  it("retains video capture when the upload promise rejects", async () => {
    const capture = vi.fn().mockRejectedValueOnce(new Error("Upload failed")).mockResolvedValue(undefined);
    render(<VideoRecorderSelfManaged onCapture={capture} />);
    fireEvent.click(screen.getByRole("button", { name: "Start filming" }));
    await screen.findByRole("button", { name: "Stop filming" });
    fireEvent.click(screen.getByRole("button", { name: "Stop filming" }));
    await screen.findByRole("button", { name: "Attach film note" });
    fireEvent.click(screen.getByRole("button", { name: "Attach film note" }));
    await screen.findByRole("alert");
    fireEvent.click(screen.getByRole("button", { name: "Attach film note" }));
    await screen.findByRole("button", { name: "Start filming" });
    expect(capture.mock.calls[0]![0]).toEqual(capture.mock.calls[1]![0]);
  });
});

describe("recording budgets and component lifecycle", () => {
  it("honestly reports unsupported Opus recording without opening microphone", async () => {
    FakeRecorder.isTypeSupported.mockReturnValue(false);
    expect(pickVoiceMimeType()).toBeUndefined();
    render(<VoiceRecorderSelfManaged onCapture={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: "Start recording" }));
    expect(await screen.findByText(/Opus recording.*not supported/)).toBeTruthy();
    expect(navigator.mediaDevices.getUserMedia).not.toHaveBeenCalled();
  });
  it("stops and releases devices without retaining oversized capture", async () => {
    const capture = vi.fn();
    render(<VoiceRecorderSelfManaged onCapture={capture} />);
    fireEvent.click(screen.getByRole("button", { name: "Start recording" }));
    await screen.findByRole("button", { name: "Stop recording" });
    act(() => FakeRecorder.last.ondataavailable?.({ data: { size: MAX_VOICE_BYTES + 1 } as Blob }));
    expect(await screen.findByText(/Recording reached the 32 MiB limit/)).toBeTruthy();
    expect(FakeRecorder.last.state).toBe("inactive"); expect(stopTrack).toHaveBeenCalled();
    expect(screen.queryByRole("button", { name: "Attach voice annotation" })).toBeNull();
    expect(capture).not.toHaveBeenCalled();
  });
  it("keeps hook order stable for empty, populated and empty lists", () => {
    const props = { onSelect: vi.fn(), onDelete: vi.fn() };
    const view = render(<AnnotationPanel {...props} annotations={[]} />);
    view.rerender(<AnnotationPanel {...props} annotations={[record]} />);
    expect(screen.getByText("Annotations on this article (1)")).toBeTruthy();
    view.rerender(<AnnotationPanel {...props} annotations={[]} />);
    expect(screen.getByText(/No annotations yet/)).toBeTruthy();
  });
});
