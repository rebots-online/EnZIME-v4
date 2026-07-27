// Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
// All rights reserved.
// Unauthorized use without prior written consent is strictly prohibited.

// EnZIM - Deep Link Utilities
// Generates stable URIs and multi-format clipboard payloads for annotations

import type { Annotation, AnnotationReference } from '../types/annotation';

export function generateDeepLink(
  archiveFingerprint: string,
  articleUrl: string,
  annotationId: string,
): string {
  return `enzim://${encodeURIComponent(archiveFingerprint)}/${articleUrl}#annotation/${annotationId}`;
}

export function parseDeepLink(uri: string): {
  archiveFingerprint: string;
  articleUrl: string;
  annotationId: string;
} | null {
  const match = uri.match(/^enzim:\/\/([^/]+)\/(.+)#annotation\/(.+)$/);
  if (!match) return null;
  return {
    archiveFingerprint: decodeURIComponent(match[1]),
    articleUrl: match[2],
    annotationId: match[3],
  };
}

export function buildAnnotationReference(
  annotation: Annotation,
  archiveName: string,
  articleTitle: string,
  surroundingText?: string,
): AnnotationReference {
  const deepLink = generateDeepLink(archiveName, annotation.articleUrl, annotation.id);
  return {
    source: 'EnZIM',
    archive: archiveName,
    article: { url: annotation.articleUrl, title: articleTitle },
    annotation: {
      id: annotation.id,
      type: annotation.type,
      content: annotation.content,
      tags: annotation.tags,
      created: annotation.createdAt,
    },
    context: surroundingText ? { surroundingText } : undefined,
    deepLink,
  };
}

export function toPlainText(ref: AnnotationReference): string {
  const contentPreview = ref.annotation.content.length > 120
    ? ref.annotation.content.slice(0, 120) + '...'
    : ref.annotation.content;
  return `[EnZIM] ${ref.article.title} — "${contentPreview}" (${ref.archive})`;
}

export function toMarkdown(ref: AnnotationReference): string {
  const contentPreview = ref.annotation.content.length > 200
    ? ref.annotation.content.slice(0, 200) + '...'
    : ref.annotation.content;
  return `> ${contentPreview}\n\n— *[${ref.article.title}](${ref.deepLink})*, ${ref.archive}`;
}

export function toJSON(ref: AnnotationReference): string {
  return JSON.stringify(ref, null, 2);
}

export function toHTML(ref: AnnotationReference): string {
  const contentPreview = ref.annotation.content.length > 200
    ? ref.annotation.content.slice(0, 200) + '...'
    : ref.annotation.content;
  return `<blockquote style="border-left:3px solid #0ea5e9;padding:8px 12px;margin:8px 0;background:#f0f9ff;">
  <p style="margin:0 0 4px">${escapeHtml(contentPreview)}</p>
  <footer style="font-size:0.85em;color:#6b7280;">
    — <a href="${escapeHtml(ref.deepLink)}">${escapeHtml(ref.article.title)}</a>, ${escapeHtml(ref.archive)}
  </footer>
</blockquote>`;
}

function escapeHtml(str: string): string {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

export async function copyToClipboard(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    // Fallback for older browsers
    const textarea = document.createElement('textarea');
    textarea.value = text;
    textarea.style.position = 'fixed';
    textarea.style.opacity = '0';
    document.body.appendChild(textarea);
    textarea.select();
    try {
      document.execCommand('copy');
      return true;
    } catch {
      return false;
    } finally {
      document.body.removeChild(textarea);
    }
  }
}
