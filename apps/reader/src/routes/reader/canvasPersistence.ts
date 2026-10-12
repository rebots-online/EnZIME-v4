import { annotationAssets, sha256Hex, uploadAnnotationAsset, type AnnotationRecord, type ReaderAnnotationAsset, type SelectorBundle } from "../../services/annotationService";
import { useAnnotationStore } from "../../stores/annotationStore";
import type { CanvasStroke } from "./rendererBridge";

const colors: Record<string, string> = { "highlight-yellow": "#d49b00", "highlight-mint": "#12794d", "highlight-pink": "#c63466", "highlight-blue": "#2365b4" };
export function canvasInkSvg(strokes: readonly CanvasStroke[], width: number, height: number): string {
  if (![width, height].every(n => Number.isFinite(n) && n >= 1 && n <= 4096) || !strokes.length || strokes.length > 1000) throw new Error("Invalid drawing dimensions or stroke count.");
  let points = 0;
  const paths = strokes.map(stroke => {
    if (!colors[stroke.colorToken] || !Number.isFinite(stroke.width) || stroke.width < 0.5 || stroke.width > 32 || !stroke.points.length) throw new Error("Invalid drawing style.");
    points += stroke.points.length;
    if (points > 100000) throw new Error("Drawing exceeds the point limit.");
    const d = stroke.points.map((point, i) => {
      if (![point.x, point.y].every(Number.isFinite) || point.x < 0 || point.x > width || point.y < 0 || point.y > height) throw new Error("Drawing point is outside the canvas.");
      return `${i ? "L" : "M"}${point.x.toFixed(2)} ${point.y.toFixed(2)}`;
    }).join(" ");
    return `<path d="${d}${stroke.points.length === 1 ? " l0.01 0" : ""}" stroke="${colors[stroke.colorToken]}" stroke-width="${stroke.width}" fill="none" stroke-linecap="round" stroke-linejoin="round"/>`;
  }).join("");
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${width} ${height}" width="${width}" height="${height}">${paths}</svg>`;
  if (svg.length > 8 * 1024 * 1024) throw new Error("Drawing exceeds the encoded size limit.");
  return svg;
}
export interface CanvasSaveSession { pending: Map<string, { annotation: AnnotationRecord; fingerprint: string }>; busy: Set<string> }
export interface CanvasSaveRequest {
  draftId: string; selector: SelectorBundle; kind: "highlight" | "type" | "draw" | "photo" | "video" | "voice";
  bodyText?: string; strokes?: readonly CanvasStroke[]; width?: number; height?: number;
  capture?: { bytes: Uint8Array; mimeType: string; durationMs?: number };
}
export async function saveCanvasDraft(session: CanvasSaveSession, request: CanvasSaveRequest): Promise<AnnotationRecord> {
  if (session.busy.has(request.draftId)) throw new Error("This annotation is already saving.");
  session.busy.add(request.draftId);
  try {
    const store = useAnnotationStore.getState();
    const layer = store.layers.find(candidate => candidate.visibility === "private") ?? store.layers[0];
    if (!layer) throw new Error("No writable annotation layer is available.");
    if (request.bodyText && request.bodyText.length > 16384) throw new Error("Annotation text exceeds the limit.");
    let asset = request.capture;
    if (request.kind === "draw") asset = { bytes: new TextEncoder().encode(canvasInkSvg(request.strokes ?? [], request.width ?? 0, request.height ?? 0)), mimeType: "image/svg+xml; charset=utf-8" };
    if ((request.kind === "photo" || request.kind === "video" || request.kind === "voice") && !asset) throw new Error("Capture bytes are missing.");
    const digest = asset ? await sha256Hex(asset.bytes) : undefined;
    const fingerprint = JSON.stringify({ kind: request.kind, selector: request.selector, bodyText: request.bodyText, digest, mimeType: asset?.mimeType, durationMs: asset?.durationMs });
    let pending = session.pending.get(request.draftId);
    if (pending && pending.fingerprint !== fingerprint) throw new Error("Discard the failed attachment before changing its content.");
    if (!pending) {
      const annotation = await store.create({ annotationType: request.kind === "draw" ? "ink" : request.kind === "highlight" ? "highlight" : "comment", selector: request.selector, bodyText: request.bodyText, style: { colorToken: "highlight-blue", emphasis: "solid", ...(request.kind === "draw" ? { strokeWidth: 3 } : {}) }, sidecarLayerId: layer.sidecarLayerId, tagIds: [] });
      if (!annotation) throw new Error("Annotation could not be saved. Your draft is retained.");
      pending = { annotation, fingerprint }; session.pending.set(request.draftId, pending);
    }
    let record = pending.annotation;
    if (asset) record = await uploadAnnotationAsset({ annotationId: record.annotationId, assetKind: request.kind === "draw" ? "ink_svg" : request.kind === "photo" ? "image" : request.kind === "voice" ? "voice_opus" : "file", mimeType: asset.mimeType, bytes: asset.bytes, durationMs: asset.durationMs, sha256Hex: digest! });
    useAnnotationStore.setState(state => ({ annotations: [record, ...state.annotations.filter(existing => existing.annotationId !== record.annotationId)] }));
    session.pending.delete(request.draftId);
    return record;
  } finally { session.busy.delete(request.draftId); }
}

export function validAnnotationAsset(asset: ReaderAnnotationAsset, annotationId: string, protocolBase: string): boolean {
  if (!asset || asset.annotationId !== annotationId || !asset.annotationAssetId || asset.annotationAssetId.length > 128 || !Number.isSafeInteger(asset.byteLength) || asset.byteLength < 0 || asset.byteLength > 128 * 1024 * 1024) return false;
  try {
    const actual = new URL(asset.resourceUrl), base = new URL(protocolBase);
    const expectedPath = `/annotation/${encodeURIComponent(annotationId)}/${encodeURIComponent(asset.annotationAssetId)}`;
    return actual.protocol === base.protocol && actual.host === base.host && actual.pathname === expectedPath && !actual.search && !actual.hash && !actual.username && !actual.password && /^(?:image\/(?:jpeg|png|gif|webp|svg\+xml)|audio\/(?:ogg|webm)|video\/(?:webm|mp4|ogg))(?:\s*;[^\r\n]*)?$/i.test(asset.mimeType);
  } catch { return false; }
}
export async function loadCanvasAssets(records: readonly AnnotationRecord[], protocolBase: string, cancelled: () => boolean): Promise<Map<string, ReaderAnnotationAsset[]>> {
  const result = new Map<string, ReaderAnnotationAsset[]>();
  let next = 0;
  const selected = records.slice(0, 2000);
  await Promise.all(Array.from({ length: Math.min(4, selected.length) }, async () => {
    while (!cancelled()) {
      const record = selected[next++]; if (!record) return;
      const assets = await annotationAssets(record.annotationId);
      if (!cancelled()) result.set(record.annotationId, assets.filter(asset => validAnnotationAsset(asset, record.annotationId, protocolBase)));
    }
  }));
  return result;
}
