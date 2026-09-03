// OpenPackCard — Home entry that opens an arbitrary .zim file (§15.1 / §24).
//
// Flow: pick a .zim path via the file dialog → readerService.readerOpen(path)
// → navigate to /reader/:packId/:mainPageKey.
//
// DIALOG PLUGIN STATUS (note for the architect):
//   Neither `@tauri-apps/plugin-dialog` nor a legacy `@tauri-apps/api/dialog`
//   module is present in apps/reader/package.json. Per the task's gated
//   approach, the dialog is loaded via a non-literal dynamic import (a
//   `string`-typed specifier + `/* @vite-ignore */`) so that:
//     - `tsc --noEmit` does not resolve (and reject) the missing module, and
//     - Vite does not try to bundle it at build time.
//   The affordance is also gated on `window.__TAURI_INTERNALS__` being present:
//   in the pure web dev path it renders as a disabled-but-explained control,
//   and in the Tauri app it attempts the dynamic import. If the plugin is not
//   registered on the Rust/JS side at runtime, the caught error surfaces a
//   recoverable StatusBanner. Installing `@tauri-apps/plugin-dialog` and
//   registering it in tauri.conf.json is the post-M1 hardening step.

import { useState, type ReactNode } from "react";
import { useNavigate } from "react-router-dom";
import { useReaderStore } from "../../stores/readerStore";
import { StatusBanner } from "./shared";

// Typed as `string` (widened) so neither tsc nor Vite treats this as a literal
// module specifier to resolve. See file header.
const DIALOG_MODULE: string = "@tauri-apps/plugin-dialog";

type DialogModule = {
  open?(opts?: unknown): Promise<string | string[] | null>;
};

type PickStatus = "idle" | "picking" | "opening" | "error";

function tauriAvailable(): boolean {
  return (
    typeof window !== "undefined" &&
    "__TAURI_INTERNALS__" in window
  );
}

async function pickZimFile(): Promise<string | null> {
  const mod = (await import(/* @vite-ignore */ DIALOG_MODULE)) as DialogModule;
  if (typeof mod.open !== "function") {
    throw new Error("File dialog is not available in this build.");
  }
  const chosen = await mod.open({
    multiple: false,
    directory: false,
    filters: [{ name: "ZIM archive", extensions: ["zim"] }],
  });
  if (Array.isArray(chosen)) return chosen[0] ?? null;
  return chosen ?? null;
}

export function OpenPackCard(): ReactNode {
  const navigate = useNavigate();
  const openPack = useReaderStore((s) => s.open);
  const [status, setStatus] = useState<PickStatus>("idle");
  const [message, setMessage] = useState<string>("");
  const [tauri] = useState<boolean>(() => tauriAvailable());

  const disabled = !tauri || status === "picking" || status === "opening";

  async function handleOpen(): Promise<void> {
    setStatus("picking");
    setMessage("");
    try {
      const path = await pickZimFile();
      if (!path) {
        setStatus("idle");
        return;
      }
      setStatus("opening");
      const result = await openPack(path);
      if (!result) {
        // open() swallows the classified error into the store slice; surface it.
        const err = useReaderStore.getState().currentArticle.error;
        setStatus("error");
        setMessage(err?.message ?? "Could not open this ZIM file.");
        return;
      }
      setStatus("idle");
      navigate(`/reader/${result.packId}/${result.mainPageKey}`);
    } catch (raw) {
      const text =
        typeof raw === "string"
          ? raw
          : raw instanceof Error
            ? raw.message
            : "The file dialog or ZIM open call failed.";
      setStatus("error");
      setMessage(text);
    }
  }

  return (
    <section
      className="home-card home-card--open-pack"
      aria-labelledby="home-open-pack-heading"
    >
      <header className="home-card__header">
        <h2 className="home-card__title" id="home-open-pack-heading">
          Open a ZIM file
        </h2>
        <span className="home-card__hint">.zim archive</span>
      </header>

      <p className="home-section-intro">
        Choose any ZIM archive on this device to start reading at its main
        page. Files are opened locally — nothing is uploaded.
      </p>

      <div className="home-continue__actions">
        <button
          type="button"
          className="home-button"
          onClick={() => void handleOpen()}
          disabled={disabled}
          aria-disabled={disabled}
          title={
            tauri
              ? "Pick a .zim file from this device"
              : "Available in the EnZIME desktop / Android app"
          }
        >
          {status === "picking"
            ? "Choosing…"
            : status === "opening"
              ? "Opening…"
              : "Open ZIM file"}
        </button>
        {!tauri ? (
          <span className="home-pill home-pill--warning">
            Web build — dialog unavailable
          </span>
        ) : null}
      </div>

      {status === "error" ? (
        <StatusBanner
          tone="warning"
          icon="!"
          title="Could not open ZIM file"
          detail={message}
        />
      ) : null}
    </section>
  );
}
