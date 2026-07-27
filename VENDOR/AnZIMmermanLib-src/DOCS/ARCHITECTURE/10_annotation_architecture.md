# Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
# All rights reserved.
# Unauthorized use without prior written consent is strictly prohibited.

# EnZIM Annotation System Architecture

## Overview

The EnZIM Annotation System enables users to create rich, multi-modal annotations on ZIM archive articles and share them via lightweight sidecar bundles. This document defines the architecture, data models, sharing protocol, and integration points.

## Goals

1. **Rich Annotations**: Support text notes, voice recordings, ink drawings, photos, videos, and text highlights
2. **Click-to-Stick**: Users can tap/click anywhere on article canvas to place annotations
3. **Lightweight Sharing**: Sidecar bundles reference ZIM by fingerprint (not included)
4. **Proprietary Moat**: `.enzim` format is proprietary, increasing switching costs
5. **Cross-Platform**: Works on desktop (Tauri) and mobile (Tauri Mobile/WebView)

## Data Model

### Annotation Interface

```typescript
export type AnnotationType = 'text' | 'voice' | 'ink' | 'photo' | 'video' | 'highlight';

export interface AnnotationPosition {
  x: number;           // Percentage 0-100 of container width
  y: number;           // Percentage 0-100 of container height
  selector?: string;   // CSS selector for DOM element (for highlights)
  offset?: number;     // Character offset (for highlights)
  rect?: DOMRect;      // Bounding rectangle (optional)
}

export interface Annotation {
  id: string;                    // UUID v4
  type: AnnotationType;
  articleUrl: string;            // ZIM article path (e.g., "A/Main_Page")
  archiveId: string;             // Archive identifier
  content: string;               // Text or base64 data URL
  position?: AnnotationPosition;
  color?: 'yellow' | 'blue' | 'green' | 'pink' | 'orange';
  createdAt: string;             // ISO 8601 timestamp
  modifiedAt: string;            // ISO 8601 timestamp
  creator?: string;              // User identifier
  deviceId?: string;             // Device identifier for sync
}
```

### AnnotationManifest Interface

```typescript
export interface AnnotationManifest {
  version: '1.0';
  zimFingerprint: string;        // SHA-256 hash of ZIM file
  zimSource?: string;            // URL or identifier for ZIM source
  articleUrl: string;            // Article path within ZIM
  articleTitle?: string;         // Human-readable title
  createdAt: string;             // ISO 8601 timestamp
  modifiedAt: string;            // ISO 8601 timestamp
  creator: string;               // User who created bundle
  annotationIds: string[];        // IDs of included annotations
}
```

## Sidecar Bundle Format

### File Structure

```
annotation-bundle.enzim (ZIP)
├── manifest.json           # AnnotationManifest
├── metadata.json           # Export metadata
└── annotations/            # Media files
    ├── voice-001.webm
    ├── ink-001.png
    ├── photo-001.jpg
    └── video-001.mp4
```

### manifest.json

```json
{
  "version": "1.0",
  "zimFingerprint": "a1b2c3d4e5f6...",
  "zimSource": "https://download.kiwix.org/zim/wikipedia_en_all_maxi_2024-01.zim",
  "articleUrl": "A/Main_Page",
  "articleTitle": "Main Page",
  "createdAt": "2026-02-08T15:30:00Z",
  "modifiedAt": "2026-02-08T15:30:00Z",
  "creator": "user@example.com",
  "annotationIds": ["uuid-1", "uuid-2", "uuid-3"]
}
```

### metadata.json

```json
{
  "version": "1.0",
  "exportVersion": "EnZIM-1.0.0",
  "creator": "user@example.com",
  "deviceId": "device-uuid",
  "exportDate": "2026-02-08T15:30:00Z",
  "platform": "android",
  "annotationCount": 3,
  "totalSizeBytes": 154320
}
```

## ZIM Fingerprinting

### Algorithm

For ZIM files > 100MB:
1. Read first 1MB of file
2. Read last 1MB of file
3. Get file size in bytes
4. Concatenate: `SHA256(first1MB + last1MB + fileSizeString)`

For ZIM files <= 100MB:
1. Hash entire file with SHA-256

### Rationale

- Fast fingerprinting for large files (2MB read vs full file)
- Sufficient uniqueness for practical purposes
- File size prevents collisions from truncated files

## Storage Strategy

### Local Storage (IndexedDB + App-Private Files)

```
IndexedDB: "enzim-annotation-store"
├── annotations: Annotation[]
└── metadata: { lastSync, deviceId }

App-Private Files:
└── annotations/
    ├── voice-001.webm
    ├── ink-001.png
    ├── photo-001.jpg
    └── video-001.mp4
```

### Export Location

- Desktop: User's Downloads folder via Tauri FS API
- Mobile: Share sheet via platform intents
- Web: Browser download via Blob URL

## Sharing Protocol

### Export Flow

1. User selects annotations to share
2. System computes ZIM fingerprint
3. System creates `.enzim` bundle (ZIP)
4. Bundle is shared via:
   - Email attachment
   - QR code (for small bundles < 2KB)
   - P2P sync (future)
   - Cloud storage (future premium)

