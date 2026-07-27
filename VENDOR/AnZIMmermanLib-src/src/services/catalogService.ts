// Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
// All rights reserved.
// Unauthorized use without prior written consent is strictly prohibited.

// EnZIM - Offline ZIM Reader & Knowledge Explorer
// Kiwix OPDS Catalog Service — fetches ZIM catalog for in-app browsing/downloading

export interface CatalogEntry {
  id: string;
  title: string;
  description: string;
  language: string;
  tags: string[];
  favicon?: string;
  size: number;
  articleCount: number;
  mediaCount: number;
  url: string;
  name: string;
  flavour: string;
  category: string;
}

export interface CatalogSearchResult {
  entries: CatalogEntry[];
  totalResults: number;
  startIndex: number;
  itemsPerPage: number;
}

const KIWIX_CATALOG_BASE = 'https://library.kiwix.org';
const OPDS_SEARCH = '/catalog/search';

function parseSize(sizeStr: string): number {
  const num = parseFloat(sizeStr);
  if (isNaN(num)) return 0;
  if (sizeStr.includes('G')) return Math.round(num * 1024 * 1024 * 1024);
  if (sizeStr.includes('M')) return Math.round(num * 1024 * 1024);
  if (sizeStr.includes('K')) return Math.round(num * 1024);
  return Math.round(num);
}

function parseXmlText(xml: string, tag: string): string {
  const regex = new RegExp(`<${tag}[^>]*>([\\s\\S]*?)</${tag}>`, 'i');
  const match = xml.match(regex);
  return match ? match[1].trim() : '';
}

function parseXmlAttr(xml: string, tag: string, attr: string): string {
  const tagRegex = new RegExp(`<${tag}[^>]*${attr}="([^"]*)"[^>]*/?>`, 'i');
  const match = xml.match(tagRegex);
  return match ? match[1] : '';
}

function parseEntryFromXml(entryXml: string): CatalogEntry | null {
  const id = parseXmlText(entryXml, 'id');
  const title = parseXmlText(entryXml, 'title');
  const description = parseXmlText(entryXml, 'summary') || parseXmlText(entryXml, 'subtitle');
  const language = parseXmlText(entryXml, 'language');
  const name = parseXmlText(entryXml, 'name') || parseXmlAttr(entryXml, 'link[^>]*rel="http://opds-spec.org/acquisition/open-access"', 'href');
  const articleCountStr = parseXmlText(entryXml, 'articleCount');
  const mediaCountStr = parseXmlText(entryXml, 'mediaCount');
  const sizeStr = parseXmlText(entryXml, 'size');
  const category = parseXmlText(entryXml, 'category') || '';
  const flavour = parseXmlText(entryXml, 'flavour') || '';
  const tags = parseXmlText(entryXml, 'tags')?.split(';').filter(Boolean) ?? [];

  // Find the download link
  const linkRegex = /href="([^"]*\.zim)"/i;
  const linkMatch = entryXml.match(linkRegex);
  const downloadPath = linkMatch ? linkMatch[1] : '';

  // Find favicon
  const iconRegex = /href="([^"]*)"[^>]*rel="http:\/\/opds-spec\.org\/image\/thumbnail"/i;
  const iconMatch = entryXml.match(iconRegex);
  const favicon = iconMatch ? `${KIWIX_CATALOG_BASE}${iconMatch[1]}` : undefined;

  if (!id || !title) return null;

  return {
    id,
    title,
    description,
    language: language || 'en',
    tags,
    favicon,
    size: parseSize(sizeStr) || parseInt(sizeStr) || 0,
    articleCount: parseInt(articleCountStr) || 0,
    mediaCount: parseInt(mediaCountStr) || 0,
    url: downloadPath.startsWith('http') ? downloadPath : `${KIWIX_CATALOG_BASE}${downloadPath}`,
    name: name || id,
    flavour,
    category,
  };
}

export class CatalogService {
  private baseUrl: string;

  constructor(baseUrl: string = KIWIX_CATALOG_BASE) {
    this.baseUrl = baseUrl;
  }

  async search(query: string = '', options: {
    language?: string;
    category?: string;
    count?: number;
    start?: number;
  } = {}): Promise<CatalogSearchResult> {
    const params = new URLSearchParams();
    if (query) params.set('q', query);
    if (options.language) params.set('lang', options.language);
    if (options.category) params.set('tag', `_category:${options.category}`);
    params.set('count', String(options.count ?? 24));
    params.set('start', String(options.start ?? 0));

    const url = `${this.baseUrl}${OPDS_SEARCH}?${params.toString()}`;

    try {
      const response = await this.fetchCatalog(url);
      return this.parseOpdsFeed(response, options.start ?? 0, options.count ?? 24);
    } catch (error) {
      console.error('Catalog search failed:', error);
      return { entries: [], totalResults: 0, startIndex: 0, itemsPerPage: 24 };
    }
  }

  async getCategories(): Promise<string[]> {
    return [
      'wikipedia', 'wiktionary', 'wikibooks', 'wikiversity',
      'wikiquote', 'wikisource', 'wikivoyage', 'wikinews',
      'ted', 'gutenberg', 'stackexchange', 'freecodecamp',
      'other',
    ];
  }

  async getLanguages(): Promise<{ code: string; name: string }[]> {
    return [
      { code: 'eng', name: 'English' },
      { code: 'fra', name: 'French' },
      { code: 'deu', name: 'German' },
      { code: 'spa', name: 'Spanish' },
      { code: 'por', name: 'Portuguese' },
      { code: 'ita', name: 'Italian' },
      { code: 'rus', name: 'Russian' },
      { code: 'zho', name: 'Chinese' },
      { code: 'jpn', name: 'Japanese' },
      { code: 'kor', name: 'Korean' },
      { code: 'ara', name: 'Arabic' },
      { code: 'hin', name: 'Hindi' },
      { code: 'mul', name: 'Multilingual' },
    ];
  }

  private async fetchCatalog(url: string): Promise<string> {
    // Use Tauri HTTP plugin if available, otherwise fetch
    try {
      const { fetch: tauriFetch } = await import('@tauri-apps/plugin-http');
      const response = await tauriFetch(url, {
        method: 'GET',
        headers: { 'Accept': 'application/atom+xml' },
      });
      return await response.text();
    } catch {
      // Fallback to regular fetch (browser/dev mode)
      const response = await fetch(url, {
        headers: { 'Accept': 'application/atom+xml' },
      });
      return await response.text();
    }
  }

  private parseOpdsFeed(xml: string, startIndex: number, itemsPerPage: number): CatalogSearchResult {
    const totalStr = xml.match(/<totalResults>(\d+)<\/totalResults>/i);
    const totalResults = totalStr ? parseInt(totalStr[1]) : 0;

    const entries: CatalogEntry[] = [];
    const entryRegex = /<entry>([\s\S]*?)<\/entry>/gi;
    let match;
    while ((match = entryRegex.exec(xml)) !== null) {
      const entry = parseEntryFromXml(match[1]);
      if (entry) entries.push(entry);
    }

    return {
      entries,
      totalResults,
      startIndex,
      itemsPerPage,
    };
  }
}

export const catalogService = new CatalogService();
