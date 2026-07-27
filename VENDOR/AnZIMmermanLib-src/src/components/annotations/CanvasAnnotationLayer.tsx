// Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
// All rights reserved.
// Unauthorized use without prior written consent is strictly prohibited.

// EnZIM - Canvas Annotation Layer Component

import { useMemo, useCallback } from 'react';
import { MapPin } from 'lucide-react';
import { useAnnotationStore } from '../../stores/annotationStore';
import type { Annotation, AnnotationType } from '../../types/annotation';

interface CanvasAnnotationLayerProps {
  articleUrl: string;
  archiveId: string;
  contentRef: React.RefObject<HTMLElement>;
  isPlacingMode: boolean;
  placementType: AnnotationType | null;
  onPlacement: (position: { x: number; y: number }) => void;
  onPinClick?: (annotation: Annotation) => void;
}

export function CanvasAnnotationLayer({
  articleUrl,
  archiveId,
  contentRef,
  isPlacingMode,
  placementType,
  onPlacement,
  onPinClick,
}: CanvasAnnotationLayerProps) {
  const { annotations, removeAnnotation } = useAnnotationStore();

  // Filter annotations for this article
  const pinnedAnnotations = useMemo(() => {
    return annotations.filter(
      (a) => a.articleUrl === articleUrl && a.archiveId === archiveId && a.position
    );
  }, [annotations, articleUrl, archiveId]);

  // Calculate display position from percentage
  const calculateDisplayPosition = useCallback((annotation: Annotation) => {
    const container = contentRef.current;
    if (!container || !annotation.position) return { x: 0, y: 0 };

    const rect = container.getBoundingClientRect();
    return {
      x: (annotation.position.x / 100) * rect.width,
      y: (annotation.position.y / 100) * rect.height,
    };
  }, [contentRef]);

  // Handle canvas click for placement
  const handleCanvasClick = useCallback((e: React.MouseEvent) => {
    if (!isPlacingMode) return;

    const container = contentRef.current;
    if (!container) return;

    // Don't place if clicking on a pin
    if ((e.target as HTMLElement).closest('.annotation-pin')) return;

    const rect = container.getBoundingClientRect();
    const x = ((e.clientX - rect.left) / rect.width) * 100;
    const y = ((e.clientY - rect.top) / rect.height) * 100;

    onPlacement({ x, y });
  }, [isPlacingMode, contentRef, onPlacement]);

  // Handle pin click
  const handlePinClick = useCallback((e: React.MouseEvent, annotation: Annotation) => {
    e.stopPropagation();
    onPinClick?.(annotation);
  }, [onPinClick]);

  // Handle pin delete
  const handlePinDelete = useCallback((e: React.MouseEvent, id: string) => {
    e.stopPropagation();
    removeAnnotation(id);
  }, [removeAnnotation]);

  // Get color class for pin
  const getPinColorClass = (annotation: Annotation): string => {
    switch (annotation.color) {
      case 'yellow': return 'text-yellow-500';
      case 'blue': return 'text-blue-500';
      case 'green': return 'text-green-500';
      case 'pink': return 'text-pink-500';
      case 'orange': return 'text-orange-500';
      default: return 'text-[var(--accent)]';
    }
  };

  return (
    <div
      className={`absolute inset-0 ${isPlacingMode ? 'cursor-crosshair' : 'pointer-events-none'}`}
      onClick={handleCanvasClick}
    >
      {/* Render pinned annotations */}
      {pinnedAnnotations.map((annotation) => {
        const pos = calculateDisplayPosition(annotation);
        const colorClass = getPinColorClass(annotation);

        return (
          <div
            key={annotation.id}
            className="annotation-pin absolute pointer-events-auto group"
            style={{
              left: `${pos.x}px`,
              top: `${pos.y}px`,
              transform: 'translate(-50%, -100%)',
            }}
          >
            <button
              onClick={(e) => handlePinClick(e, annotation)}
              className="relative"
              aria-label={`Annotation: ${annotation.type}`}
            >
              <MapPin className={`w-6 h-6 ${colorClass} drop-shadow-md`} />
              
              {/* Tooltip on hover */}
              <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 hidden group-hover:block z-50">
                <div className="bg-[var(--bg-surface)] border border-[var(--border)] rounded-lg p-2 shadow-lg whitespace-nowrap">
                  <p className="text-xs text-[var(--text-primary)] capitalize">
                    {annotation.type} annotation
                  </p>
                  {annotation.type === 'text' && (
                    <p className="text-xs text-secondary truncate max-w-[150px]">
                      {annotation.content.substring(0, 50)}...
                    </p>
                  )}
                </div>
              </div>
            </button>

            {/* Delete button on hover */}
            <button
              onClick={(e) => handlePinDelete(e, annotation.id)}
              className="absolute -top-2 -right-2 w-5 h-5 bg-red-500 text-white rounded-full flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity text-xs"
              aria-label="Delete annotation"
            >
              ×
            </button>
          </div>
        );
      })}

      {/* Placement mode indicator */}
      {isPlacingMode && placementType && (
        <div className="absolute top-4 left-1/2 -translate-x-1/2 bg-[var(--accent)] text-white px-4 py-2 rounded-full text-sm shadow-lg pointer-events-none">
          Click anywhere to place {placementType} annotation
        </div>
      )}
    </div>
  );
}
