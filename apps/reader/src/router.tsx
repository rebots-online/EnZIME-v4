import { Navigate, createBrowserRouter } from "react-router-dom";
import { AppShell } from "./components/shell/AppShell";
import { HomeView } from "./components/home/HomeView";
import { LibraryView } from "./routes/library/LibraryView";
import { ReaderView } from "./routes/reader/ReaderView";

/**
 * M1 registers the Home, Library, and Reader routes inside the AppShell
 * layout. `/reader/:packId/:articleKey` (§15.3) calls the `reader_article`
 * command via readerStore and renders a real ZIM article through the §7.3
 * sandbox. `/library` hosts the DynDon download flow (storage fit + queue)
 * via libraryStore / libraryService. Later milestones add `/setup`, `/ask`,
 * `/notes`, `/memory`, `/settings` (§15.3) as each vertical slice lands; the
 * NavigationRail marks unbuilt destinations as scheduled rather than
 * navigating to nothing.
 */
export const router = createBrowserRouter([
  {
    path: "/",
    element: <AppShell />,
    children: [
      { index: true, element: <HomeView /> },
      { path: "library", element: <LibraryView /> },
      { path: "reader/:packId/:articleKey", element: <ReaderView /> },
      { path: "*", element: <Navigate to="/" replace /> },
    ],
  },
]);
