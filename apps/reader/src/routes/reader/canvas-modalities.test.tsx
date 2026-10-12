import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { buildRendererBridgeScript, validateRendererMessage, type CanvasStroke } from "./rendererBridge";
import { canvasInkSvg, loadCanvasAssets, saveCanvasDraft, validAnnotationAsset, type CanvasSaveSession } from "./canvasPersistence";
import { ArticleContent } from "./ArticleContent";
import { createMemoryRouter, RouterProvider } from "react-router-dom";
import { ReaderView } from "./ReaderView";
import { ReaderCanvasCapture } from "./ReaderCanvasCapture";
import { readerService } from "../../services/readerService";
import { useReaderStore } from "../../stores/readerStore";
import { ReaderCanvasTools } from "./ReaderCanvasTools";
import { PhotoCaptureSelfManaged } from "../../components/annotation/PhotoCapture";
import { MAX_VIDEO_BYTES, MAX_VIDEO_DURATION_MS, VideoRecorderSelfManaged } from "../../components/annotation/VideoRecorder";
import { annotationService, type AnnotationRecord, type ReaderAnnotationAsset, type SelectorBundle } from "../../services/annotationService";
import { useAnnotationStore } from "../../stores/annotationStore";
import { invoke } from "@tauri-apps/api/core";

vi.mock("./AnnotationDrawer", () => ({ AnnotationDrawer: () => null }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(), convertFileSrc: (path: string) => `http://enzime-resource.localhost/${path}` }));
vi.mock("../../services/annotationService", async importOriginal => ({ ...await importOriginal<typeof import("../../services/annotationService")>(), sha256Hex: vi.fn(async () => "a".repeat(64)) }));
const article = { packId: "pack", articleKey: "A/Home" };
const selector: SelectorBundle = { article, textQuote: { exact: "Paragraph text", prefix: "", suffix: "" }, textPosition: { normalizedStart: 0, normalizedEnd: 14 }, capturedContext: "Paragraph text" };
const record: AnnotationRecord = { annotationId: "note", annotationType: "comment", selector, bodyText: "Typed note", style: { colorToken: "highlight-blue", emphasis: "solid" }, tags: [], sidecarLayerId: "layer", resolution: { state: "resolved", confidence: 1, explanation: "Exact" }, pinned: false, revision: 1, createdAtMs: 1, updatedAtMs: 1 };
const asset: ReaderAnnotationAsset = { annotationId: "note", annotationAssetId: "asset", assetKind: "image", mimeType: "image/jpeg", byteLength: 3, resourceUrl: "http://enzime-resource.localhost/annotation/note/asset" };
const resources = { urlForEntry: (pack: string, key: string) => `http://enzime-resource.localhost/pack/${pack}/${key}`, cspSources: ["http://enzime-resource.localhost"] };
const initial = useAnnotationStore.getState();
const initialReader = useReaderStore.getState();
const strokes: CanvasStroke[] = [{ colorToken: "highlight-blue", width: 3, points: [{ x: 1, y: 2 }, { x: 3, y: 4 }] }];
class ReadableBlob extends Blob { arrayBuffer(): Promise<ArrayBuffer> { return new Promise((resolve, reject) => { const reader = new FileReader(); reader.onload = () => resolve(reader.result as ArrayBuffer); reader.onerror = reject; reader.readAsArrayBuffer(this); }); } }
let tracks: ReturnType<typeof vi.fn>;
class Recorder {
  static last: Recorder;
  static isTypeSupported = vi.fn((mime: string) => mime.includes("webm"));
  state = "inactive"; mimeType: string;
  ondataavailable?: (event: { data: Blob }) => void; onstop?: () => void;
  constructor(_stream: MediaStream, options?: MediaRecorderOptions) { this.mimeType = options?.mimeType ?? "video/webm"; Recorder.last = this; }
  start() { this.state = "recording"; }
  stop() { this.state = "inactive"; this.ondataavailable?.({ data: new Blob([new Uint8Array([1, 2, 3])], { type: this.mimeType }) }); this.onstop?.(); }
}
const context = { clearRect: vi.fn(), beginPath: vi.fn(), moveTo: vi.fn(), lineTo: vi.fn(), stroke: vi.fn(), drawImage: vi.fn() };
beforeEach(() => {
  vi.clearAllMocks(); vi.mocked(invoke).mockResolvedValue([]);
  useReaderStore.setState(initialReader, true);
  useAnnotationStore.setState({ ...initial, status: { kind: "loaded" }, annotations: [], layers: [{ sidecarLayerId: "layer", name: "Local", visibility: "private", trustState: "local", enabled: true, annotationCount: 0 }] }, true);
  vi.stubGlobal("Blob", ReadableBlob); vi.stubGlobal("MediaRecorder", Recorder); vi.stubGlobal("PointerEvent", MouseEvent);
  tracks = vi.fn(); Object.defineProperty(navigator, "mediaDevices", { configurable: true, value: { getUserMedia: vi.fn(async () => ({ getTracks: () => [{ stop: tracks }] })) } });
  vi.spyOn(HTMLMediaElement.prototype, "play").mockResolvedValue(undefined);
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockReturnValue(context as unknown as CanvasRenderingContext2D);
  URL.createObjectURL = vi.fn(() => "blob:photo"); URL.revokeObjectURL = vi.fn();
  Recorder.isTypeSupported.mockImplementation(mime => mime.includes("webm"));
});
afterEach(() => { cleanup(); vi.unstubAllGlobals(); vi.useRealTimers(); });
function runtime(html = "<p>Paragraph text</p><p>Other passage</p>") {
  const frame = document.createElement("iframe"); document.body.append(frame);
  const win = frame.contentWindow! as Window & typeof globalThis, doc = win.document;
  doc.body.innerHTML = html;
  vi.spyOn(win.HTMLCanvasElement.prototype, "getContext").mockReturnValue(context as unknown as CanvasRenderingContext2D);
  let rect = { x: 10, y: 20, left: 10, top: 20, right: 310, bottom: 120, width: 300, height: 100, toJSON() {} };
  vi.spyOn(win.HTMLElement.prototype, "getBoundingClientRect").mockImplementation(() => rect);
  const receive = vi.spyOn(win.parent, "postMessage").mockImplementation(() => undefined);
  new Function("window", "document", "NodeFilter", buildRendererBridgeScript("channel", article))(win, doc, NodeFilter);
  const send = (data: object) => win.dispatchEvent(new MessageEvent("message", { source: win.parent, data: { schemaVersion: 1, channel: "channel", ...data } }));
  const messages = (kind: string) => receive.mock.calls.map(call => call[0]).filter(message => message.kind === kind);
  return { win, doc, send, messages, close: () => frame.remove(), resize: () => { rect = { ...rect, width: 200, right: 210 }; win.dispatchEvent(new Event("resize")); } };
}