### Import Flow

1. User receives `.enzim` bundle
2. System extracts manifest
3. System verifies ZIM fingerprint against local archives
4. If match: Import annotations into store
5. If mismatch: Show warning with ZIM source info

### Fingerprint Verification

```typescript
async function verifyBundleZIM(
  bundle: Blob,
  localZimPath: string
): Promise<boolean> {
  const manifest = await extractManifest(bundle);
  const localFingerprint = await computeZIMFingerprint(localZimPath);
  return manifest.zimFingerprint === localFingerprint;
}
```

## Integration Architecture

### Component Hierarchy

```
ReaderView
├── ReaderToolbar
│   ├── Navigation (back/forward)
│   ├── Zoom controls
│   ├── Bookmark button
│   ├── Share button → ShareMenu
│   ├── Download/Upload buttons → Bundle dialogs
│   ├── Pin placement button → CanvasAnnotationLayer
│   └── Annotation panel toggle
├── ArticleContent (ref for positioning)
│   ├── TextHighlighter (selection handler)
│   └── CanvasAnnotationLayer (click-to-stick overlay)
│       └── PinnedAnnotation[] (rendered pins)
└── AnnotationPanel (conditional)
    ├── VoiceRecorder
    ├── InkCanvas
    ├── TextNoteEditor
    ├── PhotoCapture
    └── VideoCapture
```

### State Management

```typescript
// ReaderView State
interface ReaderViewState {
  showAnnotationPanel: boolean;
  panelPosition: { x: number; y: number } | null;
  placementMode: boolean;
  selectedPlacementType: AnnotationType | null;
  showShareMenu: boolean;
  shareBundleBlob: Blob | null;
  showExportDialog: boolean;
  showImportDialog: boolean;
}

// CanvasAnnotationLayer State
interface CanvasState {
  pinnedAnnotations: Array<Annotation & { displayX: number; displayY: number }>;
  isPlacingMode: boolean;
  placementType: AnnotationType | null;
  tempPosition: { x: number; y: number } | null;
}
```

## Positioning System

### Canvas Click-to-Stick

1. User enters placement mode via toolbar
2. Cursor changes to crosshair
3. User clicks on article content
4. System captures click coordinates
5. Convert screen coordinates to percentages:
   ```typescript
   function calculateRelativePosition(
     clientX: number,
     clientY: number,
     container: HTMLElement
   ): { xPercent: number; yPercent: number } {
     const rect = container.getBoundingClientRect();
     return {
       xPercent: ((clientX - rect.left) / rect.width) * 100,
       yPercent: ((clientY - rect.top) / rect.height) * 100
     };
   }
   ```
6. Store `xPercent` and `yPercent` in annotation position
7. Render pin at stored percentages (responsive)

### Text Highlighting

1. User selects text in article
2. Floating toolbar appears
3. User selects highlight color
4. System captures:
   - Selected text content
   - CSS selector path to container element
   - Character offset within container
   - Bounding rectangle for visual overlay
5. Store highlight annotation
6. Render overlay on article using selector + offset

## Mobile Considerations

### Permissions

- **Camera**: `camera` permission for photo/video capture
- **Microphone**: `microphone` permission for voice recording
- **Storage**: No external storage permission needed (app-private only)

### Camera API

```html
<!-- Mobile camera capture -->
<input type="file" accept="image/*" capture="environment">
<input type="file" accept="video/*" capture="environment">
```

### Touch Interactions

- Ink drawing: Touch events (`touchstart`, `touchmove`, `touchend`)
- Pin placement: Tap to place
- Pin selection: Tap to edit
- Long-press for context menu

## Security & Privacy

### Data Isolation

- Annotations stored in app-private storage only
- No cloud sync without explicit user action
- No analytics on annotation content

### Bundle Integrity

- ZIM fingerprint prevents misattributed annotations
- Optional: Digital signature for bundle authenticity (future)

## Performance Considerations

### Media Compression

- Images: Resize to max 1024x1024, 80% JPEG quality
- Audio: Compress to 128kbps AAC/Opus
- Video: Compress to 720p, H.264 baseline

### Lazy Loading

- Pin icons: Load immediately
- Media content: Lazy load on scroll
- Audio/video: Load on user interaction

### Storage Quotas

- Monitor `navigator.storage.estimate()`
- Warn at 80% quota usage
- Suggest cleanup at 90%

## Future Enhancements

1. **P2P Sync**: WebRTC-based direct annotation sharing
2. **Cloud Backup**: Optional encrypted cloud storage
3. **Collaboration**: Real-time annotation sharing between users
4. **AI Features**: Auto-summarize annotations, suggest related content
5. **Export Formats**: PDF export with annotations, EPUB with embedded notes

## Related Documents

- `10_annotation_architecture.mmd` - Data flow diagrams
- `CHECKLIST-downloader-annotator.md` - Implementation checklist
- `../src/types/annotation.ts` - TypeScript interfaces (implementation)
