// Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
// All rights reserved.
// Unauthorized use without prior written consent is strictly prohibited.

// EnZIM - Text Highlighter Component

import { useState, useEffect, useCallback, useRef } from 'react';
import { X } from 'lucide-react';
import type { AnnotationPosition, AnnotationColor } from '../../types/annotation';

interface TextHighlighterProps {
  onHighlight: (text: string, position: AnnotationPosition, color: AnnotationColor) => void;
  containerRef: React.RefObject<HTMLElement>;
}

const COLORS: { name: AnnotationColor; class: string; bgClass: string }[] = [
  { name: 'yellow', class: 'bg-yellow-400', bgClass: 'bg-yellow-200' },
  { name: 'blue', class: 'bg-blue-400', bgClass: 'bg-blue-200' },
  { name: 'green', class: 'bg-green-400', bgClass: 'bg-green-200' },
  { name: 'pink', class: 'bg-pink-400', bgClass: 'bg-pink-200' },
];

interface SelectionInfo {
  text: string;
  rect: DOMRect;
  selector: string;
  offset: number;
  containerElement: Element;
}

export function TextHighlighter({ onHighlight, containerRef }: TextHighlighterProps) {
  const [showToolbar, setShowToolbar] = useState(false);
  const [toolbarPosition, setToolbarPosition] = useState({ x: 0, y: 0 });
  const [selectionInfo, setSelectionInfo] = useState<SelectionInfo | null>(null);
  const toolbarRef = useRef<HTMLDivElement>(null);

  const generateSelector = useCallback((element: Element): string => {
    const path: string[] = [];
    let current: Element | null = element;
    const container = containerRef.current;

    while (current && current !== container && current !== document.body) {
      let selector = current.tagName.toLowerCase();
      
      if (current.id) {
        selector += `#${current.id}`;
        path.unshift(selector);
        break;
      }
      
      if (current.className && typeof current.className === 'string') {
        const classes = current.className.split(' ').filter(c => c && !c.startsWith('_'));
        if (classes.length > 0) {
          selector += `.${classes.join('.')}`;
        }
      }
      
      // Add nth-child for siblings
      const siblings = Array.from(current.parentElement?.children || []);
      const index = siblings.indexOf(current) + 1;
      if (siblings.length > 1) {
        selector += `:nth-child(${index})`;
      }
      
      path.unshift(selector);
      current = current.parentElement;
    }

    return path.join(' > ');
  }, [containerRef]);

  const getSelectionInfo = useCallback((): SelectionInfo | null => {
    const selection = window.getSelection();
    if (!selection || selection.rangeCount === 0) return null;

    const range = selection.getRangeAt(0);
    const text = selection.toString().trim();
    
    if (!text || text.length < 2) return null;

    const container = containerRef.current;
    if (!container) return null;

    // Check if selection is within container
    const commonAncestor = range.commonAncestorContainer;
    const element = commonAncestor.nodeType === Node.ELEMENT_NODE 
      ? commonAncestor as Element 
      : commonAncestor.parentElement;
    
    if (!element || !container.contains(element)) return null;

    const rect = range.getBoundingClientRect();
    const selector = generateSelector(element);
    
    // Calculate offset from container start
    const preRange = document.createRange();
    preRange.selectNodeContents(container);
    preRange.setEnd(range.startContainer, range.startOffset);
    const offset = preRange.toString().length;

    return {
      text,
      rect,
      selector,
      offset,
      containerElement: element,
    };
  }, [containerRef, generateSelector]);

  const handleSelection = useCallback(() => {
    const info = getSelectionInfo();
    if (info) {
      setSelectionInfo(info);
      
      // Position toolbar above selection
      const toolbarHeight = 40;
      const x = info.rect.left + info.rect.width / 2;
      const y = info.rect.top - toolbarHeight - 8;
      
      setToolbarPosition({ x, y });
      setShowToolbar(true);
    } else {
      setShowToolbar(false);
    }
  }, [getSelectionInfo]);

  const handleHighlight = useCallback((color: AnnotationColor) => {
    if (!selectionInfo) return;

    const position: AnnotationPosition = {
      x: 0,
      y: 0,
      selector: selectionInfo.selector,
      offset: selectionInfo.offset,
      rect: {
        top: selectionInfo.rect.top,
        left: selectionInfo.rect.left,
        width: selectionInfo.rect.width,
        height: selectionInfo.rect.height,
      },
    };

    onHighlight(selectionInfo.text, position, color);
    
    // Clear selection
    window.getSelection()?.removeAllRanges();
    setShowToolbar(false);
    setSelectionInfo(null);
  }, [selectionInfo, onHighlight]);

  const handleClose = useCallback(() => {
    window.getSelection()?.removeAllRanges();
    setShowToolbar(false);
    setSelectionInfo(null);
  }, []);

  // Listen for selection events
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const handleMouseUp = () => {
      // Small delay to allow selection to complete
      setTimeout(handleSelection, 10);
    };

    document.addEventListener('mouseup', handleMouseUp);
    return () => {
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [containerRef, handleSelection]);

  // Close toolbar on scroll
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const handleScroll = () => {
      if (showToolbar) {
        setShowToolbar(false);
      }
    };

    container.addEventListener('scroll', handleScroll);
    return () => {
      container.removeEventListener('scroll', handleScroll);
    };
  }, [containerRef, showToolbar]);

  if (!showToolbar) return null;

  return (
    <div
      ref={toolbarRef}
      className="fixed z-50 flex items-center gap-1 p-1.5 bg-[var(--bg-surface)] border border-[var(--border)] rounded-lg shadow-lg animate-in fade-in zoom-in-95 duration-100"
      style={{
        left: `${toolbarPosition.x}px`,
        top: `${toolbarPosition.y}px`,
        transform: 'translateX(-50%)',
      }}
    >
      <div className="flex items-center gap-1">
        {COLORS.map((c) => (
          <button
            key={c.name}
            onClick={() => handleHighlight(c.name)}
            className={`w-6 h-6 rounded-full ${c.class} hover:scale-110 transition-transform`}
            aria-label={`Highlight with ${c.name}`}
            title={c.name}
          />
        ))}
      </div>
      
      <div className="w-px h-5 bg-[var(--border)] mx-1" />
      
      <button
        onClick={handleClose}
        className="p-1 hover:bg-[var(--border)] rounded transition-colors"
        aria-label="Close"
      >
        <X className="w-4 h-4 text-secondary" />
      </button>

      {/* Arrow pointer */}
      <div 
        className="absolute left-1/2 -bottom-2 -translate-x-1/2 w-0 h-0 border-l-8 border-r-8 border-t-8 border-transparent border-t-[var(--bg-surface)]"
        style={{
          filter: 'drop-shadow(0 1px 0 var(--border))',
        }}
      />
    </div>
  );
}
