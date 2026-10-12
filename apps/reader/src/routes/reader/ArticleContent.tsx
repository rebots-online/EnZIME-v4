import { useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import type { ArticleResponse } from "../../services/readerService";
import type { AnnotationRecord, SelectorBundle } from "../../services/annotationService";
import { buildRendererBridgeScript, resolveZimReference, validateRendererMessage, type ReaderResourceContext } from "./rendererBridge";

const READER_CSS = `
  :root { color-scheme: light dark; }
  html, body {
    margin: 0;
    padding: 0;
    background: #ffffff;
    color: #1a1d28;
  }
  body {
    font-family: "Literata", Georgia, "Times New Roman", serif;
    font-size: 1.0625rem;
    line-height: 1.7;
    padding: clamp(1rem, 4vw, 2.5rem) clamp(1rem, 6vw, 4rem);
    max-width: 46rem;
    margin: 0 auto;
    word-wrap: break-word;
  }
  @media (prefers-color-scheme: dark) {
    html, body { background: #0e1118; color: #e8eaf2; }
    a { color: #9cc4ff; }
  }
  img, picture, figure, video, audio, table, blockquote, pre, hr {
    max-width: 100%;
    height: auto;
  }
  h1, h2, h3, h4 { line-height: 1.3; }
  a { color: #2a5db0; }
  a[data-internal="true"] { text-decoration: underline; }
  @media (prefers-reduced-motion: reduce) {
    * { animation: none !important; transition: none !important; }
  }
`;


function rewriteCss(css: string, articleKey: string, packId: string, resources: ReaderResourceContext): string {
  // Decode escapes before recognizing URLs. Unsupported/dynamic URL constructs
  // fail closed; CSP independently denies network and active content.
  const decoded = css.replace(/\\([0-9a-f]{1,6})\s?/gi, (_, hex: string) => String.fromCodePoint(Math.min(parseInt(hex, 16), 0x10ffff))).replace(/\\(.)/g, "$1").replace(/\/\*[\s\S]*?\*\//g, "");
  if (/image-set|expression|behavior|binding|[<>]/i.test(decoded)) return "";
  const withImports = decoded.replace(/@import\s+(?:url\(\s*["']?([^"')]+)["']?\s*\)|"([^"]+)"|'([^']+)')\s*([^;]*);/gi, (_, url: string, quoted: string, single: string, media: string) => {
    const target = resolveZimReference(articleKey, (url ?? quoted ?? single).trim());
    if (!target || /[^a-z0-9():.\s,-]/i.test(media)) return "";
    return `@import "${resources.urlForEntry(packId, target.entryKey)}" ${media};`;
  });
  return withImports.replace(/url\(\s*(?:"([^"]*)"|'([^']*)'|([^)]*))\s*\)/gi, (_, a: string, b: string, c: string) => {
    const target = resolveZimReference(articleKey, (a ?? b ?? c ?? "").trim());
    return target ? `url("${resources.urlForEntry(packId, target.entryKey).replace(/["\\\n\r]/g, "")}")` : 'url("")';
  });
}

