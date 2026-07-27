import { createBrowserRouter } from "react-router-dom";
import App, {
  AskView,
  HomeView,
  LibraryView,
  MemoryView,
  NotesView,
  ReaderView,
  SettingsView,
  SetupRoute,
} from "./App";

export const router = createBrowserRouter([
  { path: "/setup", element: <SetupRoute /> },
  { path: "/", element: <HomeView /> },
  { path: "/library", element: <LibraryView /> },
  { path: "/reader/:packId/:articleKey", element: <ReaderView /> },
  { path: "/ask", element: <AskView /> },
  { path: "/ask/:conversationId", element: <AskView /> },
  { path: "/notes", element: <NotesView /> },
  { path: "/memory", element: <MemoryView /> },
  { path: "/settings", element: <SettingsView /> },
]);

export const AppShell = <App />;
