// Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
// All rights reserved.
// Unauthorized use without prior written consent is strictly prohibited.

// EnZIM - Annotation Types

export type AnnotationType = 'text' | 'voice' | 'ink' | 'photo' | 'video' | 'highlight' | 'screen-clip';

export type AnnotationColor = 'yellow' | 'blue' | 'green' | 'pink' | 'orange';

export interface AnnotationPosition {
  x: number;
  y: number;
  selector?: string;
  offset?: number;
  rect?: {
    top: number;
    left: number;
    width: number;
    height: number;
  };
}

export interface Annotation {
  id: string;
  type: AnnotationType;
  articleUrl: string;
  archiveId: string;
  content: string;
  position?: AnnotationPosition;
  color?: AnnotationColor;
  tags?: string[];
  transcription?: string;
  thumbnail?: string;
  duration?: number;
  createdAt: string;
  modifiedAt: string;
  creator?: string;
  deviceId?: string;
}

export interface AnnotationReference {
  source: 'EnZIM';
  archive: string;
  article: { url: string; title: string };
  annotation: {
    id: string;
    type: AnnotationType;
    content: string;
    tags?: string[];
    created: string;
  };
  context?: { surroundingText?: string };
  deepLink: string;
}

export interface AnnotationManifest {
  version: '1.0';
  zimFingerprint: string;
  zimSource?: string;
  articleUrl: string;
  articleTitle?: string;
  createdAt: string;
  modifiedAt: string;
  creator: string;
  annotationIds: string[];
}

export interface AnnotatedArticleSummary {
  articleUrl: string;
  articleTitle?: string;
  count: number;
  types: AnnotationType[];
  latestTimestamp: string;
}