describe("canvas trust and persistence", () => {
  it("validates numeric drawing payloads and rejects oversized, foreign and raw-markup fields", () => {
    const message = { schemaVersion: 1, channel: "channel", kind: "canvas_ink_commit", draftId: "draft", selector, strokes, width: 300, height: 100 };
    expect(validateRendererMessage(message, "channel", article)).toEqual(message);
    for (const x of [NaN, Infinity, -1, 301]) expect(validateRendererMessage({ ...message, strokes: [{ ...strokes[0], points: [{ x, y: 0 }] }] }, "channel", article)).toBeUndefined();
    expect(validateRendererMessage({ ...message, svg: "<svg onload=bad()>" }, "channel", article)).toBeUndefined();
    expect(validateRendererMessage({ ...message, selector: { ...selector, article: { ...article, packId: "other" } } }, "channel", article)).toBeUndefined();
    expect(validateRendererMessage({ ...message, strokes: Array.from({ length: 1001 }, () => strokes[0]) }, "channel", article)).toBeUndefined();
    expect(canvasInkSvg(strokes, 300, 100)).toContain('stroke="#2365b4"');
    expect(() => canvasInkSvg([{ ...strokes[0]!, colorToken: 'red" onload="bad()' }], 300, 100)).toThrow();
  });
  it("uploads parent-generated passive SVG and retries failed media without duplicate annotations", async () => {
    const create = vi.fn().mockResolvedValue({ ...record, annotationType: "ink" }); useAnnotationStore.setState({ create });
    const begin = vi.spyOn(annotationService, "assetBegin").mockRejectedValueOnce(new Error("Disk full")).mockResolvedValue({ uploadToken: "token", maximumChunkBytes: 4096 });
    const chunk = vi.spyOn(annotationService, "assetChunk").mockResolvedValue(undefined); vi.spyOn(annotationService, "assetCommit").mockResolvedValue({ ...record, annotationType: "ink" });
    const session: CanvasSaveSession = { pending: new Map(), busy: new Set() };
    const input = { draftId: "draft", selector, kind: "draw" as const, strokes, width: 300, height: 100 };
    await expect(saveCanvasDraft(session, input)).rejects.toThrow("Disk full");
    expect(session.pending.get("draft")?.annotation.annotationId).toBe("note");
    await saveCanvasDraft(session, input);
    expect(create).toHaveBeenCalledTimes(1); expect(begin.mock.calls.every(([request]) => request.annotationId === "note")).toBe(true);
    const svg = atob(chunk.mock.calls[0]![0].bytesBase64); expect(svg).toContain("<svg"); expect(svg).not.toContain("<script");
    expect(session.pending.size).toBe(0);
  });
  it("validates exact native asset origin and annotation membership", () => {
    expect(validAnnotationAsset(asset, "note", resources.urlForEntry("pack", "A/Home"))).toBe(true);
    for (const resourceUrl of ["https://evil.test/annotation/note/asset", "http://enzime-resource.localhost/annotation/other/asset", "http://enzime-resource.localhost/annotation/note/asset?x=1", "javascript:bad()"])
      expect(validAnnotationAsset({ ...asset, resourceUrl }, "note", resources.urlForEntry("pack", "A/Home"))).toBe(false);
  });
  it("bounds metadata loading to four concurrent requests and stops starting old-article work", async () => {
    const resolvers: ((value: unknown) => void)[] = [];
    vi.mocked(invoke).mockImplementation(() => new Promise(resolve => resolvers.push(resolve)));
    let cancelled = false;
    const job = loadCanvasAssets(Array.from({ length: 8 }, (_, i) => ({ ...record, annotationId: `note${i}` })), resources.urlForEntry("pack", "A/Home"), () => cancelled);
    expect(invoke).toHaveBeenCalledTimes(4);
    cancelled = true; for (const resolve of resolvers) resolve([]);
    expect((await job).size).toBe(0); expect(invoke).toHaveBeenCalledTimes(4);
  });
});

