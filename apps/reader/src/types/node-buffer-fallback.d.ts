// Ambient declaration for the Node-only fallback in `services/annotationService.ts`.
//
// The browser path calls `btoa`; the `Buffer` branch exists for non-browser
// (Node/test) environments and is never reached inside the Tauri webview. This
// types that fallback so the file compiles without a full @types/node dev
// dependency. Remove this file if/when @types/node is added to the workspace.
declare const Buffer: {
  from(data: string, encoding: string): {
    toString(encoding: string): string;
  };
};
