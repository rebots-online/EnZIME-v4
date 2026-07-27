// Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
// All rights reserved.
// Unauthorized use without prior written consent is strictly prohibited.

// EnZIM - Text Note Editor Component

import { useState, useCallback } from 'react';
import { Save, StickyNote } from 'lucide-react';
import type { AnnotationColor } from '../../types/annotation';

interface TextNoteEditorProps {
  onSave: (text: string, color: AnnotationColor) => void;
  onCancel?: () => void;
  initialText?: string;
  initialColor?: AnnotationColor;
}

const COLORS: { name: AnnotationColor; class: string; label: string }[] = [
  { name: 'yellow', class: 'bg-yellow-400', label: 'Yellow' },
  { name: 'blue', class: 'bg-blue-400', label: 'Blue' },
  { name: 'green', class: 'bg-green-400', label: 'Green' },
  { name: 'pink', class: 'bg-pink-400', label: 'Pink' },
];

export function TextNoteEditor({ 
  onSave, 
  onCancel, 
  initialText = '', 
  initialColor = 'yellow' 
}: TextNoteEditorProps) {
  const [noteText, setNoteText] = useState(initialText);
  const [noteColor, setNoteColor] = useState<AnnotationColor>(initialColor);

  const handleSave = useCallback(() => {
    const trimmed = noteText.trim();
    if (!trimmed) return;
    
    onSave(trimmed, noteColor);
    setNoteText('');
  }, [noteText, noteColor, onSave]);

  const handleCancel = useCallback(() => {
    setNoteText('');
    onCancel?.();
  }, [onCancel]);

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center gap-2 text-secondary">
        <StickyNote className="w-4 h-4" />
        <span className="text-sm">Text Note</span>
      </div>

      <textarea
        value={noteText}
        onChange={(e) => setNoteText(e.target.value)}
        placeholder="Type your note..."
        className="w-full h-32 p-3 bg-[var(--bg-surface)] border border-[var(--border)] rounded-lg resize-none focus:outline-none focus:border-[var(--accent)] transition-colors text-sm"
        autoFocus
      />

      <div className="flex items-center gap-2">
        <span className="text-xs text-secondary">Color:</span>
        <div className="flex gap-2">
          {COLORS.map((c) => (
            <button
              key={c.name}
              onClick={() => setNoteColor(c.name)}
              className={`w-6 h-6 rounded-full ${c.class} transition-all ${
                noteColor === c.name 
                  ? 'ring-2 ring-[var(--accent)] ring-offset-2 ring-offset-[var(--bg-primary)] scale-110' 
                  : 'hover:scale-105'
              }`}
              aria-label={`Select ${c.label} color`}
              title={c.label}
            />
          ))}
        </div>
      </div>

      <div className="flex gap-2">
        <button
          onClick={handleCancel}
          className="flex-1 px-4 py-2 bg-[var(--bg-surface)] hover:bg-[var(--border)] border border-[var(--border)] rounded-lg transition-colors text-sm"
        >
          Cancel
        </button>
        <button
          onClick={handleSave}
          disabled={!noteText.trim()}
          className="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-[var(--accent)] text-white hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg transition-colors text-sm"
        >
          <Save className="w-4 h-4" />
          Save
        </button>
      </div>
    </div>
  );
}
