export type ReaderCanvasTool = "select" | "highlight" | "draw" | "type" | "voice" | "photo" | "video";
export interface ReaderCanvasToolsProps { tool: ReaderCanvasTool; onToolChange: (tool: ReaderCanvasTool) => void; busy: boolean }
export function ReaderCanvasTools({ tool, onToolChange, busy }: ReaderCanvasToolsProps) {
  const labels: Record<ReaderCanvasTool, string> = { select: "Read / select", highlight: "Highlight", draw: "Draw", type: "Type", voice: "Voice", photo: "Photograph", video: "Video clip" };
  return <div className="reader-toolbar" role="toolbar" aria-label="Article canvas annotation tools">
    {(Object.keys(labels) as ReaderCanvasTool[]).map(mode => <button key={mode} type="button" className={`reader-chip${tool === mode ? "" : " reader-chip--ghost"}`} aria-pressed={tool === mode} disabled={busy} onClick={() => onToolChange(mode)}>{labels[mode]}</button>)}
  </div>;
}