describe("actual trusted canvas bridge", () => {
  it("types at a text anchor, retains failure, saves and replays without polluting article text", () => {
    const r = runtime(); r.send({ kind: "tool_changed", tool: "type" }); r.doc.querySelector("p")!.click();
    const input = r.doc.querySelector("textarea")!; input.value = "A note <script>is plain text</script>";
    Array.from(r.doc.querySelectorAll("button")).find(button => button.textContent === "Save note")!.click();
    const commit = r.messages("canvas_type_commit")[0]; expect(commit.selector.textQuote.exact).toBe("Paragraph text");
    r.send({ kind: "draft_result", draftId: commit.draftId, saved: false, error: "Disk full" });
    expect(input.value).toContain("A note"); expect(r.doc.querySelector("[role=alert]")?.textContent).toBe("Disk full");
    r.send({ kind: "draft_result", draftId: commit.draftId, saved: true });
    expect(r.doc.querySelector("textarea")).toBeNull();
    r.send({ kind: "canvas_annotations_changed", annotations: [{ ...record, bodyText: commit.bodyText, colorToken: "highlight-blue", assets: [] }] });
    expect(r.doc.querySelector("[data-ez-canvas=saved]")?.textContent).toContain("<script>"); expect(r.doc.querySelector("script")).toBeNull();
    const range = r.doc.createRange(); range.selectNodeContents(r.doc.querySelectorAll("p")[2]!); r.win.getSelection()!.removeAllRanges(); r.win.getSelection()!.addRange(range); r.doc.dispatchEvent(new Event("selectionchange"));
    const selection = r.messages("selection_changed").at(-1)?.selector;
    expect(selection?.textQuote.exact).toBe("Other passage"); expect(selection?.textPosition.normalizedStart).toBe(14);
    r.close();
    const reopened = runtime(); reopened.send({ kind: "canvas_annotations_changed", annotations: [{ ...record, bodyText: commit.bodyText, colorToken: "highlight-blue", assets: [] }] });
    expect(reopened.doc.querySelector("[data-ez-canvas=saved]")?.textContent).toContain("A note"); reopened.close();
  });
  it("draws numeric points over a paragraph, reflows and dispatches photo/video anchors", () => {
    const r = runtime(); r.send({ kind: "tool_changed", tool: "draw" }); r.doc.querySelector("p")!.click();
    const canvas = r.doc.querySelector("canvas")!;
    canvas.dispatchEvent(new r.win.MouseEvent("pointerdown", { clientX: 20, clientY: 30 }));
    canvas.dispatchEvent(new r.win.MouseEvent("pointermove", { clientX: 40, clientY: 50 })); canvas.dispatchEvent(new r.win.MouseEvent("pointerup"));
    Array.from(r.doc.querySelectorAll("button")).find(button => button.textContent === "Save drawing")!.click();
    const commit = r.messages("canvas_ink_commit")[0]; expect(commit.strokes[0].points).toEqual([{ x: 10, y: 10 }, { x: 30, y: 30 }]);
    expect(validateRendererMessage(commit, "channel", article)).toEqual(commit);
    r.resize(); expect(r.doc.querySelector<HTMLElement>("[data-ez-canvas=draft]")?.style.width).toBe("200px");
    r.send({ kind: "draft_result", draftId: commit.draftId, saved: true });
    for (const tool of ["photo", "video", "voice"]) {
      r.send({ kind: "tool_changed", tool }); r.doc.querySelector("p")!.click();
      const request = r.messages("canvas_capture_request").at(-1); expect(request.captureKind).toBe(tool); expect(request.selector.article).toEqual(article);
      r.send({ kind: "discard_draft", draftId: request.draftId });
    }
    r.close();
  });
  it("anchors image annotations by scoped entry and keeps unresolved cards visible", () => {
    const r = runtime('<p>Paragraph text</p><img data-ez-resource="I/photo.jpg" alt="Photo">');
    r.send({ kind: "tool_changed", tool: "photo" }); r.doc.querySelector("img")!.click();
    const request = r.messages("canvas_capture_request")[0]; expect(request.selector.imageRegion).toEqual({ resourceEntryKey: "I/photo.jpg", xFraction: 0, yFraction: 0, widthFraction: 1, heightFraction: 1 });
    r.send({ kind: "discard_draft", draftId: request.draftId });
    r.send({ kind: "canvas_annotations_changed", annotations: [{ ...record, selector: { ...selector, textPosition: undefined, textQuote: { exact: "Missing", prefix: "", suffix: "" } }, colorToken: "highlight-blue", assets: [asset, { ...asset, mimeType: "video/webm" }] }] });
    const section = r.doc.querySelector<HTMLElement>("[data-ez-canvas=unresolved]")!;
    expect(section.hidden).toBe(false); expect(section.textContent).toContain("Annotation retained here"); expect(section.querySelector("img")?.src).toBe(asset.resourceUrl); expect(section.querySelector("video")?.controls).toBe(true);
    r.close();
  });
  it("offers voice recording separately from typed notes", () => {
    render(<ReaderCanvasTools tool="select" busy={false} onToolChange={vi.fn()} />);
    expect((screen.getByRole("button", { name: "Voice" }) as HTMLButtonElement).disabled).toBe(false);
    expect(screen.getByRole("button", { name: "Type" })).toBeTruthy();
  });
});

