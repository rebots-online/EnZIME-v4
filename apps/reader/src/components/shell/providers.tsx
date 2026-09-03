import {
  Component,
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ErrorInfo,
  type ReactNode,
} from "react";
import { invoke } from "@tauri-apps/api/core";

/* ────────────────────────────────────────────────────────────────────────── */
/* ErrorBoundary — a render-time fault never blanks the app again.            */

type BoundaryState = { error: Error | null };

export class ErrorBoundary extends Component<
  { children: ReactNode },
  BoundaryState
> {
  state: BoundaryState = { error: null };

  static getDerivedStateFromError(error: Error): BoundaryState {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo): void {
    // Surface to the diagnostics pipeline when wired; never silent.
    console.error("EnZIME render fault:", error, info.componentStack ?? "");
  }

  reset = (): void => this.setState({ error: null });

  render(): ReactNode {
    if (this.state.error) {
      return (
        <main className="ez-error-boundary" role="alert">
          <h1>Something broke while rendering</h1>
          <p>{this.state.error.message}</p>
          <button type="button" className="ez-shell__search" onClick={this.reset}>
            Try again
          </button>
        </main>
      );
    }
    return this.props.children;
  }
}

/* ────────────────────────────────────────────────────────────────────────── */
/* ThemeProvider — applies the frozen Aurora Glass theme to the document.     */

type ThemeMode = "dark" | "light";

const THEME_KEY = "enzime.ui.theme";

const ThemeContext = createContext<{ mode: ThemeMode }>({ mode: "dark" });

export function ThemeProvider({ children }: { children: ReactNode }): ReactNode {
  const [mode] = useState<ThemeMode>(() => {
    if (typeof localStorage === "undefined") return "dark";
    const stored = localStorage.getItem(THEME_KEY);
    return stored === "light" ? "light" : "dark";
  });

  useEffect(() => {
    const root = document.documentElement;
    root.dataset.theme = mode;
    root.style.colorScheme = mode;
  }, [mode]);

  const value = useMemo(() => ({ mode }), [mode]);
  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}

/* ────────────────────────────────────────────────────────────────────────── */
/* IpcProvider — single seam to the Tauri command bus (§15.5).                */

export type InvokeFn = typeof invoke;

const IpcContext = createContext<InvokeFn>(invoke);

export function useIpc(): InvokeFn {
  return useContext(IpcContext);
}

export function IpcProvider({ children }: { children: ReactNode }): ReactNode {
  // Single seam to the Tauri command bus (§15.5). `invoke` is a stable
  // module-level binding; services call `useIpc()("command", args)`.
  return <IpcContext.Provider value={invoke}>{children}</IpcContext.Provider>;
}

/* ────────────────────────────────────────────────────────────────────────── */
/* EventBridge — subscribes to the §14.6 event registry and fans out.         */

type EventListener = (payload: unknown) => void;

const listeners = new Map<string, Set<EventListener>>();

export function onAppEvent(event: string, fn: EventListener): () => void {
  let set = listeners.get(event);
  if (!set) {
    set = new Set();
    listeners.set(event, set);
  }
  set.add(fn);
  return () => {
    set?.delete(fn);
    if (set && set.size === 0) listeners.delete(event);
  };
}

export function EventBridge({ children }: { children: ReactNode }): ReactNode {
  // When the Tauri event API is present, attach a single subscription per
  // event name and dispatch to registered listeners. Degrades gracefully when
  // the runtime is web-only (e.g. vite dev without the shell).
  useEffect(() => {
    let detachAll: (() => void) | undefined;
    (async () => {
      try {
        const { listen } = await import("@tauri-apps/api/event");
        const names = Array.from(listeners.keys());
        const detaches = await Promise.all(
          names.map((n) =>
            listen(n, (e) => {
              listeners.get(n)?.forEach((fn) => fn(e.payload));
            }).then((un) => un),
          ),
        );
        detachAll = () => detaches.forEach((d) => d());
      } catch {
        // Web-only runtime: events arrive only once the shell is present.
      }
    })();
    return () => detachAll?.();
  }, []);

  return <>{children}</>;
}

/* ────────────────────────────────────────────────────────────────────────── */
/* ToastRegion — transient, accessible announcements (§15.1 ToastRegion).     */

type Toast = { id: number; body: ReactNode };
type ToastContextValue = { push: (body: ReactNode) => void };

const ToastContext = createContext<ToastContextValue>({ push: () => {} });

export function useToast(): ToastContextValue {
  return useContext(ToastContext);
}

export function ToastProvider({ children }: { children: ReactNode }): ReactNode {
  const [toasts, setToasts] = useState<Toast[]>([]);
  const nextId = useRef(1);

  const push = useCallback((body: ReactNode) => {
    const id = nextId.current++;
    setToasts((cur) => [...cur, { id, body }]);
    window.setTimeout(() => {
      setToasts((cur) => cur.filter((t) => t.id !== id));
    }, 4200);
  }, []);

  const value = useMemo(() => ({ push }), [push]);

  return (
    <ToastContext.Provider value={value}>
      {children}
      <aside className="ez-toast-region" aria-live="polite" aria-atomic="false">
        {toasts.map((t) => (
          <div className="ez-toast" key={t.id} role="status">
            {t.body}
          </div>
        ))}
      </aside>
    </ToastContext.Provider>
  );
}