export function buildSandboxedDoc(rawHtml: string, packId: string, articleKey: string, channel: string, nonce: string, resources: ReaderResourceContext): string {
  try {
    const doc = new DOMParser().parseFromString(rawHtml, "text/html");
    doc.querySelectorAll("script,iframe,frame,frameset,object,embed,form,base,meta,svg,math,template,portal,input,button,textarea,select,noscript").forEach(el => el.remove());
    const passiveData = (value: string) => value.length <= 1024 * 1024 && /^data:image\/(png|jpeg|gif|webp);base64,[a-z0-9+/=]+$/i.test(value);
    const resource = (value: string, el: Element): string | undefined => {
      if (passiveData(value) && el.tagName === "IMG") return value;
      const target = resolveZimReference(articleKey, value);
      if (!target) return;
      el.setAttribute("data-ez-resource", target.entryKey);
      return resources.urlForEntry(packId, target.entryKey) + (target.fragment ? `#${encodeURIComponent(target.fragment)}` : "");
    };
    const safeAttrs = new Set("id class title lang dir role hidden alt width height colspan rowspan scope datetime controls preload muted loop loading decoding type rel style href src srcset poster".split(" "));
    for (const el of Array.from(doc.querySelectorAll("*"))) {
      for (const attr of Array.from(el.attributes)) {
        const name = attr.name.toLowerCase();
        if (!safeAttrs.has(name) && !name.startsWith("aria-")) el.removeAttribute(attr.name);
      }
      if (el.tagName === "LINK" && el.getAttribute("rel")?.toLowerCase() !== "stylesheet") { el.remove(); continue; }
      const href = el.getAttribute("href");
      if (href !== null) {
        el.removeAttribute("href");
        if (el.tagName === "A") {
          const target = resolveZimReference(articleKey, href);
          if (target) {
            el.setAttribute("href", "#"); el.setAttribute("data-ez-entry", target.entryKey);
            if (target.fragment !== undefined) el.setAttribute("data-ez-fragment", target.fragment);
          }
        } else if (el.tagName === "LINK") {
          const url = resource(href, el); if (url) el.setAttribute("href", url); else el.remove();
        }
      }
      for (const name of ["src", "poster"]) {
        const value = el.getAttribute(name); if (value === null) continue;
        const url = resource(value, el); if (url) el.setAttribute(name, url); else el.removeAttribute(name);
      }
      const srcset = el.getAttribute("srcset");
      if (srcset !== null) {
        // Data URLs are deliberately excluded from srcset's comma grammar.
        const sources = srcset.split(",").map(candidate => {
          const [url, descriptor, ...extra] = candidate.trim().split(/\s+/);
          if (!url || extra.length || (descriptor && !/^\d+(?:\.\d+)?[wx]$/.test(descriptor))) return "";
          const rewritten = resource(url, el);
          return rewritten ? `${rewritten}${descriptor ? ` ${descriptor}` : ""}` : "";
        }).filter(Boolean);
        if (sources.length) el.setAttribute("srcset", sources.join(", ")); else el.removeAttribute("srcset");
      }
      const style = el.getAttribute("style");
      if (style !== null) el.setAttribute("style", rewriteCss(style, articleKey, packId, resources));
      if (el.tagName === "STYLE") el.textContent = rewriteCss(el.textContent ?? "", articleKey, packId, resources);
    }
    const sources = resources.cspSources.filter(source => /^(?:enzime-resource:|https?:\/\/[a-z0-9.-]+(?::[0-9]+)?)$/i.test(source)).join(" ");
    const policy = `default-src 'none'; script-src 'nonce-${nonce}'; style-src 'unsafe-inline' ${sources}; img-src ${sources} data:; media-src ${sources}; font-src ${sources}; connect-src 'none'; frame-src 'none'; object-src 'none'; form-action 'none'; base-uri 'none'`;
    const csp = doc.createElement("meta"); csp.httpEquiv = "Content-Security-Policy"; csp.content = policy; doc.head.prepend(csp);
    const style = doc.createElement("style"); style.textContent = READER_CSS; doc.head.append(style);
    const bridge = doc.createElement("script"); bridge.setAttribute("nonce", nonce); bridge.dataset.ezBridge = "true";
    bridge.textContent = buildRendererBridgeScript(channel, { packId, articleKey }); doc.body.append(bridge);
    return "<!DOCTYPE html>" + doc.documentElement.outerHTML;
  } catch {
    // No regex sanitizer: a parser failure must never enable original markup.
    return `<!DOCTYPE html><html><head><meta http-equiv="Content-Security-Policy" content="default-src 'none'"></head><body><p>Article could not be rendered safely.</p></body></html>`;
  }
}

export interface ArticleContentProps {
  readonly article: ArticleResponse;
  readonly packId: string;
  readonly articleKey: string;
  readonly resources: ReaderResourceContext;
  readonly annotations: readonly AnnotationRecord[];
  readonly onSelectionChange: (selection?: SelectorBundle) => void;
  readonly onOpenArticle: (articleKey: string, fragment?: string) => void;
  readonly selectedAnnotationId?: string;
}