describe("real component capture state machines", () => {
  it("takes bounded JPEG bytes, retains failed save, and cleans camera and blob URL", async () => {
    vi.spyOn(HTMLVideoElement.prototype, "videoWidth", "get").mockReturnValue(8000); vi.spyOn(HTMLVideoElement.prototype, "videoHeight", "get").mockReturnValue(4000);
    vi.spyOn(HTMLCanvasElement.prototype, "toBlob").mockImplementation(callback => callback(new Blob([new Uint8Array([4, 5, 6])], { type: "image/jpeg" })));
    const save = vi.fn().mockRejectedValueOnce(new Error("Disk full")).mockResolvedValue(undefined);
    const view = render(<PhotoCaptureSelfManaged onCapture={save} onDiscard={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: "Start camera" })); await screen.findByLabelText("Live camera preview");
    fireEvent.click(screen.getByRole("button", { name: "Take photograph" })); await screen.findByAltText("Captured annotation photograph");
    expect(tracks).toHaveBeenCalled(); fireEvent.click(screen.getByRole("button", { name: "Save photograph" })); await screen.findByRole("alert");
    expect(save.mock.calls[0]![0]).toEqual({ bytes: new Uint8Array([4, 5, 6]), mimeType: "image/jpeg", width: 4096, height: 2048 });
    fireEvent.click(screen.getByRole("button", { name: "Save photograph" })); await screen.findByRole("button", { name: "Start camera" });
    expect(save.mock.calls[0]![0]).toEqual(save.mock.calls[1]![0]); view.unmount(); expect(URL.revokeObjectURL).toHaveBeenCalledWith("blob:photo");
  });
  it("offers camera permission retry and releases a late stream after cancellation", async () => {
    vi.mocked(navigator.mediaDevices.getUserMedia).mockRejectedValueOnce(new Error("Denied"));
    const view = render(<PhotoCaptureSelfManaged onCapture={vi.fn()} onDiscard={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: "Start camera" })); expect(await screen.findByRole("alert")).toBeTruthy();
    let resolve!: (stream: MediaStream) => void;
    vi.mocked(navigator.mediaDevices.getUserMedia).mockImplementationOnce(() => new Promise(done => { resolve = done; }));
    fireEvent.click(screen.getByRole("button", { name: "Start camera" })); view.unmount();
    await act(async () => resolve({ getTracks: () => [{ stop: tracks }] } as unknown as MediaStream)); expect(tracks).toHaveBeenCalled();
  });
  it("has live muted video and rejects a final chunk exceeding the hard byte budget", async () => {
    const save = vi.fn(); render(<VideoRecorderSelfManaged onCapture={save} />);
    fireEvent.click(screen.getByRole("button", { name: "Start filming" }));
    const preview = await screen.findByLabelText("Live video preview") as HTMLVideoElement; expect(preview.muted).toBe(true);
    act(() => Recorder.last.ondataavailable?.({ data: { size: MAX_VIDEO_BYTES + 1 } as Blob }));
    expect(await screen.findByRole("alert")).toHaveProperty("textContent", expect.stringContaining("128 MiB")); expect(tracks).toHaveBeenCalled(); expect(save).not.toHaveBeenCalled();
  });
  it("automatically stops video within the two-minute recording budget", async () => {
    vi.useFakeTimers(); render(<VideoRecorderSelfManaged onCapture={vi.fn()} />);
    await act(async () => { fireEvent.click(screen.getByRole("button", { name: "Start filming" })); });
    expect(Recorder.last.state).toBe("recording");
    await act(async () => { vi.advanceTimersByTime(MAX_VIDEO_DURATION_MS - 1000); });
    expect(Recorder.last.state).toBe("inactive"); expect(tracks).toHaveBeenCalled();
  });
});


