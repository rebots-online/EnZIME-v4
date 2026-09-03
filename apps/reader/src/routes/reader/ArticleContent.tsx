// ArticleContent — the §7.3 rendering boundary.
//
// ZIM `rawHtml` is NEVER injected into the privileged React DOM. It is rendered
// inside a sandboxed <iframe> with NO `allow-scripts` token, so article scripts
// cannot execute. Internal article links are rewritten parent-side to reader
// routes and given `target="_top"`; combined with the
// `allow-top-navigation-by-user-activation` sandbox token, a user-initiated
// click on an internal link performs a route change (no scripts involved). The
// iframe is also denied `allow-same-origin`, so it runs at an opaque origin
// isolated from the parent origin's storage.
//
// Defence in depth:
//   1. Parent-side sanitisation (DOMParser; regex fallback): strips <script>,
//      <iframe>/<object>/<embed>/<form>/<link>/<base>, all `on*=` attributes,
//      and `javascript:`/`vbscript:`/`data:text/html` URLs.
//   2. A strict §18.3 Content-Security-Policy injected as a <meta> inside the
//      sandbox document (`default-src 'none'`, `connect-src 'none'`, etc.).
//   3. The `sandbox` token set never grants `allow-scripts` — the decisive
//      guard. Article CSS still renders via the injected <style>.
//
// Note for the architect: article images currently reference relative paths and
// will not resolve until the Rust layer rewrites them to scoped
// `enzime-resource://` URLs (§7.3 step 4). The Literata webfont is passed via
// `font-family` with a serif fallback; hosting it under `enzime-resource:` is a
// post-M1 task.

import { useMemo, type ReactNode } from "react";
import type { ArticleResponse } from "../../services/readerService";
import { normalizeArticleKey } from "../../stores/readerStore";

/**
 * Sandbox tokens. `allow-scripts` and `allow-same-origin` are deliberately
 * absent: article scripts cannot run and the sandbox doc gets an opaque
 * origin. `allow-top-navigation-by-user-activation` lets a user-initiated click
 * on a rewritten internal link navigate the app route.
 */
const SANDBOX_TOKENS = "allow-top-navigation-by-user-activation";

/**
 * §18.3 Article sandbox CSP. `style-src 'unsafe-inline'` permits the injected
 * reading CSS plus any inline article styling; every other capability is
 * locked down.
 */
const ARTICLE_CSP = [
  "default-src 'none'",
  "img-src enzime-resource: data:",
  "media-src enzime-resource:",
  "style-src 'unsafe-inline' enzime-resource:",
  "font-src enzime-resource:",
  "connect-src 'none'",
  "frame-src 'none'",
  "object-src 'none'",
  "form-action 'none'",
  "base-uri 'none'",
].join("; ");

/** Quiet, opaque reading surface — Literata via --ez-font-reader (with fallbacks). */
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

const FORBIDDEN_SELECTORS =
  "script, iframe, object, embed, form, link, base, meta[http-equiv]";

function stripDangerousAttributes(root: ParentNode): void {
  root.querySelectorAll("*").forEach((el) => {
    // Snapshot via Array.from — NamedNodeMap is live.
    for (const attr of Array.from(el.attributes)) {
      const name = attr.name.toLowerCase();
      const value = attr.value.trim();
      if (name.startsWith("on")) {
        el.removeAttribute(attr.name);
        continue;
      }
      if (
        (name === "href" ||
          name === "src" ||
          name === "xlink:href" ||
          name === "action" ||
          name === "formaction" ||
          name === "poster") &&
        /^(javascript:|vbscript:|data:text\/html)/i.test(value)
      ) {
        el.setAttribute(attr.name, "#");
      }
    }
  });
}

function rewriteInternalLinks(root: ParentNode, packId: string): void {
  if (!packId) return;
  root.querySelectorAll("a[href]").forEach((a) => {
    const href = a.getAttribute("href") ?? "";
    const key = normalizeArticleKey(href);
    if (!key) return;
    a.setAttribute("href", `/reader/${packId}/${key}`);
    a.setAttribute("target", "_top");
    a.setAttribute("data-internal", "true");
    a.setAttribute("rel", "noopener");
  });
}

/** Regex fallback used only if DOMParser is unavailable or throws. */
function regexSanitize(rawHtml: string): string {
  let html = rawHtml;
  html = html.replace(
    /<\s*(script|iframe|object|embed|form|link|base)[\s\S]*?<\/\s*\1\s*>/gi,
    "",
  );
  html = html.replace(
    /<\s*(script|iframe|object|embed|form|link|base|meta)[^>]*>/gi,
    "",
  );
  html = html.replace(/\son[a-z]+\s*=\s*("[^"]*"|'[^']*'|[^\s>]+)/gi, "");
  html = html.replace(
    /(href|src|xlink:href|action)\s*=\s*("javascript:[^"]*"|'javascript:[^']*'|javascript:[^\s>]+)/gi,
    '$1="#"',
  );
  return html;
}

function buildSandboxedDoc(rawHtml: string, packId: string): string {
  const hasDomParser =
    typeof DOMParser !== "undefined" && typeof document !== "undefined";

  if (hasDomParser) {
    try {
      const doc = new DOMParser().parseFromString(rawHtml, "text/html");
      doc.querySelectorAll(FORBIDDEN_SELECTORS).forEach((el) => el.remove());
      stripDangerousAttributes(doc);
      rewriteInternalLinks(doc, packId);

      const head = doc.querySelector("head");
      if (head) {
        head
          .querySelectorAll("meta[http-equiv], meta[charset]")
          .forEach((m) => m.remove());
        const csp = doc.createElement("meta");
        csp.setAttribute("http-equiv", "Content-Security-Policy");
        csp.setAttribute("content", ARTICLE_CSP);
        const charset = doc.createElement("meta");
        charset.setAttribute("charset", "utf-8");
        head.prepend(csp);
        head.insertBefore(charset, csp);
        const style = doc.createElement("style");
        style.setAttribute("data-ez-reader", "true");
        style.textContent = READER_CSS;
        head.appendChild(style);
      }
      return "<!DOCTYPE html>" + doc.documentElement.outerHTML;
    } catch {
      // fall through to regex path
    }
  }

  const sanitized = regexSanitize(rawHtml);
  const head = `<meta charset="utf-8"><meta http-equiv="Content-Security-Policy" content="${ARTICLE_CSP}"><style>${READER_CSS}</style>`;
  return `<!DOCTYPE html><html><head>${head}</head><body>${sanitized}</body></html>`;
}

export interface ArticleContentProps {
  readonly article: ArticleResponse;
  readonly packId: string;
}

export function ArticleContent({
  article,
  packId,
}: ArticleContentProps): ReactNode {
  const sandboxedDoc = useMemo(
    () => buildSandboxedDoc(article.rawHtml, packId),
    [article.rawHtml, packId],
  );

  const isHtml =
    article.mimeType.length === 0 || /html|xml|xhtml/i.test(article.mimeType);

  if (!isHtml) {
    // Non-HTML payload — present as preformatted text, never as markup.
    return (
      <pre className="reader-article__fallback" role="article">
        {article.rawHtml}
      </pre>
    );
  }

  return (
    <div className="reader-article">
      {article.internalLinks.length > 0 ? (
        <span className="reader-sr-only">
          Article contains {article.internalLinks.length} internal link
          {article.internalLinks.length === 1 ? "" : "s"}.
        </span>
      ) : null}
      <iframe
        className="reader-article__frame"
        sandbox={SANDBOX_TOKENS}
        srcDoc={sandboxedDoc}
        title={article.title}
      />
    </div>
  );
}