export function ArticleContent(props: ArticleContentProps): ReactNode {
  const frame = useRef<HTMLIFrameElement>(null);
  const [resourceError, setResourceError] = useState<string>();
  const [placement, setPlacement] = useState<Record<string, "resolved" | "ambiguous" | "orphaned">>({});
  const session = useMemo(() => ({ channel: crypto.randomUUID(), nonce: crypto.randomUUID() }), [props.packId, props.articleKey, props.article.rawHtml]);
  const doc = useMemo(() => buildSandboxedDoc(props.article.rawHtml, props.packId, props.articleKey, session.channel, session.nonce, props.resources), [props.article.rawHtml, props.packId, props.articleKey, session, props.resources]);
  const latest = useRef(props); latest.current = props;
  function sendAnnotations() {
    const annotations = latest.current.annotations.filter(record => record.selector.article.packId === props.packId && record.selector.article.articleKey === props.articleKey && !!record.selector.textQuote?.exact).slice(0, 2000).map(record => ({ annotationId: record.annotationId, selector: record.selector, colorToken: record.style.colorToken }));
    frame.current?.contentWindow?.postMessage({ kind: "annotations_changed", annotations, schemaVersion: 1, channel: session.channel }, "*");
  }
  useEffect(() => {
    setResourceError(undefined); setPlacement({}); latest.current.onSelectionChange(undefined);
    const receive = (event: MessageEvent) => {
      if (event.source !== frame.current?.contentWindow) return;
      const message = validateRendererMessage(event.data, session.channel, { packId: props.packId, articleKey: props.articleKey });
      if (!message) return;
      if (message.kind === "ready") {
        sendAnnotations();
        let fragment = window.location.hash.slice(1);
        try { fragment = decodeURIComponent(fragment); } catch { fragment = ""; }
        if (fragment.length <= 4096) frame.current?.contentWindow?.postMessage({ kind: "focus_fragment", fragment, schemaVersion: 1, channel: session.channel }, "*");
      }
      if (message.kind === "selection_changed") latest.current.onSelectionChange(message.selector);
      if (message.kind === "open_article") latest.current.onOpenArticle(message.articleKey, message.fragment);
      if (message.kind === "annotation_resolution" && latest.current.annotations.some(record => record.annotationId === message.annotationId && record.selector.article.packId === props.packId && record.selector.article.articleKey === props.articleKey)) {
        setPlacement(current => current[message.annotationId] === message.state ? current : { ...current, [message.annotationId]: message.state });
      }
      if (message.kind === "resource_failed") setResourceError(message.reason);
    };
    window.addEventListener("message", receive); return () => window.removeEventListener("message", receive);
  }, [session, props.packId, props.articleKey]);
  useEffect(() => { sendAnnotations(); }, [props.annotations, session]);
  useEffect(() => {
    if (props.selectedAnnotationId) frame.current?.contentWindow?.postMessage({ kind: "focus_annotation", annotationId: props.selectedAnnotationId, schemaVersion: 1, channel: session.channel }, "*");
  }, [props.selectedAnnotationId, session]);
  if (props.article.mimeType && !/html|xml|xhtml/i.test(props.article.mimeType)) return <pre className="reader-article__fallback" role="article">{props.article.rawHtml}</pre>;
  const active = props.annotations.filter(record => record.selector.article.packId === props.packId && record.selector.article.articleKey === props.articleKey);
  const ambiguous = active.filter(record => placement[record.annotationId] === "ambiguous").length;
  const orphaned = active.filter(record => placement[record.annotationId] === "orphaned").length;
  return <div className="reader-article">
    {ambiguous || orphaned ? <p role="status">Annotation placement: {ambiguous} ambiguous, {orphaned} unavailable in this article. Saved annotations are unchanged.</p> : null}
    {resourceError ? <p role="status">{resourceError}</p> : null}
    <iframe ref={frame} key={session.channel} className="reader-article__frame" sandbox="allow-scripts" srcDoc={doc} title={props.article.title} />
  </div>;
}
