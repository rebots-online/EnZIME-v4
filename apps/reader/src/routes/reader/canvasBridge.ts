import type { SelectorBundle } from "../../services/annotationService";
import type { CanvasStroke, RendererCanvasAnnotation } from "./rendererBridge";
import type { ReaderCanvasTool } from "./ReaderCanvasTools";
interface CanvasHelpers {
  stream(): { text: string; points: { node: Text; offset: number; end: number }[] };
  locate(selector: SelectorBundle, text: string): { state: "resolved" | "ambiguous" | "orphaned"; range?: [number, number] };
  rangeSelector(range: Range): SelectorBundle | undefined;
}
// Serialized together with rendererRuntime. Keep all executable dependencies
// inside this function; the iframe receives only bounded data from its parent.
export function installCanvasBridge(channel: string, article: { packId: string; articleKey: string }, helpers: CanvasHelpers) {
  const send = (payload: object) => window.parent.postMessage({ schemaVersion: 1, channel, ...payload }, "*");
  let tool: ReaderCanvasTool = "select";
  let desiredTool: ReaderCanvasTool = "select";
  let draft: { id: string; selector: SelectorBundle; root: HTMLElement; saving: boolean; failed: boolean; anchor: Element; place?: () => void } | undefined;
  const saved = new Map<string, { card: HTMLElement; annotation: RendererCanvasAnnotation; place?: () => void }>();
  const unresolved = document.createElement("section"); unresolved.dataset.ezCanvas = "unresolved"; unresolved.setAttribute("aria-label", "Annotations with unresolved placement"); unresolved.hidden = true; document.body.append(unresolved);
  function button(label: string, action: () => void): HTMLButtonElement {
    const node = document.createElement("button"); node.type = "button"; node.textContent = label;
    node.style.cssText = "font:600 14px system-ui;padding:8px 12px;border:1px solid #8797b0;border-radius:8px;background:#f5f8ff;color:#172334;cursor:pointer";
    node.addEventListener("click", action); return node;
  }
  function selectorFor(element: Element): SelectorBundle | undefined {
    if (element.tagName === "IMG") {
      const key = element.getAttribute("data-ez-resource");
      if (key) return { article, imageRegion: { resourceEntryKey: key, xFraction: 0, yFraction: 0, widthFraction: 1, heightFraction: 1 }, capturedContext: (element.getAttribute("alt") ?? "").slice(0, 16384) };
    }
    const range = document.createRange(); range.selectNodeContents(element); return helpers.rangeSelector(range);
  }
  function anchorFor(selector: SelectorBundle): Element | undefined {
    if (selector.imageRegion) {
      const matches = Array.from(document.querySelectorAll("img[data-ez-resource]")).filter(el => !el.closest("[data-ez-canvas]") && el.getAttribute("data-ez-resource") === selector.imageRegion!.resourceEntryKey);
      return matches.length === 1 ? matches[0] : undefined;
    }
    const { text, points } = helpers.stream(); const match = helpers.locate(selector, text).range;
    if (!match) return;
    return points[match[0]]?.node.parentElement?.closest("p,li,h1,h2,h3,h4,h5,h6,blockquote,td,figcaption,pre,div") ?? points[match[0]]?.node.parentElement ?? undefined;
  }
  function styleCard(node: HTMLElement) {
    node.style.cssText = "position:relative;box-sizing:border-box;padding:12px;margin:12px 0;border:1px solid #8296b8;border-radius:10px;background:#f3f7ff;color:#172334;font:15px/1.5 system-ui;max-width:100%;overflow-wrap:anywhere";
  }
  function fixedPlacement(node: HTMLElement, anchor: Element) {
    return () => {
      const rect = anchor.getBoundingClientRect();
      node.style.position = "fixed"; node.style.zIndex = "10";
      node.style.left = `${rect.left}px`; node.style.top = `${rect.top}px`;
      node.style.width = `${Math.max(1, rect.width)}px`; node.style.height = `${Math.max(40, rect.height)}px`;
    };
  }
  function clearDraft(notify = true) {
    if (!draft) return;
    const id = draft.id; draft.root.remove(); draft = undefined; tool = desiredTool;
    if (notify) send({ kind: "canvas_draft_state", draftId: id, dirty: false });
  }
  function setSaving() {
    if (!draft) return;
    draft.saving = true;
    draft.root.querySelectorAll<HTMLInputElement | HTMLButtonElement | HTMLTextAreaElement>("button,input,textarea").forEach(el => { el.disabled = true; });
  }
  function createDraft(anchor: Element, selector: SelectorBundle) {
    const id = crypto.randomUUID(); const root = document.createElement("section"); root.dataset.ezCanvas = "draft"; root.setAttribute("aria-label", "Article annotation draft"); styleCard(root);
    draft = { id, selector, root, saving: false, failed: false, anchor };
    send({ kind: "canvas_draft_state", draftId: id, dirty: true });
    if (tool === "photo" || tool === "video" || tool === "voice") {
      root.textContent = "Capture is open in the reader. Finish or discard it before choosing another anchor."; anchor.after(root);
      send({ kind: "canvas_capture_request", draftId: id, selector, captureKind: tool }); return;
    }
    if (tool === "type") {
      const input = document.createElement("textarea"); input.maxLength = 16384; input.placeholder = "Type a note at this passage…"; input.setAttribute("aria-label", "Type annotation on article"); input.style.cssText = "box-sizing:border-box;width:100%;min-height:100px;font:16px/1.5 system-ui;background:white;color:#172334;border:1px solid #8296b8";
      const controls = document.createElement("div"); controls.style.cssText = "display:flex;gap:8px;margin-top:8px";
      controls.append(button("Save note", () => { if (!draft || draft.saving || !input.value.trim()) return; setSaving(); send({ kind: "canvas_type_commit", selector, bodyText: input.value, draftId: id }); }), button("Discard note", () => clearDraft()));
      root.append(input, controls); anchor.after(root); input.focus(); return;
    }
    if (tool === "draw") {
      const rect = anchor.getBoundingClientRect(); const width = Math.min(4096, Math.max(1, Math.round(rect.width))), height = Math.min(4096, Math.max(40, Math.round(rect.height)));
      const canvas = document.createElement("canvas"); canvas.width = width; canvas.height = height; canvas.setAttribute("aria-label", "Draw over article"); canvas.style.cssText = "width:100%;height:100%;touch-action:none;display:block;cursor:crosshair;background:#eef4ff22;outline:2px solid #2365b4";
      const strokes: CanvasStroke[] = []; let active: CanvasStroke | undefined; let count = 0;
      const ctx = canvas.getContext("2d");
      const point = (event: PointerEvent) => { const box = canvas.getBoundingClientRect(); return { x: Math.max(0, Math.min(width, (event.clientX - box.left) * width / Math.max(1, box.width))), y: Math.max(0, Math.min(height, (event.clientY - box.top) * height / Math.max(1, box.height))) }; };
      function paint() { if (!ctx) return; ctx.clearRect(0, 0, width, height); ctx.strokeStyle = "#2365b4"; ctx.lineWidth = 3; ctx.lineCap = "round"; ctx.lineJoin = "round"; for (const stroke of strokes) { ctx.beginPath(); stroke.points.forEach((p, i) => { if (i) ctx.lineTo(p.x, p.y); else ctx.moveTo(p.x, p.y); }); if (stroke.points.length === 1) { const p = stroke.points[0]!; ctx.lineTo(p.x + 0.01, p.y); } ctx.stroke(); } }
      canvas.addEventListener("pointerdown", event => {
        if (!draft || draft.saving || draft.failed || strokes.length >= 1000 || count >= 100000) return;
        event.preventDefault(); canvas.setPointerCapture?.(event.pointerId); active = { points: [point(event)], colorToken: "highlight-blue", width: 3 }; count++; strokes.push(active); paint();
      });
      canvas.addEventListener("pointermove", event => { if (!active || count >= 100000 || draft?.saving) return; event.preventDefault(); active.points.push(point(event)); count++; paint(); });
      const finish = () => { active = undefined; }; canvas.addEventListener("pointerup", finish); canvas.addEventListener("pointercancel", finish);
      const controls = document.createElement("div"); controls.style.cssText = "display:flex;gap:8px;padding:6px;background:#f3f7ff;border:1px solid #8296b8";
      controls.append(button("Save drawing", () => { if (!draft || draft.saving || !strokes.length) return; finish(); setSaving(); send({ kind: "canvas_ink_commit", draftId: id, selector, strokes, width, height }); }), button("Discard drawing", () => clearDraft()));
      root.style.cssText = "box-sizing:border-box;padding:0;margin:0"; root.append(canvas, controls); document.body.append(root); draft.place = fixedPlacement(root, anchor); draft.place();
    }
  }
  function replay(annotations: readonly RendererCanvasAnnotation[]) {
    const ids = new Set(annotations.map(a => a.annotationId));
    for (const [id, entry] of saved) if (!ids.has(id)) { entry.card.remove(); saved.delete(id); }
    for (const annotation of annotations) {
      if (annotation.selector.article.packId !== article.packId || annotation.selector.article.articleKey !== article.articleKey || annotation.annotationType === "highlight") continue;
      saved.get(annotation.annotationId)?.card.remove();
      const card = document.createElement("aside"); card.dataset.ezCanvas = "saved"; card.dataset.ezAnnotationCard = annotation.annotationId; card.setAttribute("aria-label", "Saved article annotation"); styleCard(card);
      if (annotation.bodyText && !(annotation.annotationType === "ink" && annotation.bodyText.trim().startsWith("<svg"))) { const text = document.createElement("p"); text.textContent = annotation.bodyText; text.style.whiteSpace = "pre-wrap"; card.append(text); }
      for (const asset of annotation.assets) {
        let media: HTMLImageElement | HTMLVideoElement | HTMLAudioElement | undefined;
        if (asset.mimeType.startsWith("image/")) { const img = document.createElement("img"); img.alt = annotation.annotationType === "ink" ? "Saved drawing on article" : "Annotation photograph"; img.loading = "lazy"; media = img; }
        else if (asset.mimeType.startsWith("video/")) { const video = document.createElement("video"); video.controls = true; video.preload = "none"; media = video; }
        else if (asset.mimeType.startsWith("audio/")) { const audio = document.createElement("audio"); audio.controls = true; audio.preload = "none"; media = audio; }
        if (media) { media.src = asset.resourceUrl; media.style.cssText = "display:block;max-width:100%;max-height:70vh"; card.append(media); }
      }
      if (!card.childNodes.length) { const pending = document.createElement("p"); pending.textContent = "Annotation saved; attachment is not available."; card.append(pending); }
      const anchor = anchorFor(annotation.selector);
      const entry: { card: HTMLElement; annotation: RendererCanvasAnnotation; place?: () => void } = { card, annotation };
      if (anchor) {
        if (annotation.annotationType === "ink" && annotation.assets.length) {
          card.style.cssText = "pointer-events:none;box-sizing:border-box;margin:0;padding:0;background:transparent";
          card.querySelectorAll("img").forEach(img => { img.style.cssText = "width:100%;height:100%;object-fit:fill"; });
          document.body.append(card); entry.place = fixedPlacement(card, anchor); entry.place();
        } else anchor.after(card);
      } else {
        const notice = document.createElement("p"); notice.textContent = "Original anchor is unavailable or ambiguous. Annotation retained here."; card.prepend(notice); unresolved.append(card);
      }
      saved.set(annotation.annotationId, entry);
    }
    unresolved.hidden = !unresolved.childElementCount;
  }
  function reflow() {
    draft?.place?.();
    for (const entry of saved.values()) {
      const anchor = anchorFor(entry.annotation.selector);
      if (anchor && entry.place) { entry.place = fixedPlacement(entry.card, anchor); entry.place(); }
    }
  }
  window.addEventListener("resize", reflow); window.addEventListener("scroll", reflow, { passive: true });
  if (typeof ResizeObserver !== "undefined") new ResizeObserver(reflow).observe(document.body);
  document.addEventListener("click", event => {
    if (tool === "select" || tool === "highlight" || draft) return;
    const target = event.target as Element | null;
    if (!target || target.closest("[data-ez-canvas]")) return;
    const anchor = target.closest("img,p,li,h1,h2,h3,h4,h5,h6,blockquote,td,figcaption,pre");
    if (!anchor) return;
    const selector = selectorFor(anchor); if (!selector) return;
    event.preventDefault(); event.stopImmediatePropagation(); createDraft(anchor, selector);
  }, true);
  window.addEventListener("message", event => {
    const message = event.data;
    if (event.source !== window.parent || !message || message.channel !== channel || message.schemaVersion !== 1) return;
    if (message.kind === "tool_changed" && ["select", "highlight", "draw", "type", "voice", "photo", "video"].includes(message.tool)) { desiredTool = message.tool; if (!draft) tool = message.tool; }
    if (message.kind === "canvas_annotations_changed" && Array.isArray(message.annotations) && message.annotations.length <= 2000) replay(message.annotations);
    if (message.kind === "discard_draft" && draft && draft.id === message.draftId && !draft.saving) clearDraft();
    if (message.kind === "draft_result" && draft && draft.id === message.draftId) {
      if (message.saved === true) clearDraft();
      else {
        draft.saving = false; draft.failed = true;
        draft.root.querySelectorAll<HTMLButtonElement>("button").forEach(el => { el.disabled = false; });
        const error = draft.root.querySelector<HTMLElement>("[role=alert]") ?? document.createElement("p"); error.setAttribute("role", "alert"); error.textContent = typeof message.error === "string" ? message.error.slice(0, 512) : "Save failed. Retry or discard."; draft.root.append(error);
      }
    }
    if (message.kind === "focus_annotation") {
      const entry = saved.get(message.annotationId);
      if (entry) (anchorFor(entry.annotation.selector) ?? entry.card).scrollIntoView({ block: "center" });
    }
  });
}
