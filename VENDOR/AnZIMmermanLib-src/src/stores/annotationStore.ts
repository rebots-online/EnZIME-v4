// Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
// All rights reserved.
// Unauthorized use without prior written consent is strictly prohibited.

// EnZIM - Annotation Store

import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { Annotation, AnnotationType, AnnotatedArticleSummary } from '../types/annotation';

interface AnnotationState {
  annotations: Annotation[];
  isLoading: boolean;
  error: string | null;

  // Actions
  addAnnotation: (annotation: Omit<Annotation, 'id' | 'createdAt' | 'modifiedAt'>) => string;
  updateAnnotation: (id: string, updates: Partial<Omit<Annotation, 'id'>>) => void;
  removeAnnotation: (id: string) => void;
  getAnnotationsForArticle: (articleUrl: string, archiveId: string) => Annotation[];
  getAnnotationById: (id: string) => Annotation | undefined;
  getAllAnnotations: () => Annotation[];
  getAnnotationsByType: (type: AnnotationType) => Annotation[];
  getAnnotationsForArchive: (archiveId: string) => Annotation[];
  getAnnotationCountForArchive: (archiveId: string) => number;
  getAnnotatedArticlesForArchive: (archiveId: string) => AnnotatedArticleSummary[];
  importAnnotations: (annotations: Annotation[]) => void;
  exportAnnotationsForArticle: (articleUrl: string, archiveId: string) => Annotation[];
  searchAnnotations: (query: string) => Annotation[];
  clearError: () => void;
}

function generateId(): string {
  return Date.now().toString(36) + Math.random().toString(36).substr(2, 9);
}

export const useAnnotationStore = create<AnnotationState>()(
  persist(
    (set, get) => ({
      annotations: [],
      isLoading: false,
      error: null,

      addAnnotation: (annotation) => {
        const id = generateId();
        const now = new Date().toISOString();
        const newAnnotation: Annotation = {
          ...annotation,
          id,
          createdAt: now,
          modifiedAt: now,
        };
        set((state) => ({
          annotations: [...state.annotations, newAnnotation],
        }));
        return id;
      },

      updateAnnotation: (id, updates) => {
        const now = new Date().toISOString();
        set((state) => ({
          annotations: state.annotations.map((a) =>
            a.id === id ? { ...a, ...updates, modifiedAt: now } : a
          ),
        }));
      },

      removeAnnotation: (id) => {
        set((state) => ({
          annotations: state.annotations.filter((a) => a.id !== id),
        }));
      },

      getAnnotationsForArticle: (articleUrl, archiveId) => {
        return get().annotations.filter(
          (a) => a.articleUrl === articleUrl && a.archiveId === archiveId
        );
      },

      getAnnotationById: (id) => {
        return get().annotations.find((a) => a.id === id);
      },

      getAllAnnotations: () => {
        return get().annotations;
      },

      getAnnotationsByType: (type) => {
        return get().annotations.filter((a) => a.type === type);
      },

      getAnnotationsForArchive: (archiveId) => {
        return get().annotations.filter((a) => a.archiveId === archiveId);
      },

      getAnnotationCountForArchive: (archiveId) => {
        return get().annotations.filter((a) => a.archiveId === archiveId).length;
      },

      getAnnotatedArticlesForArchive: (archiveId) => {
        const archiveAnnotations = get().annotations.filter((a) => a.archiveId === archiveId);
        const grouped = new Map<string, Annotation[]>();
        for (const a of archiveAnnotations) {
          const existing = grouped.get(a.articleUrl) || [];
          existing.push(a);
          grouped.set(a.articleUrl, existing);
        }
        const summaries: AnnotatedArticleSummary[] = [];
        for (const [articleUrl, annotations] of grouped) {
          const types = [...new Set(annotations.map((a) => a.type))];
          const latest = annotations.reduce((max, a) => 
            a.modifiedAt > max ? a.modifiedAt : max, annotations[0].modifiedAt
          );
          summaries.push({
            articleUrl,
            count: annotations.length,
            types,
            latestTimestamp: latest,
          });
        }
        return summaries.sort((a, b) => b.latestTimestamp.localeCompare(a.latestTimestamp));
      },

      importAnnotations: (newAnnotations) => {
        set((state) => {
          const existingIds = new Set(state.annotations.map((a) => a.id));
          const uniqueNewAnnotations = newAnnotations.filter(
            (a) => !existingIds.has(a.id)
          );
          return {
            annotations: [...state.annotations, ...uniqueNewAnnotations],
          };
        });
      },

      exportAnnotationsForArticle: (articleUrl, archiveId) => {
        return get().annotations.filter(
          (a) => a.articleUrl === articleUrl && a.archiveId === archiveId
        );
      },

      searchAnnotations: (query) => {
        const lowerQuery = query.toLowerCase();
        return get().annotations.filter((a) => {
          if (a.content.toLowerCase().includes(lowerQuery)) return true;
          if (a.articleUrl.toLowerCase().includes(lowerQuery)) return true;
          if (a.transcription?.toLowerCase().includes(lowerQuery)) return true;
          if (a.tags?.some((t) => t.toLowerCase().includes(lowerQuery))) return true;
          if (a.createdAt.includes(query)) return true;
          if (a.creator?.toLowerCase().includes(lowerQuery)) return true;
          return false;
        });
      },

      clearError: () => {
        set({ error: null });
      },
    }),
    {
      name: 'enzim-annotation-store',
      partialize: (state) => ({
        annotations: state.annotations,
      }),
    }
  )
);
