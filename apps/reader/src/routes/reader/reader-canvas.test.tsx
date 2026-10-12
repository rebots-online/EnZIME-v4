import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, render } from "@testing-library/react";
import { ArticleContent, buildSandboxedDoc } from "./ArticleContent";
import { buildRendererBridgeScript, resolveZimReference, validateRendererMessage } from "./rendererBridge";
import { ReaderStyles } from "./ReaderView";
import type { AnnotationRecord, SelectorBundle } from "../../services/annotationService";

const article = { packId: "pack", articleKey: "A/nested/Café" };
const resources = { urlForEntry: (pack: string, key: string) => `http://enzime-resource.localhost/pack/${pack}/${key.split("/").map(encodeURIComponent).join("/")}`, cspSources: ["http://enzime-resource.localhost"] };
const selector: SelectorBundle = { article, textQuote: { exact: "hello", prefix: "", suffix: "" }, textPosition: { normalizedStart: 0, normalizedEnd: 5 }, capturedContext: "hello" };
afterEach(cleanup);

function runtime(html: string) {
  const iframe = document.createElement("iframe"); document.body.append(iframe);
  const win = iframe.contentWindow!; const doc = win.document;
  doc.body.innerHTML = html;
  // Run only the generated application bridge. This tests its DOM algorithm;
  // jsdom does not enforce native WebView CSP or opaque sandbox origins.
  const receive = vi.spyOn(win.parent, "postMessage").mockImplementation(() => undefined);
  new Function("window", "document", "NodeFilter", buildRendererBridgeScript("channel", article))(win, doc, NodeFilter);
  const send = (data: object, source: MessageEventSource | null = win.parent) => win.dispatchEvent(new MessageEvent("message", { source, data: { schemaVersion: 1, channel: "channel", ...data } }));
  return { win, doc, send, receive, close: () => iframe.remove() };
}

describe("archive references", () => {
  it("resolves nested, Unicode, namespaces and fragments without losing path components", () => {
    expect(resolveZimReference(article.articleKey, "../Autre_%C3%A9t%C3%A9#section")).toEqual({ entryKey: "A/Autre_été", fragment: "section" });
    expect(resolveZimReference(article.articleKey, "#intro")).toEqual({ entryKey: article.articleKey, fragment: "intro" });
    expect(resolveZimReference(article.articleKey, "/I/icon.png")).toEqual({ entryKey: "I/icon.png", fragment: undefined });
    expect(resolveZimReference(article.articleKey, "../Caf%C3%A9%23Menu%3Fa")).toEqual({ entryKey: "A/Café#Menu?a", fragment: undefined });
    expect(resolveZimReference(article.articleKey, "A/Home")).toEqual({ entryKey: "A/Home", fragment: undefined });
  });
  it.each(["javascript:alert(1)", "//evil.test/a", "https://user:pass@evil.test/x", "../../../../secret", "A\\evil", "java%0ascript:bad", "%2f%2fevil.test", "data:text/html,bad", "bad%ZZ"])("rejects %s", reference => expect(resolveZimReference(article.articleKey, reference)).toBeUndefined());
});

