import type { SelectorBundle } from "../../services/annotationService";

export interface ReaderResourceContext {
  urlForEntry: (packId: string, entryKey: string) => string;
  cspSources: readonly string[];
}
export interface RendererHighlight {
  annotationId: string;
  selector: SelectorBundle;
  colorToken: string;
}
type Envelope = { schemaVersion: 1; channel: string };
export type RendererMessageV1 = Envelope & (
  | { kind: "ready" }
  | { kind: "open_article"; articleKey: string; fragment?: string }
  | { kind: "selection_changed"; selector?: SelectorBundle }
  | { kind: "resource_failed"; entryKey: string; reason: string }
  | { kind: "annotation_resolution"; annotationId: string; state: "resolved" | "ambiguous" | "orphaned" }
  | { kind: "scroll_position"; scrollAnchor: string; progressFraction: number }
);
export type RendererParentMessageV1 = Envelope & (
  | { kind: "annotations_changed"; annotations: readonly RendererHighlight[] }
  | { kind: "focus_annotation"; annotationId: string }
  | { kind: "focus_fragment"; fragment: string }
);

export function resolveZimReference(articleKey: string, reference: string): { entryKey: string; fragment?: string } | undefined {
  if (reference.length > 4096 || /[\\\u0000-\u001f\u007f]/u.test(reference) || reference.startsWith("//")) return;
  const hash = reference.indexOf("#");
  const pathPart = (hash < 0 ? reference : reference.slice(0, hash)).split("?")[0]!;
  let path: string, fragment: string | undefined;
  try {
    path = decodeURIComponent(pathPart);
    fragment = hash < 0 ? undefined : decodeURIComponent(reference.slice(hash + 1));
  } catch { return; }
  if (/[\\\u0000-\u001f\u007f]/u.test(path) || path.startsWith("//") || /^[a-z][\w+.-]*:/i.test(path)) return;
  if (!path) return { entryKey: articleKey, fragment };
  const absolute = path.startsWith("/") || /^[A-Za-z-]\//.test(path);
  const parts = absolute ? [] : articleKey.split("/").slice(0, -1);
  for (const part of path.split("/")) {
    if (!part || part === ".") continue;
    if (part === "..") { if (!parts.length) return; parts.pop(); }
    else parts.push(part);
  }
  if (parts.length < 2 || !/^[A-Za-z-]$/.test(parts[0]!)) return;
  return { entryKey: parts.join("/"), fragment };
}

export function validateRendererMessage(value: unknown, channel: string, article: { packId: string; articleKey: string }): RendererMessageV1 | undefined {
  const object = (v: unknown): v is Record<string, unknown> => !!v && typeof v === "object" && !Array.isArray(v);
  const text = (v: unknown, limit: number): v is string => typeof v === "string" && v.length <= limit;
  const fields = (v: Record<string, unknown>, allowed: string[]) => Object.keys(v).every(k => allowed.includes(k));
  if (!object(value) || value.schemaVersion !== 1 || value.channel !== channel) return;
  const base = ["schemaVersion", "channel", "kind"];
  switch (value.kind) {
    case "ready": if (!fields(value, base)) return; break;
    case "open_article":
      if (!fields(value, [...base, "articleKey", "fragment"]) || !text(value.articleKey, 4096) || resolveZimReference(article.articleKey, "/" + value.articleKey.split("/").map(encodeURIComponent).join("/"))?.entryKey !== value.articleKey || (value.fragment !== undefined && !text(value.fragment, 4096))) return;
      break;
    case "selection_changed": {
      if (!fields(value, [...base, "selector"])) return;
      if (value.selector === undefined) break;
      const s = value.selector;
      if (!object(s) || !fields(s, ["article", "textQuote", "textPosition", "capturedContext"]) || !object(s.article) || !fields(s.article, ["packId", "articleKey"]) || s.article.packId !== article.packId || s.article.articleKey !== article.articleKey || !text(s.capturedContext, 16640)) return;
      const q = s.textQuote, p = s.textPosition;
      if (!object(q) || !fields(q, ["exact", "prefix", "suffix"]) || !text(q.exact, 16384) || !q.exact.length || !text(q.prefix, 128) || !text(q.suffix, 128) || !object(p) || !fields(p, ["normalizedStart", "normalizedEnd"])) return;
      if (!Number.isSafeInteger(p.normalizedStart) || !Number.isSafeInteger(p.normalizedEnd) || (p.normalizedStart as number) < 0 || (p.normalizedEnd as number) - (p.normalizedStart as number) !== q.exact.length) return;
      break;
    }
    case "annotation_resolution":
      if (!fields(value, [...base, "annotationId", "state"]) || !text(value.annotationId, 128) || !value.annotationId.length || !["resolved", "ambiguous", "orphaned"].includes(value.state as string)) return;
      break;
    case "resource_failed":
      if (!fields(value, [...base, "entryKey", "reason"]) || !text(value.entryKey, 4096) || !text(value.reason, 512)) return;
      break;
    case "scroll_position":
      if (!fields(value, [...base, "scrollAnchor", "progressFraction"]) || !text(value.scrollAnchor, 4096) || typeof value.progressFraction !== "number" || !Number.isFinite(value.progressFraction) || value.progressFraction < 0 || value.progressFraction > 1) return;
      break;
    default: return;
  }
  return value as RendererMessageV1;
}

