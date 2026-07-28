import {
  useCallback,
  useEffect,
  useId,
  useRef,
  useState,
  type CSSProperties,
  type PointerEvent as ReactPointerEvent,
} from "react";
import { EzButton, EzIconButton, useReducedMotion } from "./sharedUi";

export interface InkStroke {
  readonly color: string;
  readonly strokeWidth: number;
  readonly points: ReadonlyArray<{ readonly x: number; readonly y: number }>;
}

export interface InkCanvasProps {
  enabled: boolean;
  colorToken: string;
  strokeWidth: number;
  onCommitSvg: (svg: string) => void;
  onCancel: () => void;
}

const DEFAULT_WIDTH = 320;
const DEFAULT_HEIGHT = 240;

function resolveColorToken(token: string): string {
  switch (token) {
    case "highlight-mint":
      return "var(--ez-color-highlight-mint)";
    case "highlight-pink":
      return "var(--ez-color-highlight-pink)";
    case "highlight-blue":
      return "var(--ez-color-highlight-blue)";
    case "highlight-yellow":
    default:
      return "var(--ez-color-highlight-yellow)";
  }
}

export function InkCanvas({
  enabled,
  colorToken,
  strokeWidth,
  onCommitSvg,
  onCancel,
}: InkCanvasProps): JSX.Element {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const [strokes, setStrokes] = useState<InkStroke[]>([]);
  const [active, setActive] = useState<InkStroke | null>(null);
  const [dimensions, setDimensions] = useState({
    width: DEFAULT_WIDTH,
    height: DEFAULT_HEIGHT,
  });
  const reduced = useReducedMotion();
  const headingId = useId();

  useEffect(() => {
    const element = containerRef.current;
    if (!element) {
      return;
    }
    const measure = () => {
      const rect = element.getBoundingClientRect();
      setDimensions({
        width: Math.max(160, Math.round(rect.width)),
        height: DEFAULT_HEIGHT,
      });
    };
    measure();
    if (typeof ResizeObserver === "undefined") {
      return;
    }
    const observer = new ResizeObserver(measure);
    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  const toPoint = useCallback(
    (event: ReactPointerEvent<HTMLCanvasElement>) => {
      const canvas = event.currentTarget;
      const rect = canvas.getBoundingClientRect();
      return {
        x: event.clientX - rect.left,
        y: event.clientY - rect.top,
      };
    },
    [],
  );

  const handlePointerDown = useCallback(
    (event: ReactPointerEvent<HTMLCanvasElement>) => {
      if (!enabled) {
        return;
      }
      event.preventDefault();
      canvasFocusAndARIA(event.currentTarget);
      const point = toPoint(event);
      setActive({
        color: resolveColorToken(colorToken),
        strokeWidth,
        points: [point],
      });
    },
    [colorToken, enabled, strokeWidth, toPoint],
  );

  const handlePointerMove = useCallback(
    (event: ReactPointerEvent<HTMLCanvasElement>) => {
      if (!active) {
        return;
      }
      event.preventDefault();
      const point = toPoint(event);
      setActive({
        ...active,
        points: [...active.points, point],
      });
    },
    [active, toPoint],
  );

  const finishStroke = useCallback(() => {
    if (!active) {
      return;
    }
    setStrokes((previous) => [...previous, active]);
    setActive(null);
  }, [active]);

  const handlePointerUp = useCallback(
    (event: ReactPointerEvent<HTMLCanvasElement>) => {
      if (!active) {
        return;
      }
      event.preventDefault();
      finishStroke();
    },
    [active, finishStroke],
  );

  const handlePointerLeave = useCallback(
    (event: ReactPointerEvent<HTMLCanvasElement>) => {
      if (!active) {
        return;
      }
      event.preventDefault();
      const point = toPoint(event);
      setActive({
        ...active,
        points: [...active.points, point],
      });
      finishStroke();
    },
    [active, finishStroke, toPoint],
  );

  const undo = useCallback(() => {
    setStrokes((previous) => previous.slice(0, -1));
  }, []);

  const clear = useCallback(() => {
    setStrokes([]);
    setActive(null);
  }, []);

  const commit = useCallback(() => {
    if (strokes.length === 0) {
      return;
    }
    const svg = renderSvg(strokes, dimensions.width, dimensions.height);
    onCommitSvg(svg);
    setStrokes([]);
    setActive(null);
  }, [dimensions.height, dimensions.width, onCommitSvg, strokes]);

  const liveStrokes = active ? [...strokes, active] : strokes;
  const description = describeCanvas(strokes.length, enabled);

  return (
    <section
      aria-labelledby={headingId}
      style={panelStyle}
      data-ez-ink-canvas=""
    >
      <header style={headerStyle}>
        <h3 id={headingId} style={headingStyle}>
          Ink annotation
        </h3>
        <span
          style={{
            color: enabled
              ? "var(--ez-color-success)"
              : "var(--ez-color-text-muted)",
            fontFamily: "var(--ez-font-mono)",
            fontSize: "0.8rem",
          }}
        >
          {enabled ? "● Tool active" : "○ Tool inactive"}
        </span>
      </header>
      <div ref={containerRef} style={canvasFrameStyle}>
        <canvas
          tabIndex={enabled ? 0 : -1}
          role="img"
          aria-label="Ink drawing surface"
          aria-description={description}
          onPointerDown={handlePointerDown}
          onPointerMove={handlePointerMove}
          onPointerUp={handlePointerUp}
          onPointerLeave={handlePointerLeave}
          style={{
            width: "100%",
            height: dimensions.height,
            touchAction: "none",
            cursor: enabled ? "crosshair" : "default",
            transition: reduced ? "none" : "box-shadow var(--ez-duration-fast)",
            boxShadow: enabled
              ? "inset 0 0 0 2px var(--ez-color-border-focus)"
              : "inset 0 0 0 1px var(--ez-color-border-subtle)",
          }}
        />
        <InkOverlay strokes={liveStrokes} dimensions={dimensions} />
      </div>
      <p
        style={{
          margin: 0,
          color: "var(--ez-color-text-muted)",
          fontSize: "0.875rem",
        }}
      >
        {description}
      </p>
      <footer style={footerStyle}>
        <EzIconButton
          ariaLabel="Undo last stroke"
          onClick={undo}
          disabled={strokes.length === 0}
          title="Undo"
        >
          ↺
        </EzIconButton>
        <EzIconButton
          ariaLabel="Clear all strokes"
          onClick={clear}
          disabled={strokes.length === 0}
          title="Clear"
        >
          ✕
        </EzIconButton>
        <span style={{ flex: 1 }} />
        <EzButton
          variant="ghost"
          onClick={onCancel}
          ariaLabel="Cancel ink annotation"
        >
          Cancel
        </EzButton>
        <EzButton
          onClick={commit}
          disabled={strokes.length === 0}
          ariaLabel="Save ink annotation"
        >
          Save ink
        </EzButton>
      </footer>
    </section>
  );
}

function InkOverlay({
  strokes,
  dimensions,
}: {
  strokes: InkStroke[];
  dimensions: { width: number; height: number };
}): JSX.Element {
  if (strokes.length === 0) {
    return <span aria-hidden="true" style={{ display: "none" }} />;
  }
  return (
    <svg
      aria-hidden="true"
      viewBox={`0 0 ${dimensions.width} ${dimensions.height}`}
      style={{
        position: "absolute",
        inset: 0,
        pointerEvents: "none",
      }}
    >
      {strokes.map((stroke, strokeIndex) => (
        <path
          key={strokeIndex}
          d={pathFromStroke(stroke)}
          stroke={stroke.color}
          strokeWidth={stroke.strokeWidth}
          strokeLinecap="round"
          strokeLinejoin="round"
          fill="none"
        />
      ))}
    </svg>
  );
}

function canvasFocusAndARIA(canvas: HTMLCanvasElement): void {
  if (typeof canvas.focus === "function") {
    canvas.focus();
  }
}

function pathFromStroke(stroke: InkStroke): string {
  if (stroke.points.length === 0) {
    return "";
  }
  const [first, ...rest] = stroke.points;
  const segments: string[] = [`M ${first.x.toFixed(2)} ${first.y.toFixed(2)}`];
  for (const point of rest) {
    segments.push(`L ${point.x.toFixed(2)} ${point.y.toFixed(2)}`);
  }
  return segments.join(" ");
}

function renderSvg(
  strokes: InkStroke[],
  width: number,
  height: number,
): string {
  const paths = strokes
    .map((stroke) => {
      const d = pathFromStroke(stroke);
      return `<path d="${d}" stroke="${cssColorToRgb(
        stroke.color,
      )}" stroke-width="${stroke.strokeWidth}" stroke-linecap="round" stroke-linejoin="round" fill="none" />`;
    })
    .join("");
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${width} ${height}" width="${width}" height="${height}">${paths}</svg>`;
}

function cssColorToRgb(token: string): string {
  if (token.startsWith("var(")) {
    return "currentColor";
  }
  return token;
}

function describeCanvas(strokeCount: number, enabled: boolean): string {
  if (!enabled) {
    return "Ink tool is inactive. Select the ink tool to draw.";
  }
  if (strokeCount === 0) {
    return "Draw with mouse, touch, or stylus. Strokes are not saved until you choose Save ink.";
  }
  return `${strokeCount} stroke${strokeCount === 1 ? "" : "s"} ready. Choose Save ink to attach.`;
}

const panelStyle: CSSProperties = {
  background: "var(--ez-color-bg-glass)",
  borderRadius: "var(--ez-radius-md)",
  padding: "var(--ez-space-4)",
  display: "flex",
  flexDirection: "column",
  gap: "var(--ez-space-3)",
};

const headerStyle: CSSProperties = {
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  gap: "var(--ez-space-3)",
};

const headingStyle: CSSProperties = {
  margin: 0,
  fontSize: "0.95rem",
  letterSpacing: "0.04em",
  textTransform: "uppercase",
  color: "var(--ez-color-text-secondary)",
};

const canvasFrameStyle: CSSProperties = {
  position: "relative",
  background: "var(--ez-color-bg-elevated)",
  borderRadius: "var(--ez-radius-sm)",
  padding: "var(--ez-space-2)",
  display: "flex",
};

const footerStyle: CSSProperties = {
  display: "flex",
  alignItems: "center",
  gap: "var(--ez-space-2)",
};