describe("renderer trust boundary", () => {
  it("keeps one nonce bridge, strips archive capability, scopes media and rewrites safe CSS", () => {
    const html = buildSandboxedDoc(`<html><head><base href="https://evil.test/"><meta http-equiv="refresh" content="0;url=https://evil.test"><style>.x{background:url('../image.png')} @import 'https://evil.test/a';</style><link rel="stylesheet" href="../../-/style.css"></head><body onload="alert(1)"><script nonce="stolen">alert(1)</script><iframe srcdoc="bad"></iframe><form action="bad"><button>go</button></form><a target="_top" href="../Next">Next</a><img nonce="stolen" src="../photo.png" onerror="alert(1)"><img src="https://evil.test/pixel"><img src="data:image/svg+xml;base64,PHN2Zz4="><p style="background:url('../photo.png')">Text</p><svg><animate onbegin="alert(1)"/></svg></body></html>`, "pack", article.articleKey, "channel", "nonce", resources);
    const doc = new DOMParser().parseFromString(html, "text/html");
    expect(doc.querySelectorAll("script")).toHaveLength(1);
    expect(doc.querySelector("script")?.getAttribute("nonce")).toBe("nonce");
    expect(doc.querySelector("script")?.textContent).not.toContain("alert(1)");
    expect(doc.querySelector("iframe,form,base,svg,[onload],[onerror],[target]")).toBeNull();
    expect(doc.querySelectorAll("[nonce]")).toHaveLength(1);
    expect(doc.querySelector("a")?.getAttribute("data-ez-entry")).toBe("A/Next");
    expect(doc.querySelectorAll("img")[0]?.getAttribute("src")).toBe("http://enzime-resource.localhost/pack/pack/A/photo.png");
    expect(doc.querySelectorAll("img")[1]?.hasAttribute("src")).toBe(false);
    expect(doc.querySelectorAll("img")[2]?.hasAttribute("src")).toBe(false);
    expect(doc.querySelector("p")?.getAttribute("style")).toContain("/pack/pack/A/photo.png");
    const csp = doc.querySelector("meta")?.getAttribute("content");
    expect(csp).toContain("script-src 'nonce-nonce'");
    expect(csp).toContain("connect-src 'none'");
    expect(csp).not.toMatch(/(?:^|\s)https?:[;\s]/);
  });
  it("fails closed when DOMParser fails", () => {
    vi.spyOn(DOMParser.prototype, "parseFromString").mockImplementationOnce(() => { throw new Error("parser unavailable"); });
    const doc = buildSandboxedDoc("<script>bad()</script><img onerror='bad()'>", "pack", "A/Home", "channel", "nonce", resources);
    expect(doc).toContain("could not be rendered safely"); expect(doc).not.toContain("<script"); expect(doc).not.toContain("bad()");
  });
  it("checks exact channel, schema, identity, bounds and fields", () => {
    const message = { schemaVersion: 1, channel: "channel", kind: "selection_changed", selector };
    expect(validateRendererMessage(message, "channel", article)).toEqual(message);
    expect(validateRendererMessage(message, "other", article)).toBeUndefined();
    expect(validateRendererMessage(message, "channel", { ...article, packId: "other" })).toBeUndefined();
    expect(validateRendererMessage({ ...message, command: "invoke" }, "channel", article)).toBeUndefined();
    expect(validateRendererMessage({ ...message, selector: { ...selector, textPosition: { normalizedStart: NaN, normalizedEnd: 5 } } }, "channel", article)).toBeUndefined();
    expect(validateRendererMessage({ ...message, selector: { ...selector, textQuote: { ...selector.textQuote, exact: "x".repeat(16385) } } }, "channel", article)).toBeUndefined();
  });
  it("parent rejects forged sources and old article messages, and uses only allow-scripts", () => {
    const selection = vi.fn();
    const props = { article: { rawHtml: "<p>hello</p>", title: "Hello", mimeType: "text/html", internalLinks: [] }, packId: "pack", articleKey: article.articleKey, resources, annotations: [], onSelectionChange: selection, onOpenArticle: vi.fn() };
    const view = render(<ArticleContent {...props} />);
    const iframe = view.container.querySelector("iframe")!;
    expect(iframe.getAttribute("sandbox")).toBe("allow-scripts");
    const script = new DOMParser().parseFromString(iframe.srcdoc, "text/html").querySelector("script")!.textContent!;
    const channel = script.match(/\)\("([^"]+)",\{/)! [1]!;
    selection.mockClear();
    const data = { schemaVersion: 1, channel, kind: "selection_changed", selector };
    window.dispatchEvent(new MessageEvent("message", { data, source: window }));
    expect(selection).not.toHaveBeenCalled();
    window.dispatchEvent(new MessageEvent("message", { data, source: iframe.contentWindow }));
    expect(selection).toHaveBeenCalledWith(selector);
    view.rerender(<ArticleContent {...props} articleKey="A/Other" />);
    selection.mockClear();
    window.dispatchEvent(new MessageEvent("message", { data, source: iframe.contentWindow }));
    expect(selection).not.toHaveBeenCalled();
  });
  it("accepts placement status only for current supplied annotation IDs without mutating records", () => {
    const record: AnnotationRecord = { annotationId: "note", annotationType: "highlight", selector, style: { colorToken: "highlight-yellow", emphasis: "solid" }, tags: [], sidecarLayerId: "local", resolution: { state: "resolved", confidence: 1, explanation: "Stored result" }, pinned: false, revision: 1, createdAtMs: 1, updatedAtMs: 1 };
    const props = { article: { rawHtml: "<p>hello</p>", title: "Hello", mimeType: "text/html", internalLinks: [] }, packId: "pack", articleKey: article.articleKey, resources, annotations: [record], onSelectionChange: vi.fn(), onOpenArticle: vi.fn() };
    const view = render(<ArticleContent {...props} />);
    const iframe = view.container.querySelector("iframe")!;
    const script = new DOMParser().parseFromString(iframe.srcdoc, "text/html").querySelector("script")!.textContent!;
    const channel = script.match(/\)\("([^"]+)",\{/)! [1]!;
    const message = { schemaVersion: 1, channel, kind: "annotation_resolution", annotationId: "unknown", state: "ambiguous" };
    act(() => window.dispatchEvent(new MessageEvent("message", { source: iframe.contentWindow, data: message })));
    expect(view.queryByRole("status")).toBeNull();
    act(() => window.dispatchEvent(new MessageEvent("message", { source: iframe.contentWindow, data: { ...message, annotationId: "note" } })));
    expect(view.getByRole("status").textContent).toContain("Annotation placement: 1 ambiguous");
    expect(record.resolution.state).toBe("resolved");
    view.rerender(<ArticleContent {...props} annotations={[]} />);
    expect(view.queryByRole("status")).toBeNull();
  });

});