// Self-contained: serialized into the opaque document; no imports, native API,
// eval, HTML injection or references to the privileged shell are available.
function rendererRuntime(channel: string, article: { packId: string; articleKey: string }) {
  const send = (payload: object) => window.parent.postMessage({ schemaVersion: 1, channel, ...payload }, "*");
  type Point = { node: Text; offset: number; end: number };
  function stream() {
    const points: Point[] = [];
    let text = "";
    const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
    let node: Node | null;
    while ((node = walker.nextNode())) {
      const t = node as Text;
      if (t.parentElement?.closest("script,style,[hidden],[aria-hidden='true'],[data-ez-bridge]")) continue;
      for (let i = 0; i < t.data.length; i++) {
        const c = t.data[i]!;
        if (/\s/u.test(c)) {
          if (text.endsWith(" ")) { const last = points[points.length - 1]!; if (last.node === t) last.end = i + 1; continue; }
          text += " ";
        } else text += c;
        points.push({ node: t, offset: i, end: i + 1 });
      }
    }
    return { text, points };
  }
  const splitsPair = (text: string, offset: number) => offset > 0 && offset < text.length && /[\uD800-\uDBFF]/.test(text[offset - 1]!) && /[\uDC00-\uDFFF]/.test(text[offset]!);
  function capture(): SelectorBundle | undefined {
    const selection = window.getSelection();
    if (!selection || selection.isCollapsed || !selection.rangeCount) return;
    const range = selection.getRangeAt(0);
    if (!document.body.contains(range.commonAncestorContainer)) return;
    const { text, points } = stream();
    let start = -1, end = -1;
    for (let i = 0; i < points.length; i++) {
      const p = points[i]!;
      if (range.comparePoint(p.node, p.end) >= 0 && range.comparePoint(p.node, p.offset) <= 0) {
        // Exact touching endpoints are not selected characters.
        if (range.startContainer === p.node && range.startOffset >= p.end) continue;
        if (range.endContainer === p.node && range.endOffset <= p.offset) continue;
        if (start < 0) start = i;
        end = i + 1;
      }
    }
    if (start < 0 || end <= start) return;
    if (splitsPair(text, start)) start--;
    if (splitsPair(text, end)) end++;
    if (end - start > 16384) return;
    const prefix = text.slice(Math.max(0, start - 128 + (splitsPair(text, start - 128) ? 1 : 0)), start);
    const suffix = text.slice(end, end + 128 - (splitsPair(text, end + 128) ? 1 : 0));
    return { article, textQuote: { exact: text.slice(start, end), prefix, suffix }, textPosition: { normalizedStart: start, normalizedEnd: end }, capturedContext: text.slice(start, end) };
  }
  function locate(selector: SelectorBundle, text: string): { state: "resolved" | "ambiguous" | "orphaned"; range?: [number, number] } {
    const q = selector.textQuote, p = selector.textPosition;
    if (!q || typeof q.exact !== "string" || !q.exact.length || q.exact.length > 16384 || typeof q.prefix !== "string" || q.prefix.length > 128 || typeof q.suffix !== "string" || q.suffix.length > 128) return { state: "orphaned" };
    if (p && Number.isSafeInteger(p.normalizedStart) && Number.isSafeInteger(p.normalizedEnd) && p.normalizedStart >= 0 && p.normalizedEnd > p.normalizedStart && text.slice(p.normalizedStart, p.normalizedEnd) === q.exact && !splitsPair(text, p.normalizedStart) && !splitsPair(text, p.normalizedEnd)) return { state: "resolved", range: [p.normalizedStart, p.normalizedEnd] };
    const matches: number[] = [];
    for (let i = text.indexOf(q.exact); i >= 0; i = text.indexOf(q.exact, i + 1)) {
      if ((!q.prefix || text.slice(0, i).endsWith(q.prefix)) && (!q.suffix || text.slice(i + q.exact.length).startsWith(q.suffix)) && !splitsPair(text, i) && !splitsPair(text, i + q.exact.length)) matches.push(i);
      if (matches.length > 1) return { state: "ambiguous" };
    }
    return matches.length === 1 ? { state: "resolved", range: [matches[0]!, matches[0]! + q.exact.length] } : { state: "orphaned" };
  }
  function focusFragment(fragment: string) {
    const target = document.getElementById(fragment) ?? Array.from(document.getElementsByName(fragment)).find(el => el.tagName === "A");
    target?.scrollIntoView();
  }
  let painting = false;
  function restoreSelection(saved: SelectorBundle | undefined) {
    if (!saved) return;
    const { text, points } = stream();
    const match = locate(saved, text).range;
    if (!match) return;
    const first = points[match[0]], last = points[match[1] - 1];
    if (!first || !last) return;
    const range = document.createRange();
    range.setStart(first.node, first.offset); range.setEnd(last.node, last.end);
    const selection = window.getSelection(); selection?.removeAllRanges(); selection?.addRange(range);
  }
  function highlights(records: readonly RendererHighlight[]) {
    const saved = capture();
    painting = true;
    document.querySelectorAll("mark[data-ez-annotation]").forEach(el => el.replaceWith(...Array.from(el.childNodes)));
    document.body.normalize();
    const colors: Record<string, string> = { "highlight-yellow": "#ffe58a", "highlight-mint": "#97e6be", "highlight-pink": "#ffc0d5", "highlight-blue": "#a9cdff" };
    for (const record of records) {
      if (!record || typeof record.annotationId !== "string" || record.annotationId.length > 128 || record.selector?.article?.packId !== article.packId || record.selector?.article?.articleKey !== article.articleKey) continue;
      const { text, points } = stream();
      const placement = locate(record.selector, text);
      send({ kind: "annotation_resolution", annotationId: record.annotationId, state: placement.state });
      const match = placement.range;
      if (!match) continue;
      // Mark each text-node segment separately, retaining the article's DOM.
      const segments: { node: Text; start: number; end: number }[] = [];
      for (let i = match[0]; i < match[1]; i++) {
        const p = points[i]; if (!p) continue;
        const last = segments[segments.length - 1];
        if (last?.node === p.node) last.end = p.end;
        else segments.push({ node: p.node, start: p.offset, end: p.end });
      }
      for (const segment of segments.reverse()) {
        const range = document.createRange(); range.setStart(segment.node, segment.start); range.setEnd(segment.node, segment.end);
        const mark = document.createElement("mark"); mark.dataset.ezAnnotation = record.annotationId;
        mark.style.backgroundColor = colors[record.colorToken] ?? colors["highlight-yellow"]!; mark.style.color = "#18202b";
        range.surroundContents(mark);
      }
    }
    restoreSelection(saved);
    // Selectionchange is asynchronous in WebViews; preserve pending selections
    // rather than emitting a spurious cleared selection while repainting marks.
    window.setTimeout(() => { painting = false; }, 0);
  }
  let scrollPending = false;
  window.addEventListener("scroll", () => {
    if (scrollPending) return;
    scrollPending = true;
    window.setTimeout(() => {
      scrollPending = false;
      const extent = Math.max(0, document.documentElement.scrollHeight - window.innerHeight);
      send({ kind: "scroll_position", scrollAnchor: "", progressFraction: extent ? Math.min(1, Math.max(0, window.scrollY / extent)) : 0 });
    }, 100);
  }, { passive: true });
  document.addEventListener("selectionchange", () => { if (!painting) send({ kind: "selection_changed", selector: capture() }); });
  document.addEventListener("click", event => {
    const anchor = (event.target as Element | null)?.closest?.("a");
    if (!anchor) return;
    event.preventDefault();
    const entryKey = anchor.getAttribute("data-ez-entry"), fragment = anchor.getAttribute("data-ez-fragment") ?? undefined;
    if (!entryKey) return;
    if (entryKey === article.articleKey && fragment !== undefined) { focusFragment(fragment); return; }
    send({ kind: "open_article", articleKey: entryKey, fragment });
  });
  document.addEventListener("error", event => {
    const entryKey = (event.target as Element | null)?.getAttribute?.("data-ez-resource");
    if (entryKey) send({ kind: "resource_failed", entryKey, reason: "Resource unavailable in this downloaded content." });
  }, true);
  window.addEventListener("message", event => {
    const m = event.data;
    if (event.source !== window.parent || !m || m.schemaVersion !== 1 || m.channel !== channel) return;
    if (m.kind === "focus_fragment" && typeof m.fragment === "string" && m.fragment.length <= 4096) focusFragment(m.fragment);
    if (m.kind === "annotations_changed" && Array.isArray(m.annotations) && m.annotations.length <= 2000) highlights(m.annotations);
    if (m.kind === "focus_annotation" && typeof m.annotationId === "string" && m.annotationId.length <= 128) {
      for (const mark of document.querySelectorAll<HTMLElement>("mark[data-ez-annotation]")) if (mark.dataset.ezAnnotation === m.annotationId) { mark.scrollIntoView({ block: "center" }); break; }
    }
  });
  send({ kind: "ready" });
}

export function buildRendererBridgeScript(channel: string, article: { packId: string; articleKey: string }): string {
  const safe = (v: unknown) => JSON.stringify(v).replace(/</g, "\\u003c").replace(/\u2028/g, "\\u2028").replace(/\u2029/g, "\\u2029");
  return `(${rendererRuntime.toString()})(${safe(channel)},${safe(article)});`;
}
