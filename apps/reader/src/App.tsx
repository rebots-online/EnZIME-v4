import type { JSX } from "react";
import { RouterProvider } from "react-router-dom";
import { router } from "./router";
import {
  ErrorBoundary,
  EventBridge,
  IpcProvider,
  ThemeProvider,
  ToastProvider,
} from "./components/shell/providers";

/**
 * Root composition (§15.1):
 *   ErrorBoundary ▸ ThemeProvider ▸ IpcProvider ▸ EventBridge ▸ ToastProvider ▸ AppRouter
 * The AppShell — with its Aurora Glass chrome — is the layout element of the
 * router, declared in `router.tsx`.
 */
export default function App(): JSX.Element {
  return (
    <ErrorBoundary>
      <ThemeProvider>
        <IpcProvider>
          <EventBridge>
            <ToastProvider>
              <RouterProvider router={router} />
            </ToastProvider>
          </EventBridge>
        </IpcProvider>
      </ThemeProvider>
    </ErrorBoundary>
  );
}
