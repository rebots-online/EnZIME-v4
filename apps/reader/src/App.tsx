import type { JSX } from "react";

declare global {
  const __APP_VERSION__: string;
}

const APP_VERSION: string = typeof __APP_VERSION__ !== "undefined"
  ? __APP_VERSION__
  : "1.0.52684";

type ViewProps = {
  readonly name: string;
};

function ViewHeading({ name }: ViewProps): JSX.Element {
  return (
    <main className="view">
      <h1>{name}</h1>
    </main>
  );
}

export function SetupRoute(): JSX.Element {
  return <ViewHeading name="SetupRoute" />;
}

export function HomeView(): JSX.Element {
  return <ViewHeading name="HomeView" />;
}

export function LibraryView(): JSX.Element {
  return <ViewHeading name="LibraryView" />;
}

export function ReaderView(): JSX.Element {
  return <ViewHeading name="ReaderView" />;
}

export function AskView(): JSX.Element {
  return <ViewHeading name="AskView" />;
}

export function NotesView(): JSX.Element {
  return <ViewHeading name="NotesView" />;
}

export function MemoryView(): JSX.Element {
  return <ViewHeading name="MemoryView" />;
}

export function SettingsView(): JSX.Element {
  return <ViewHeading name="SettingsView" />;
}

export function App(): JSX.Element {
  return (
    <>
      <div id="app-version" aria-label="application version">
        EnZIME Reader v{APP_VERSION}
      </div>
    </>
  );
}

export default App;