describe("normalized selectors and durable mark replay", () => {
  it("captures formatting-spanning Unicode selections, restores and deletes marks idempotently", () => {
    const r = runtime("<p>Hello <b>world 😀</b> today</p>");
    const range = r.doc.createRange();
    range.setStart(r.doc.querySelector("p")!.firstChild!, 0);
    range.setEnd(r.doc.querySelector("b")!.firstChild!, 8);
    r.win.getSelection()!.addRange(range); r.doc.dispatchEvent(new Event("selectionchange"));
    const message = r.receive.mock.calls.map(c => c[0]).find(m => m.kind === "selection_changed");
    expect(message.selector.textQuote.exact).toBe("Hello world 😀");
    expect(message.selector.textPosition).toEqual({ normalizedStart: 0, normalizedEnd: 14 });
    const record = { annotationId: "note", selector: message.selector, colorToken: "highlight-yellow" };
    r.send({ kind: "annotations_changed", annotations: [record] });
    expect(Array.from(r.doc.querySelectorAll("mark")).map(m => m.textContent).join("")).toBe("Hello world 😀");
    r.send({ kind: "annotations_changed", annotations: [record] });
    expect(r.doc.querySelectorAll("mark mark")).toHaveLength(0);
    expect(r.win.getSelection()!.toString()).toBe("Hello world 😀");
    r.send({ kind: "annotations_changed", annotations: [] });
    expect(r.doc.querySelectorAll("mark")).toHaveLength(0);
    r.close();
    const reopened = runtime("<p>Hello <b>world 😀</b> today</p>");
    reopened.send({ kind: "annotations_changed", annotations: [record] });
    expect(reopened.doc.querySelectorAll("mark").length).toBeGreaterThan(0);
    reopened.close();
  });
  it("does not choose the first ambiguous quote or trust foreign article records", () => {
    const r = runtime("<p>same same</p>");
    const ambiguous = { ...selector, textQuote: { exact: "same", prefix: "", suffix: "" }, textPosition: undefined };
    r.send({ kind: "annotations_changed", annotations: [{ annotationId: "a", selector: ambiguous, colorToken: "highlight-blue" }] });
    expect(r.doc.querySelectorAll("mark")).toHaveLength(0);
    expect(r.receive).toHaveBeenCalledWith({ schemaVersion: 1, channel: "channel", kind: "annotation_resolution", annotationId: "a", state: "ambiguous" }, "*");
    r.send({ kind: "annotations_changed", annotations: [{ annotationId: "a", selector: { ...ambiguous, textQuote: { exact: "same", prefix: "same ", suffix: "" } }, colorToken: "highlight-blue" }] });
    expect(r.doc.querySelector("mark")?.textContent).toBe("same");
    r.send({ kind: "annotations_changed", annotations: [{ annotationId: "b", selector: { ...selector, article: { ...article, articleKey: "A/Else" } }, colorToken: "highlight-blue" }] });
    expect(r.doc.querySelectorAll("mark")).toHaveLength(0);
    r.close();
  });
  it("routes internal links and bounded fragments without navigation or selector injection", () => {
    const r = runtime('<a data-ez-entry="A/Next" data-ez-fragment="part" href="#">Next</a><h2 id="part">Part</h2><a name="legacy"></a>');
    const scroll = vi.fn();
    r.doc.querySelector("h2")!.scrollIntoView = scroll;
    r.doc.querySelector<HTMLAnchorElement>("a")!.click();
    expect(r.receive).toHaveBeenCalledWith({ schemaVersion: 1, channel: "channel", kind: "open_article", articleKey: "A/Next", fragment: "part" }, "*");
    r.send({ kind: "focus_fragment", fragment: "part" }); expect(scroll).toHaveBeenCalledTimes(1);
    r.send({ kind: "focus_fragment", fragment: 'part\"],body' }); expect(scroll).toHaveBeenCalledTimes(1);
    r.send({ kind: "focus_fragment", fragment: "part" }, r.win); expect(scroll).toHaveBeenCalledTimes(1);
    r.close();
  });
  it("retains styles on repeated mounts", () => {
    let view = render(<ReaderStyles />); expect(view.container.querySelector("style")?.textContent).toContain("reader-surface"); view.unmount();
    view = render(<ReaderStyles />); expect(view.container.querySelector("style")?.textContent).toContain("reader-surface");
  });
});