describe("voice persistence and article lifetime", () => {
  it("records voice, preserves failed upload, and reopens native audio inline", async () => {
    const create = vi.fn().mockResolvedValue(record); useAnnotationStore.setState({ create });
    const begin = vi.spyOn(annotationService, "assetBegin").mockRejectedValueOnce(new Error("Disk full")).mockResolvedValue({ uploadToken: "token", maximumChunkBytes: 2 });
    vi.spyOn(annotationService, "assetChunk").mockResolvedValue(undefined); vi.spyOn(annotationService, "assetCommit").mockResolvedValue(record);
    const onSaved = vi.fn(); const view = render(<ReaderCanvasCapture kind="voice" selector={selector} draftId="voice" onSaved={onSaved} onCancel={vi.fn()} />);
    fireEvent.click(screen.getByRole("button", { name: "Start recording" })); await screen.findByRole("button", { name: "Stop recording" });
    fireEvent.click(screen.getByRole("button", { name: "Stop recording" })); await screen.findByRole("button", { name: "Attach voice annotation" });
    fireEvent.click(screen.getByRole("button", { name: "Attach voice annotation" })); await screen.findByText(/Upload failed: Disk full/);
    expect(onSaved).not.toHaveBeenCalled(); fireEvent.click(screen.getByRole("button", { name: "Attach voice annotation" })); await waitFor(() => expect(onSaved).toHaveBeenCalledWith(record));
    expect(create).toHaveBeenCalledTimes(1); expect(begin.mock.calls[1]![0]).toMatchObject({ assetKind: "voice_opus", mimeType: "audio/webm;codecs=opus", annotationId: "note" });
    view.unmount();
    const audio = { ...asset, assetKind: "voice_opus" as const, mimeType: "audio/webm;codecs=opus" }; vi.mocked(invoke).mockResolvedValue([audio]);
    const loaded = await loadCanvasAssets([record], resources.urlForEntry("pack", "A/Home"), () => false);
    const reopened = runtime(); reopened.send({ kind: "canvas_annotations_changed", annotations: [{ ...record, assets: loaded.get("note"), colorToken: "highlight-blue" }] });
    expect(reopened.doc.querySelector("audio")?.src).toBe(audio.resourceUrl); expect(reopened.doc.querySelector("audio")?.controls).toBe(true); reopened.close();
  });
  it("rejects late asset metadata after an article change", async () => {
    let resolve!: (value: unknown) => void; vi.mocked(invoke).mockImplementationOnce(() => new Promise(done => { resolve = done; }));
    const response = { title: "Home", rawHtml: "<p>Paragraph text</p>", mimeType: "text/html", internalLinks: [] };
    const props = { article: response, packId: "pack", articleKey: "A/Home", resources, annotations: [record], onSelectionChange: vi.fn(), onOpenArticle: vi.fn() };
    const view = render(<ArticleContent {...props} />);
    view.rerender(<ArticleContent {...props} articleKey="A/Other" annotations={[]} />);
    const frame = view.container.querySelector("iframe")!; const post = vi.spyOn(frame.contentWindow!, "postMessage");
    await act(async () => resolve([asset]));
    expect(post.mock.calls.filter(([message]) => message.kind === "canvas_annotations_changed").every(([message]) => message.annotations.length === 0)).toBe(true);
  });
  it("blocks route changes for an unfinished draft, supports Stay and explicit discard", async () => {
    useReaderStore.setState({ currentPackId: "pack", currentArticleKey: "A/Home", currentPackTitle: "Pack", currentArticle: { status: "ready", data: { title: "Home", rawHtml: "<p>Paragraph text</p>", mimeType: "text/html", internalLinks: [] }, error: null, lastUpdated: null }, loadArticle: vi.fn(async () => undefined) });
    // jsdom AbortSignal and Node's fetch Request are different realms. These
    // loader-free navigation tests do not issue a request; omit only that signal.
    const NativeRequest = Request;
    vi.stubGlobal("Request", class extends NativeRequest { constructor(input: RequestInfo | URL, init?: RequestInit) { super(input, { ...init, signal: undefined }); } });
    const router = createMemoryRouter([{ path: "/reader/:packId/:articleKey", element: <ReaderView /> }, { path: "/away", element: <p>Left article</p> }], { initialEntries: ["/reader/pack/A%2FHome"] });
    const view = render(<RouterProvider router={router} />);
    const frame = view.container.querySelector("iframe")!;
    const script = new DOMParser().parseFromString(frame.srcdoc, "text/html").querySelector("script")!.textContent!;
    const channel = script.match(/\)\("([a-f0-9-]+)"/)?.[1]; expect(channel).toBeTruthy();
    act(() => window.dispatchEvent(new MessageEvent("message", { source: frame.contentWindow, data: { schemaVersion: 1, channel, kind: "canvas_draft_state", draftId: "dirty", dirty: true } })));
    const unload = new Event("beforeunload", { cancelable: true }); window.dispatchEvent(unload); expect(unload.defaultPrevented).toBe(true);
    await act(async () => { await router.navigate("/away"); }); expect(screen.getByRole("dialog", { name: "Unsaved article annotation" })).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Stay" })); expect(router.state.location.pathname).toContain("/reader/");
    let rejectSave!: (error: Error) => void;
    useAnnotationStore.setState({ create: vi.fn(() => new Promise<AnnotationRecord | undefined>((_resolve, reject) => { rejectSave = reject; })) });
    act(() => window.dispatchEvent(new MessageEvent("message", { source: frame.contentWindow, data: { schemaVersion: 1, channel, kind: "canvas_type_commit", draftId: "dirty", selector, bodyText: "Unsaved text" } })));
    await act(async () => { await router.navigate("/away"); });
    expect((screen.getByRole("button", { name: "Discard & leave" }) as HTMLButtonElement).disabled).toBe(true);
    await act(async () => rejectSave(new Error("Disk full"))); await screen.findByText("Disk full");
    fireEvent.click(screen.getByRole("button", { name: "Discard & leave" })); await screen.findByText("Left article");
  });
});


