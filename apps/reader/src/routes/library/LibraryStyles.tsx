import type { ReactNode } from "react";
import catalog from "./styles/catalog.css?inline";
import downloads from "./styles/downloads.css?inline";

export function LibraryStyles(): ReactNode {
  return <style aria-hidden="true">{catalog + "\n" + downloads}</style>;
}
