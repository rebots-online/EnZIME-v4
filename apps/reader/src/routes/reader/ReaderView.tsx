// ReaderView — §15.1 / §15.3 `/reader/:packId/:articleKey` route.
//
// Composition: ReaderView → BreadcrumbNav, ArticleToolbar, ReaderPane
// (ArticleContent). It reads route params, drives readerStore.loadArticle, and
// renders the loading / error / populated / empty states using the shared
// Aurora Glass primitives (resolveState, StatusBanner, SkeletonBlock,
// RetryButton). It also registers a pluggable navigator so the store's
// `navigate(href)` can drive client-side route changes without importing the
// router (which would create a circular dependency).
//
// The article surface is OPAQUE and quiet (Literata via --ez-font-reader);
// Aurora Glass is reserved for the chrome (AppShell, toolbar, breadcrumb).

import { useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { useNavigate, useParams, useBlocker } from "react-router-dom";
import { useReaderStore, setRouteNavigator } from "../../stores/readerStore";
import {
  resolveState,
  RetryButton,
  SkeletonBlock,
  StatusBanner,
} from "../../components/home/shared";
import { BreadcrumbNav } from "./BreadcrumbNav";
import { ArticleToolbar } from "./ArticleToolbar";
import { ArticleContent, type CanvasDraftReply } from "./ArticleContent";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useAnnotationStore } from "../../stores/annotationStore";
import type { SelectorBundle } from "../../services/annotationService";
import { ReaderCanvasTools, type ReaderCanvasTool } from "./ReaderCanvasTools";
import { ReaderCanvasCapture } from "./ReaderCanvasCapture";
import { saveCanvasDraft, type CanvasSaveSession } from "./canvasPersistence";
import type { CanvasDraftMessage } from "./rendererBridge";
import { AnnotationDrawer } from "./AnnotationDrawer";

import { ReaderStyles } from "./ReaderStyles";
export { ReaderStyles } from "./ReaderStyles";