it("ignores reordered article successes, failures and responses after close", async () => {
  const response = { title: "New", rawHtml: "<p>New content</p>", mimeType: "text/html", internalLinks: [] };
  let oldResolve!: (data: typeof response) => void;
  let oldReject!: (error: Error) => void;
  const load = vi.spyOn(readerService, "readerArticle").mockImplementationOnce(() => new Promise(resolve => { oldResolve = resolve; })).mockResolvedValueOnce(response);
  const first = initialReader.loadArticle("pack", "A/Old"); await initialReader.loadArticle("pack", "A/New");
  oldResolve({ ...response, title: "Old" }); await first;
  expect(useReaderStore.getState().currentArticle.data?.title).toBe("New"); expect(useReaderStore.getState().currentArticleKey).toBe("A/New");
  load.mockImplementationOnce(() => new Promise((_resolve, reject) => { oldReject = reject; })).mockResolvedValueOnce(response);
  const failure = initialReader.loadArticle("pack", "A/Fail"); await initialReader.loadArticle("pack", "A/New"); oldReject(new Error("Late failure")); await failure;
  expect(useReaderStore.getState().currentArticle.status).toBe("ready");
  load.mockImplementationOnce(() => new Promise(resolve => { oldResolve = resolve; }));
  const closing = initialReader.loadArticle("pack", "A/Close"); initialReader.clearArticle(); oldResolve(response); await closing;
  expect(useReaderStore.getState().currentArticle.status).toBe("idle"); expect(useReaderStore.getState().currentArticleKey).toBeNull();
});


it("keeps ambiguous anchors visible and reflows restored ink at its resolved anchor", () => {
  const r = runtime("<p>Paragraph text</p><p>Paragraph text</p>");
  r.send({ kind: "canvas_annotations_changed", annotations: [{ ...record, selector: { ...selector, textPosition: undefined }, assets: [asset], colorToken: "highlight-blue" }] });
  expect(r.doc.querySelector("[data-ez-canvas=unresolved]")?.textContent).toContain("Typed note");
  r.send({ kind: "canvas_annotations_changed", annotations: [{ ...record, annotationType: "ink", bodyText: undefined, assets: [{ ...asset, assetKind: "ink_svg", mimeType: "image/svg+xml" }], colorToken: "highlight-blue" }] });
  const saved = r.doc.querySelector<HTMLElement>("[data-ez-canvas=saved]")!;
  expect(saved.style.position).toBe("fixed"); expect(saved.style.width).toBe("300px"); r.resize(); expect(saved.style.width).toBe("200px");
  expect(saved.querySelector("img")?.style.objectFit).toBe("fill"); r.close();
});
