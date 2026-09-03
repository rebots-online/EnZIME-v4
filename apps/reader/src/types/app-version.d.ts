/* §Mandatory App Chrome + §15.1 build plumbing.
 *
 * `__APP_VERSION__` is injected at build time by Vite's `define` (see
 * `vite.config.ts`) from the canonical `version.txt` stamped by
 * `scripts/update-version.sh`. Declared once here as an ambient global so
 * every consumer (Splash, AboutDialog, AppShell, …) reads the same typed
 * binding instead of re-declaring `declare global` per file (which would
 * trip TS2451 "Cannot redeclare block-scoped variable").
 */
declare const __APP_VERSION__: string;