export function ReaderView(): ReactNode {
  const params = useParams<{ packId: string; articleKey: string }>();
  const routePackId = params.packId ?? "";
  const routeArticleKey = params.articleKey ?? "";

  const navigate = useNavigate();
  const [selection, setSelection] = useState<SelectorBundle>();
  const annotations = useAnnotationStore(s => s.annotations);
  const selectedAnnotationId = useAnnotationStore(s => s.selectedAnnotationId);
  const [tool, setTool] = useState<ReaderCanvasTool>("select");
  const [dirtyDraftId, setDirtyDraftId] = useState<string>();
  const dirtyRef = useRef<string | undefined>(undefined);
  const [canvasBusy, setCanvasBusy] = useState(false);
  const busyRef = useRef(false);
  const [canvasError, setCanvasError] = useState<string>();
  const [discardDraftId, setDiscardDraftId] = useState<string>();
  const [requestedTool, setRequestedTool] = useState<ReaderCanvasTool>();
  const [reloadRequested, setReloadRequested] = useState(false);
  const [capture, setCapture] = useState<{ kind: "photo" | "video" | "voice"; selector: SelectorBundle; draftId: string; reply: (result: CanvasDraftReply) => void }>();
  const saves = useRef<CanvasSaveSession>({ pending: new Map(), busy: new Set() });
  const blocker = useBlocker(!!dirtyDraftId || canvasBusy);
  function setSaving(value: boolean) { busyRef.current = value; setCanvasBusy(value); }
  function discardCanvas() {
    if (busyRef.current) return;
    const id = dirtyRef.current;
    if (id) { setDiscardDraftId(id); saves.current.pending.delete(id); }
    dirtyRef.current = undefined; setDirtyDraftId(undefined); setCapture(undefined); setCanvasError(undefined);
  }
  useEffect(() => {
    if (!dirtyDraftId && !canvasBusy) return;
    const guard = (event: BeforeUnloadEvent) => { event.preventDefault(); event.returnValue = ""; };
    window.addEventListener("beforeunload", guard); return () => window.removeEventListener("beforeunload", guard);
  }, [dirtyDraftId, canvasBusy]);
  async function onCanvasMessage(message: CanvasDraftMessage, reply: (result: CanvasDraftReply) => void) {
    if (message.kind === "canvas_draft_state") {
      if (message.dirty && !dirtyRef.current) { dirtyRef.current = message.draftId; setDirtyDraftId(message.draftId); }
      if (!message.dirty && dirtyRef.current === message.draftId && !busyRef.current) { dirtyRef.current = undefined; setDirtyDraftId(undefined); }
      return;
    }
    if (message.draftId !== dirtyRef.current || busyRef.current) return;
    if (message.kind === "canvas_capture_request") {
      setCapture({ kind: message.captureKind, selector: message.selector, draftId: message.draftId, reply }); return;
    }
    setSaving(true); setCanvasError(undefined);
    try {
      await saveCanvasDraft(saves.current, message.kind === "canvas_type_commit" ? { kind: "type", draftId: message.draftId, selector: message.selector, bodyText: message.bodyText } : { kind: "draw", draftId: message.draftId, selector: message.selector, strokes: message.strokes, width: message.width, height: message.height });
      reply({ kind: "draft_result", draftId: message.draftId, saved: true });
      if (dirtyRef.current === message.draftId) { dirtyRef.current = undefined; setDirtyDraftId(undefined); }
    } catch (cause) { const error = cause instanceof Error ? cause.message : String(cause); setCanvasError(error); reply({ kind: "draft_result", draftId: message.draftId, saved: false, error }); }
    finally { setSaving(false); }
  }
  async function saveHighlight() {
    if (!selection || busyRef.current) return;
    setSaving(true); setCanvasError(undefined);
    try { await saveCanvasDraft(saves.current, { kind: "highlight", draftId: crypto.randomUUID(), selector: selection }); }
    catch (cause) { setCanvasError(cause instanceof Error ? cause.message : String(cause)); }
    finally { setSaving(false); }
  }

  const resources = useMemo(() => {
    const base = convertFileSrc("", "enzime-resource").replace(/\/$/, "");
    const url = new URL(base);
    return {
      urlForEntry: (packId: string, entryKey: string) => `${base}/pack/${encodeURIComponent(packId)}/${entryKey.split("/").map(encodeURIComponent).join("/")}`,
      cspSources: [url.protocol === "http:" || url.protocol === "https:" ? url.origin : "enzime-resource:"],
    };
  }, []);
  useEffect(() => { setSelection(undefined); }, [routePackId, routeArticleKey]);

  const currentPackId = useReaderStore((s) => s.currentPackId);
  const currentPackTitle = useReaderStore((s) => s.currentPackTitle);
  const currentArticleKey = useReaderStore((s) => s.currentArticleKey);
  const currentArticle = useReaderStore((s) => s.currentArticle);
  const loadArticle = useReaderStore((s) => s.loadArticle);

  // Register the router navigator so the store can resolve+drive article links.
  useEffect(() => {
    setRouteNavigator((path: string) => navigate(path));
    return () => setRouteNavigator(null);
  }, [navigate]);

  // Load whenever the route's (packId, articleKey) differs from what is loaded.
  // A given position is fetched exactly once; explicit retry is driven by the
  // ArticleToolbar / error-state RetryButton (which calls loadArticle directly),
  // so this effect must not re-fire on the same errored position (that would
  // loop).
  useEffect(() => {
    if (!routePackId || !routeArticleKey) return;
    // React Router has already decoded this path segment.
    const key = routeArticleKey;
    const samePosition =
      currentPackId === routePackId && currentArticleKey === key;
    if (samePosition) return;
    void loadArticle(routePackId, key);
  }, [
    routePackId,
    routeArticleKey,
    currentPackId,
    currentArticleKey,
    loadArticle,
  ]);

  const routeMatchesArticle = currentPackId === routePackId && currentArticleKey === routeArticleKey;
  const state = routeMatchesArticle ? resolveState(
    true,
    currentArticle,
    (data) => data.rawHtml.trim().length > 0,
  ) : "loading";
  const article = currentArticle.data;

  const goHome = () => navigate("/");

  let pane: ReactNode;
  if (state === "loading") {
    pane = (
      <div className="reader-pane__loading" aria-live="polite">
        <SkeletonBlock lines={4} />
      </div>
    );
  } else if (state === "populated" && article) {
    pane = (
      <>
        <ArticleContent article={article} packId={routePackId} articleKey={currentArticleKey ?? routeArticleKey}
          resources={resources} annotations={annotations} selectedAnnotationId={selectedAnnotationId}
          tool={tool} discardDraftId={discardDraftId} onCanvasMessage={(message, reply) => void onCanvasMessage(message, reply)}
          onSelectionChange={setSelection}
          onOpenArticle={(key, fragment) => navigate(`/reader/${encodeURIComponent(routePackId)}/${encodeURIComponent(key)}${fragment ? `#${encodeURIComponent(fragment)}` : ""}`)} />
        {currentArticleKey ? (
          <AnnotationDrawer
            packId={routePackId}
            articleKey={currentArticleKey}
            title={article.title}
            selection={selection}
            key={`${routePackId}/${currentArticleKey}`}
          />
        ) : null}
      </>
    );
  } else if (state === "empty") {
    pane = (
      <div className="reader-pane__empty">
        <StatusBanner
          tone="info"
          icon="i"
          title="This article is empty"
          detail="The ZIM entry resolved, but contains no renderable content."
          actions={
            <button
              type="button"
              className="reader-chip reader-chip--ghost"
              onClick={goHome}
            >
              Back to Home
            </button>
          }
        />
      </div>
    );
  } else if (state === "recoverable") {
    pane = (
      <div className="reader-pane__error">
        <StatusBanner
          tone="warning"
          icon="!"
          title="Could not load article"
          detail={currentArticle.error?.message ?? "Unexpected error."}
          actions={
            <RetryButton
              onClick={() =>
                void loadArticle(routePackId, routeArticleKey || "")
              }
              label="Try again"
            />
          }
        />
      </div>
    );
  } else if (state === "non-recoverable") {
    pane = (
      <div className="reader-pane__error">
        <StatusBanner
          tone="error"
          icon="×"
          title="Article unavailable"
          detail={
            currentArticle.error?.message ??
            "The article could not be read from this pack."
          }
          actions={
            <button
              type="button"
              className="reader-chip reader-chip--ghost"
              onClick={goHome}
            >
              Back to Home
            </button>
          }
        />
      </div>
    );
  } else {
    // capability-denied
    pane = (
      <div className="reader-pane__error">
        <StatusBanner
          tone="locked"
          icon="∅"
          title="Article locked"
          detail={
            currentArticle.error?.message ??
            "This profile cannot read articles from this pack."
          }
          actions={
            <button
              type="button"
              className="reader-chip reader-chip--ghost"
              onClick={goHome}
            >
              Back to Home
            </button>
          }
        />
      </div>
    );
  }

  const busy = state === "loading";

  return (
    <main className="reader-surface" aria-busy={busy}>
      <ReaderStyles />
      <BreadcrumbNav
        packId={routePackId}
        packTitle={currentPackTitle}
        articleTitle={article?.title ?? routeArticleKey ?? "Article"}
        onHome={goHome}
      />
      <ArticleToolbar
        articleKey={routeArticleKey}
        mimeType={article?.mimeType ?? ""}
        onHome={goHome}
        onReload={() =>
          dirtyDraftId || canvasBusy ? setReloadRequested(true) : void loadArticle(routePackId, routeArticleKey || "")
        }
      />
      <ReaderCanvasTools tool={tool} busy={canvasBusy} onToolChange={next => { if (dirtyDraftId) setRequestedTool(next); else setTool(next); }} />
      {tool !== "select" && tool !== "highlight" ? <p>Click a passage or image to {tool === "draw" ? "draw over it" : tool === "type" ? "type an anchored note" : "attach a capture"}.</p> : null}
      {tool === "highlight" ? <button className="reader-chip" type="button" disabled={!selection || canvasBusy} onClick={() => void saveHighlight()}>Save selected highlight</button> : null}
      {canvasError ? <p role="alert">{canvasError}</p> : null}
      {(blocker.state === "blocked" || requestedTool || reloadRequested) ? <section role="dialog" aria-modal="true" aria-label="Unsaved article annotation" style={{ padding: "1rem", background: "var(--ez-color-bg-elevated)", border: "1px solid var(--ez-color-border-subtle)" }}>
        <p>{canvasBusy ? "Annotation is saving. Stay here until it finishes." : "This article has an unfinished annotation. Stay to finish it, or discard the draft."}</p>
        <button type="button" onClick={() => { if (blocker.state === "blocked") blocker.reset(); setRequestedTool(undefined); setReloadRequested(false); }}>Stay</button>
        <button type="button" disabled={canvasBusy} onClick={() => {
          discardCanvas();
          if (blocker.state === "blocked") blocker.proceed();
          if (requestedTool) { setTool(requestedTool); setRequestedTool(undefined); }
          if (reloadRequested) { setReloadRequested(false); void loadArticle(routePackId, routeArticleKey || ""); }
        }}>Discard &amp; {blocker.state === "blocked" ? "leave" : requestedTool ? "switch tool" : "reload"}</button>
      </section> : null}
      {capture ? <ReaderCanvasCapture key={capture.draftId} kind={capture.kind} selector={capture.selector} draftId={capture.draftId} onBusyChange={setSaving} onCancel={discardCanvas} onSaved={() => {
        capture.reply({ kind: "draft_result", draftId: capture.draftId, saved: true }); setCapture(undefined); dirtyRef.current = undefined; setDirtyDraftId(undefined);
      }} /> : null}
      <div className="reader-pane" aria-busy={busy}>
        {pane}
      </div>
    </main>
  );
}
