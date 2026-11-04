import { useLayoutEffect, useRef, useState, type FormEvent, type ReactNode } from "react";
import { readerService, type ReaderTitleSearchPage, type ServiceError } from "../../services/readerService";

const PAGE_SIZE = 25;
const MAX_OFFSET = 10_000;

/** Mounted with the route pack as its key: changing packs discards this search. */
export function ReaderTitleSearch({ packId, onNavigate }: {
  readonly packId: string;
  readonly onNavigate: (path: string) => void;
}): ReactNode {
  const [query, setQuery] = useState("");
  const [page, setPage] = useState<ReaderTitleSearchPage>();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string>();
  const generation = useRef(0);
  const mounted = useRef(false);

  useLayoutEffect(() => {
    mounted.current = true;
    return () => { mounted.current = false; generation.current += 1; };
  }, []);

  async function search(text: string, offset: number) {
    const request = ++generation.current;
    setLoading(true);
    setError(undefined);
    setPage(undefined);
    try {
      const result = await readerService.readerSearchTitles(packId, text, offset, PAGE_SIZE);
      if (mounted.current && generation.current === request) setPage(result);
    } catch (cause) {
      if (mounted.current && generation.current === request) {
        setError((cause as Partial<ServiceError> | null)?.message ?? "Could not search titles. Try again.");
      }
    } finally {
      if (mounted.current && generation.current === request) setLoading(false);
    }
  }

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    void search(query.trim(), 0);
  }

  let status = "Search article titles in this pack.";
  if (loading) status = "Searching titles…";
  else if (page) {
    if (!page.query) status = "Enter a title or part of a title to search.";
    else if (!page.items.length) status = page.offset > 0
      ? "No titles on this page. Return to the previous page or refine your search."
      : `No titles match “${page.query}”.`;
    else status = `Showing matches ${page.offset + 1}–${page.offset + page.items.length} for “${page.query}”.`;
  }

  const nextOffset = page ? page.offset + page.limit : 0;
  return <section className="reader-title-search" aria-label="Search this pack">
    <form onSubmit={submit} className="reader-title-search__form">
      <label htmlFor="reader-title-query">Search titles in this pack</label>
      <div className="reader-title-search__input-row">
        <input id="reader-title-query" type="search" value={query} onChange={event => {
          // Editing makes any outstanding result obsolete immediately.
          generation.current += 1;
          setQuery(event.target.value); setPage(undefined); setError(undefined); setLoading(false);
        }} />
        <button type="submit" className="reader-chip" disabled={!packId}>Search</button>
      </div>
    </form>
    <p role="status" aria-live="polite">{error ? "Title search failed." : status}</p>
    {error ? <p role="alert">{error}</p> : null}
    {page && page.items.length > 0 ?
      <ul className="reader-title-search__results">
        {page.items.map(item => {
          const path = `/reader/${encodeURIComponent(packId)}/${encodeURIComponent(item.articleKey)}`;
          return <li key={item.articleKey}><a href={path} onClick={event => {
            event.preventDefault(); onNavigate(path);
          }}>{item.title}</a></li>;
        })}
      </ul>
    : null}
    {page && (page.items.length > 0 || page.offset > 0) ?
      <nav className="reader-title-search__pages" aria-label="Title search pages">
        <button type="button" className="reader-chip reader-chip--ghost" disabled={loading || page.offset === 0}
          onClick={() => void search(page.query, Math.max(0, page.offset - page.limit))}>Previous</button>
        {page.hasMore && nextOffset > MAX_OFFSET
          ? <p>Refine your search to see more matches.</p>
          : <button type="button" className="reader-chip reader-chip--ghost" disabled={loading || !page.hasMore}
            onClick={() => void search(page.query, nextOffset)}>Next</button>}
      </nav>
    : null}
  </section>;
}
