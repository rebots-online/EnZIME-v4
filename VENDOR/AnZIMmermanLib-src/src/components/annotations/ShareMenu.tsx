// Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
// All rights reserved.
// Unauthorized use without prior written consent is strictly prohibited.

// EnZIM - Share Menu: multi-format annotation sharing for deep-linking

import { useState, useRef, useEffect, useCallback } from 'react';
import {
  Share2, Copy, FileText, Code, Globe, Link2, Check,
} from 'lucide-react';
import type { Annotation } from '../../types/annotation';
import {
  buildAnnotationReference,
  toPlainText,
  toMarkdown,
  toJSON,
  toHTML,
  copyToClipboard,
  generateDeepLink,
} from '../../utils/deepLink';

interface ShareMenuProps {
  annotation: Annotation;
  archiveName: string;
  articleTitle: string;
}

type CopyFormat = 'text' | 'markdown' | 'json' | 'html' | 'deeplink';

const FORMAT_OPTIONS: { id: CopyFormat; label: string; icon: typeof Copy; description: string }[] = [
  { id: 'text', label: 'Plain Text', icon: FileText, description: 'Human-readable one-liner' },
  { id: 'markdown', label: 'Markdown', icon: Copy, description: 'Blockquote with attribution' },
  { id: 'json', label: 'JSON (for AI)', icon: Code, description: 'Structured payload for coding assistants' },
  { id: 'html', label: 'HTML', icon: Globe, description: 'Styled blockquote' },
  { id: 'deeplink', label: 'Deep Link', icon: Link2, description: 'enzim:// URI' },
];

export function ShareMenu({ annotation, archiveName, articleTitle }: ShareMenuProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [copiedFormat, setCopiedFormat] = useState<CopyFormat | null>(null);
  const menuRef = useRef<HTMLDivElement>(null);

  // Close on outside click
  useEffect(() => {
    if (!isOpen) return;
    const handler = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setIsOpen(false);
      }
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, [isOpen]);

  const handleCopy = useCallback(async (format: CopyFormat) => {
    const ref = buildAnnotationReference(annotation, archiveName, articleTitle);
    let text: string;

    switch (format) {
      case 'text':
        text = toPlainText(ref);
        break;
      case 'markdown':
        text = toMarkdown(ref);
        break;
      case 'json':
        text = toJSON(ref);
        break;
      case 'html':
        text = toHTML(ref);
        break;
      case 'deeplink':
        text = generateDeepLink(archiveName, annotation.articleUrl, annotation.id);
        break;
    }

    const success = await copyToClipboard(text);
    if (success) {
      setCopiedFormat(format);
      setTimeout(() => setCopiedFormat(null), 2000);
    }
  }, [annotation, archiveName, articleTitle]);

  return (
    <div className="relative" ref={menuRef}>
      <button
        onClick={(e) => { e.stopPropagation(); setIsOpen(!isOpen); }}
        className="p-1.5 rounded hover:bg-black/5 dark:hover:bg-white/5 transition-colors"
        title="Share annotation"
      >
        <Share2 className="w-3.5 h-3.5 text-secondary" />
      </button>

      {isOpen && (
        <div className="absolute right-0 top-full mt-1 w-64 surface rounded-lg shadow-xl border border-[var(--border)] py-1 z-50">
          <div className="px-3 py-1.5 text-xs font-medium text-secondary uppercase tracking-wider">
            Copy as...
          </div>
          {FORMAT_OPTIONS.map((opt) => (
            <button
              key={opt.id}
              onClick={(e) => { e.stopPropagation(); handleCopy(opt.id); }}
              className="w-full flex items-center gap-3 px-3 py-2 text-sm hover:bg-[var(--accent)] hover:text-white transition-colors text-left"
            >
              {copiedFormat === opt.id ? (
                <Check className="w-4 h-4 text-green-500 flex-shrink-0" />
              ) : (
                <opt.icon className="w-4 h-4 flex-shrink-0" />
              )}
              <div className="flex-1 min-w-0">
                <p className="font-medium">{opt.label}</p>
                <p className="text-xs opacity-70">{opt.description}</p>
              </div>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
