# Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
# All rights reserved.
# Unauthorized use without prior written consent is strictly prohibited.

# CHECKLIST: Annotation System Implementation
# Theme: Downloader + Annotator with Sidecar Sharing

---

## Legend

- [ ] = Not yet begun  
- [/] = In progress  
- [x] = Completed but not fully tested  
- [✅] = Fully tested and complete  

---

## Phase 1: Core Annotation System

### 1.1 Create Annotation Types & Interfaces

**File**: `src/types/annotation.ts` (new) [✅]

- [x] Create `AnnotationType` union: `'text' | 'voice' | 'ink' | 'photo' | 'video' | 'highlight'`
- [x] Create `AnnotationPosition` interface:
  - `x: number` - Percentage 0-100
  - `y: number` - Percentage 0-100
  - `selector?: string` - CSS selector for highlights
  - `offset?: number` - Character offset
  - `rect?: DOMRect` - Bounding rectangle
- [x] Create `Annotation` interface:
  - `id: string` - UUID v4
  - `type: AnnotationType`
  - `articleUrl: string` - ZIM article path
  - `archiveId: string` - Archive identifier
  - `content: string` - Text or base64 data URL
  - `position?: AnnotationPosition`
  - `color?: 'yellow' | 'blue' | 'green' | 'pink' | 'orange'`
  - `createdAt: string` - ISO 8601
  - `modifiedAt: string` - ISO 8601
  - `creator?: string`
  - `deviceId?: string`
- [x] Create `AnnotationManifest` interface:
  - `version: '1.0'`
  - `zimFingerprint: string` - SHA-256 hash
  - `zimSource?: string` - URL or identifier
  - `articleUrl: string`
  - `articleTitle?: string`
  - `createdAt: string`
  - `modifiedAt: string`
  - `creator: string`
  - `annotationIds: string[]`

### 1.2 Create Annotation Store

**File**: `src/stores/annotationStore.ts` (new) [✅]

- [x] Import `create` from 'zustand' and `persist` from 'zustand/middleware'
- [x] Create `useAnnotationStore` with initial state:
  - `annotations: []`
  - `isLoading: false`
  - `error: null`
- [x] Implement `addAnnotation(annotation)`:
  - Input: `Omit<Annotation, 'id' | 'createdAt' | 'modifiedAt'>`
  - Generate UUID: `crypto.randomUUID()` or `Date.now().toString(36) + Math.random().toString(36).substr(2)`
  - Set `createdAt: new Date().toISOString()`
  - Set `modifiedAt: new Date().toISOString()`
  - Return generated `id: string`
- [x] Implement `updateAnnotation(id, updates)`:
  - Find by `id` in `annotations` array
  - Merge `updates` Partial<Annotation>
  - Update `modifiedAt: new Date().toISOString()`
- [x] Implement `removeAnnotation(id)`:
  - Filter `annotations` array to exclude matching `id`
- [x] Implement `getAnnotationsForArticle(articleUrl, archiveId)`:
  - Filter by `articleUrl` and `archiveId`
  - Return `Annotation[]`
- [x] Implement `getAnnotationById(id)`:
  - Find by `id` in `annotations` array
  - Return `Annotation | undefined`
- [x] Implement `getAllAnnotations()`:
  - Return full `annotations` array
- [x] Implement `importAnnotations(annotations)`:
  - Concatenate new annotations to existing array
  - Prevent duplicates by checking `id`
- [x] Implement `exportAnnotationsForArticle(articleUrl, archiveId)`:
  - Return filtered annotations as `Annotation[]`
- [x] Add persist middleware:
  - `name: 'enzim-annotation-store'`
  - `partialize: (state) => ({ annotations: state.annotations })`

### 1.3 Port Voice Recording from AnnotationPanel

**File**: `src/components/annotations/VoiceRecorder.tsx` (new) [✅]

- [x] Create component with props:
  - `onRecordingComplete: (base64Audio: string) => void`
  - `onCancel?: () => void`
- [x] Add state: `isRecording: boolean`
- [x] Add refs:
  - `mediaRecorderRef: React.MutableRefObject<MediaRecorder | null>`
  - `audioChunksRef: React.MutableRefObject<Blob[]>`
- [x] Implement `startRecording(): Promise<void>`:
  - Call `navigator.mediaDevices.getUserMedia({ audio: true })`
  - Create `MediaRecorder(stream)`
  - Set `ondataavailable` to push to `audioChunksRef.current`
  - Set `onstop` to create blob from chunks
  - Call `mediaRecorder.start()`
  - Set `isRecording: true`
- [x] Implement `stopRecording(): void`:
  - Call `mediaRecorderRef.current?.stop()`
  - Set `isRecording: false`
- [x] Implement `convertBlobToBase64(blob: Blob): Promise<string>`:
  - Use `FileReader.readAsDataURL(blob)`
  - Return base64 string
- [x] Add UI:
  - Record button with pulse animation when `isRecording`
  - Mic icon from Lucide: `<Mic />`
  - MicOff icon when recording: `<MicOff />`
  - Playback preview with `<audio controls>` after recording
  - Save/Cancel buttons

### 1.4 Port Ink Drawing from AnnotationPanel

**File**: `src/components/annotations/InkCanvas.tsx` (new) [✅]

- [x] Create component with props:
  - `onSave: (dataUrl: string) => void`
  - `onClear?: () => void`
  - `initialDrawing?: string` - base64 PNG for edit mode
- [x] Add refs:
  - `canvasRef: React.RefObject<HTMLCanvasElement>`
  - `ctxRef: React.MutableRefObject<CanvasRenderingContext2D | null>`
- [x] Add state: `isDrawing: boolean`
- [x] Implement canvas setup in `useEffect`:
  - Get 2D context: `canvas.getContext('2d')`
  - Set styles: `strokeStyle = '#f97316'`, `lineWidth = 3`, `lineCap = 'round'`, `lineJoin = 'round'`
  - Canvas size: `width={280} height={200}` responsive
- [x] Implement `startDrawing(e: React.MouseEvent<HTMLCanvasElement>): void`:
  - Get canvas bounding rect
  - `ctx.beginPath()`
  - `ctx.moveTo(e.clientX - rect.left, e.clientY - rect.top)`
  - Set `isDrawing: true`
- [x] Implement `draw(e: React.MouseEvent<HTMLCanvasElement>): void`:
  - Check `isDrawing` guard
  - `ctx.lineTo(e.clientX - rect.left, e.clientY - rect.top)`
  - `ctx.stroke()`
- [x] Implement `stopDrawing(): void`:
  - Set `isDrawing: false`
- [x] Implement `clearCanvas(): void`:
  - `ctx.clearRect(0, 0, canvas.width, canvas.height)`
- [x] Implement `saveCanvas(): void`:
  - Call `canvas.toDataURL('image/png')`
  - Call `onSave(dataUrl)`
- [x] Add UI:
  - Canvas element with event handlers
  - Clear button with `<Trash2 />` icon
  - Save button with `<Save />` icon

### 1.5 Port Text Notes from AnnotationPanel

**File**: `src/components/annotations/TextNoteEditor.tsx` (new) [✅]

- [x] Create component with props:
  - `onSave: (text: string, color: AnnotationColor) => void`
  - `onCancel?: () => void`
  - `initialText?: string`
  - `initialColor?: AnnotationColor`
- [x] Add state:
  - `noteText: string`
  - `noteColor: AnnotationColor` ('yellow' | 'blue' | 'green' | 'pink')
- [x] Define color constants:
  ```typescript
  const COLORS = [
    { name: 'yellow', class: 'bg-yellow-400', value: 'yellow' },
    { name: 'blue', class: 'bg-blue-400', value: 'blue' },
    { name: 'green', class: 'bg-green-400', value: 'green' },
    { name: 'pink', class: 'bg-pink-400', value: 'pink' },
  ] as const;
  ```
- [x] Implement `saveTextNote(): void`:
  - Guard: `if (!noteText.trim()) return`
  - Call `onSave(noteText.trim(), noteColor)`
  - Reset `noteText: ''`
- [x] Add UI:
  - `<textarea>` with `placeholder="Type your note..."`
  - Color picker buttons (4 circular colored buttons)
  - Selected color indicator with ring
  - Save button with `<Save />` icon, disabled when empty

### 1.6 Implement Photo Capture

**File**: `src/components/annotations/PhotoCapture.tsx` (new) [✅]

- [x] Create component with props:
  - `onCapture: (dataUrl: string) => void`
  - `onCancel?: () => void`
- [x] Add state: `previewUrl: string | null`, `isProcessing: boolean`
- [x] Implement `handleFileSelect(e: React.ChangeEvent<HTMLInputElement>): void`:
  - Get `e.target.files[0]`
  - Call `convertToBase64(file)`
- [x] Implement `convertToBase64(file: File): Promise<string>`:
  - Use `FileReader.readAsDataURL(file)`
  - Return base64 string
- [x] Implement `compressImage(dataUrl: string): Promise<string>`:
  - Create canvas element
  - Load image from dataUrl
  - Calculate scaled dimensions (max 1024x1024)
  - Draw scaled image to canvas
  - Return `canvas.toDataURL('image/jpeg', 0.8)`
- [x] Add UI:
  - File input: `<input type="file" accept="image/*" capture="environment">` (mobile camera)
  - Desktop fallback: `<input type="file" accept="image/*">`
  - Preview thumbnail when selected
  - Retake button, Save button
  - Loading spinner during compression

### 1.7 Implement Video Capture

**File**: `src/components/annotations/VideoCapture.tsx` (new) [✅]

- [x] Create component with props:
  - `onCapture: (dataUrl: string) => void`
  - `onCancel?: () => void`
- [x] Add constants: `MAX_FILE_SIZE_MB = 10`
- [x] Add state: `selectedFile: File | null`, `error: string | null`
- [x] Implement `handleFileSelect(e: React.ChangeEvent<HTMLInputElement>): void`:
  - Get `e.target.files[0]`
  - Check `file.size <= MAX_FILE_SIZE_MB * 1024 * 1024`
  - If oversized, set `error: 'Video must be under 10MB'`
  - Otherwise call `convertToBase64(file)`
- [x] Implement `convertToBase64(file: File): Promise<string>`:
  - Use `FileReader.readAsDataURL(file)`
  - Return base64 string
- [x] Add UI:
  - File input: `<input type="file" accept="video/*" capture="environment">`
  - Desktop fallback: `<input type="file" accept="video/*">`
  - Video preview when selected
  - Error message display
  - Record again button, Save button

### 1.8 Implement Text Highlighting

**File**: `src/components/annotations/TextHighlighter.tsx` (new) [✅]

- [x] Create component with props:
  - `onHighlight: (text: string, position: AnnotationPosition, color: AnnotationColor) => void`
  - `containerRef: React.RefObject<HTMLElement>`
- [x] Add state: `showToolbar: boolean`, `toolbarPosition: { x: number; y: number }`
- [x] Implement `getSelectionInfo(): { text: string, rect: DOMRect, selector: string, offset: number } | null`:
  - Get `window.getSelection()`
  - Check `selection.rangeCount > 0`
  - Get `range = selection.getRangeAt(0)`
  - Extract `text = selection.toString().trim()`
  - Get `rect = range.getBoundingClientRect()`
  - Get `container = containerRef.current`
  - Find common ancestor element
  - Generate `selector = generateSelector(element)`
  - Calculate `offset` from container start
- [x] Implement `generateSelector(element: Element): string`:
  - Build CSS selector path from element to container
  - Use `element.tagName` + `:nth-child()` indices
  - Example: `div.article > p:nth-child(3) > span`
- [x] Implement `handleHighlight(color: AnnotationColor): void`:
  - Call `getSelectionInfo()`
  - If valid, call `onHighlight(text, { selector, offset, rect }, color)`
  - Clear selection: `window.getSelection().removeAllRanges()`
  - Hide toolbar
- [x] Add event listener in `useEffect`:
  - Listen for `mouseup` on document
  - If selection exists and is within container, show toolbar
- [x] Add UI (floating toolbar):
  - Position at `toolbarPosition`
  - 4 color buttons: yellow, blue, green, pink
  - Icons from Lucide: `<Highlighter />` or colored circles

### 1.9 Create Annotation Panel Component

**File**: `src/components/annotations/AnnotationPanel.tsx` (new) [✅]

- [x] Create component with props:
  - `articleUrl: string`
  - `archiveId: string`
  - `onClose: () => void`
  - `initialPosition?: { x: number; y: number }` - for click-to-stick
  - `initialType?: AnnotationType` - pre-select tab
- [x] Add imports:
  - `VoiceRecorder`, `InkCanvas`, `TextNoteEditor`, `PhotoCapture`, `VideoCapture`
  - `useAnnotationStore`
  - Icons: `<X />`, `<Mic />`, `<Pencil />`, `<StickyNote />`, `<Camera />`, `<Video />`, `<Trash2 />`
- [x] Add state: `activeTab: 'text' | 'voice' | 'ink' | 'photo' | 'video'`
- [x] Get store methods:
  - `const { addAnnotation, getAnnotationsForArticle, removeAnnotation } = useAnnotationStore()`
- [x] Get current annotations: `const annotations = getAnnotationsForArticle(articleUrl, archiveId)`
- [x] Implement tab handlers:
  - `handleVoiceSave(base64Audio)`:
    ```typescript
    addAnnotation({
      type: 'voice',
      articleUrl,
      archiveId,
      content: base64Audio,
      position: initialPosition,
      createdAt: new Date().toISOString(),
      modifiedAt: new Date().toISOString(),
    })
    ```
  - `handleInkSave(dataUrl)` - similar with `type: 'ink'`
  - `handleTextSave(text, color)` - similar with `type: 'text'`
  - `handlePhotoSave(dataUrl)` - similar with `type: 'photo'`
  - `handleVideoSave(dataUrl)` - similar with `type: 'video'`
- [x] Implement `handleDeleteAnnotation(id: string)`:
  - Call `removeAnnotation(id)`
- [x] Add UI layout:
  - Header: "Annotations" title, close button `<X />`
  - Tabs: Text, Voice, Ink, Photo, Video with icons
  - Tab content area with active component
  - Saved annotations list at bottom:
    - Show count: `annotations.length`
    - List with type icon, preview, timestamp, delete button
    - For voice: `<audio controls>`
    - For ink: `<img src={content} />`
    - For text: truncated text

### 1.10 Implement Canvas Click-to-Stick Annotations

**File**: `src/components/annotations/CanvasAnnotationLayer.tsx` (new) [✅]

- [x] Create component with props:
  - `articleUrl: string`
  - `archiveId: string`
  - `contentRef: React.RefObject<HTMLElement>` - ref to article content container
  - `isPlacingMode: boolean` - controlled from parent
  - `placementType: AnnotationType | null`
  - `onPlacement: (position: { x: number; y: number }) => void`
- [x] Get store data:
  - `const annotations = getAnnotationsForArticle(articleUrl, archiveId)`
- [x] Implement `calculateRelativePosition(clientX, clientY)`:
  ```typescript
  const container = contentRef.current;
  const rect = container.getBoundingClientRect();
  return {
    x: ((clientX - rect.left) / rect.width) * 100,  // percentage
    y: ((clientY - rect.top) / rect.height) * 100,   // percentage
  };
  ```
- [x] Implement `handleCanvasClick(e: React.MouseEvent)`:
  - Guard: only process if `isPlacingMode` is true
  - Get click position: `e.clientX`, `e.clientY`
  - Call `calculateRelativePosition(clientX, clientY)`
  - Call `onPlacement({ x, y })`
- [x] Implement `renderPinnedAnnotation(annotation: Annotation)`:
  - Calculate display position from percentages:
    ```typescript
    const container = contentRef.current;
    const displayX = (annotation.position.x / 100) * container.clientWidth;
    const displayY = (annotation.position.y / 100) * container.clientHeight;
    ```
  - Render pin icon at `{ left: displayX, top: displayY }`
  - Pin icon: MapPin from Lucide with annotation color
  - Click handler to open/edit annotation
- [x] Add UI:
  - Absolute positioned overlay covering `contentRef`
  - `pointer-events: none` when not in placing mode
  - `pointer-events: auto` + `cursor: crosshair` when in placing mode
  - Render all pinned annotations from `annotations` array
  - Pin icons with hover tooltip showing annotation preview

### 1.11 Integrate into ReaderView

**File**: `src/components/reader/ReaderView.tsx` (modify) [✅]

- [x] Add imports:
  - `AnnotationPanel` from '../annotations/AnnotationPanel'
  - `TextHighlighter` from '../annotations/TextHighlighter'
  - `CanvasAnnotationLayer` from '../annotations/CanvasAnnotationLayer'
  - Icons: `<MapPin />`, `<Highlighter />`, `<MessageSquare />` from Lucide
- [x] Add refs:
  - `articleContentRef: React.RefObject<HTMLDivElement>`
- [x] Add state:
  - `showAnnotationPanel: boolean`
  - `panelPosition: { x: number; y: number } | null`
  - `placementMode: boolean`
  - `selectedPlacementType: AnnotationType`
  - `highlightMode: boolean`
- [x] Add toolbar buttons (in ReaderToolbar div):
  - Pin placement button: `<MapPin className="w-4 h-4" />`
    - Toggle `placementMode` on click
    - Show dropdown when `placementMode` to select type
  - Highlight button: `<Highlighter className="w-4 h-4" />`
    - Toggle `highlightMode`
  - Annotation panel button: `<MessageSquare className="w-4 h-4" />`
    - Toggle `showAnnotationPanel`
- [x] Implement `handleHighlight(text, position, color)`:
  ```typescript
  addAnnotation({
    type: 'highlight',
    articleUrl: currentArticle.url,
    archiveId: currentArchive.id,
    content: text,
    position,
    color,
  })
  ```
- [x] Implement `handleCanvasPlacement(x, y)`:
  ```typescript
  setPanelPosition({ x, y });
  setShowAnnotationPanel(true);
  setPlacementMode(false);
  ```
- [x] Add conditional rendering:
  - `{showAnnotationPanel && <AnnotationPanel ... />}`
  - `<TextHighlighter containerRef={articleContentRef} onHighlight={handleHighlight} />`
  - `<CanvasAnnotationLayer ... />` overlay on article content area
- [x] Wrap article content with ref:
  - `<article ref={articleContentRef} ...>`

---

## Phase 2: Sidecar Bundle Format

### 2.1 Create Bundle Structure Constants

**File**: `src/utils/annotationBundle.ts` (new)

- [ ] Define constants:
  ```typescript
  export const BUNDLE_VERSION = '1.0';
  export const BUNDLE_EXTENSION = '.enzim';
  export const MANIFEST_FILENAME = 'manifest.json';
  export const METADATA_FILENAME = 'metadata.json';
  export const ANNOTATIONS_DIR = 'annotations/';
  ```

### 2.2 Implement ZIM Fingerprinting

**File**: `src/utils/annotationBundle.ts` (add)

- [ ] Install dependency: `npm install jszip` and `npm install @types/jszip --save-dev`
- [ ] Implement `computeZIMFingerprint(filePath: string): Promise<string>`:
  - For Tauri: Use `readFile` from `@tauri-apps/plugin-fs`
  - Read file stats for size
  - If size > 100MB:
    - Read first 1MB: `readFile(filePath, { offset: 0, length: 1024 * 1024 })`
    - Read last 1MB: `readFile(filePath, { offset: size - 1024 * 1024, length: 1024 * 1024 })`
    - Concatenate: `firstChunk + lastChunk + size.toString()`
  - Else (size <= 100MB):
    - Read entire file
  - Compute SHA-256: `await crypto.subtle.digest('SHA-256', data)`
  - Convert to hex string
  - Return 64-character hex fingerprint

### 2.3 Create Bundle Export Function

**File**: `src/utils/annotationBundle.ts` (add)

- [ ] Implement `exportAnnotationBundle(annotations, zimFingerprint, zimSource, articleUrl, articleTitle, creator): Promise<Blob>`:
  - Create new `JSZip()` instance
  - Create manifest:
    ```typescript
    const manifest: AnnotationManifest = {
      version: '1.0',
      zimFingerprint,
      zimSource,
      articleUrl,
      articleTitle,
      createdAt: new Date().toISOString(),
      modifiedAt: new Date().toISOString(),
      creator,
      annotationIds: annotations.map(a => a.id),
    };
    ```
  - Add manifest to ZIP: `zip.file(MANIFEST_FILENAME, JSON.stringify(manifest, null, 2))`
  - Create metadata:
    ```typescript
    const metadata = {
      version: '1.0',
      exportVersion: 'EnZIM-1.0.0',
      creator,
      deviceId: await getDeviceId(), // from storage or generate
      exportDate: new Date().toISOString(),
      platform: getPlatform(), // 'desktop' | 'mobile' | 'web'
      annotationCount: annotations.length,
    };
    ```
  - Add metadata: `zip.file(METADATA_FILENAME, JSON.stringify(metadata, null, 2))`
  - Create annotations folder: `zip.folder(ANNOTATIONS_DIR)`
  - For each annotation:
    - Extract base64 data from `annotation.content`
    - Convert to binary blob
    - Add to annotations folder: `zip.file(\`annotations/\${annotation.id}.\${ext}\`, blob)`
    - Extensions: voice → .webm, ink → .png, photo → .jpg, video → .mp4, text → .txt
  - Generate ZIP blob: `const blob = await zip.generateAsync({ type: 'blob' })`
  - Return blob

### 2.4 Create Bundle Import Function

**File**: `src/utils/annotationBundle.ts` (add)

- [ ] Implement `importAnnotationBundle(bundleBlob, expectedZimFingerprint): Promise<{ manifest, annotations }>`:
  - Load ZIP: `const zip = await JSZip.loadAsync(bundleBlob)`
  - Extract manifest: `const manifestFile = zip.file(MANIFEST_FILENAME)`
  - Parse manifest: `const manifest: AnnotationManifest = JSON.parse(await manifestFile.async('string'))`
  - Validate fingerprint:
    ```typescript
    if (manifest.zimFingerprint !== expectedZimFingerprint) {
      throw new BundleValidationError(
        `ZIM fingerprint mismatch. Expected: ${expectedZimFingerprint}, Got: ${manifest.zimFingerprint}`
      );
    }
    ```
  - Extract annotations folder
  - For each annotation ID in `manifest.annotationIds`:
    - Determine file extension based on type (from stored metadata or filename)
    - Read binary file: `const blob = await zip.file(\`annotations/\${id}.\${ext}\`).async('blob')`
    - Convert to base64: `const base64 = await blobToBase64(blob)`
    - Reconstruct annotation object
  - Return `{ manifest, annotations }`

### 2.5 Add Fingerprint Check on Import

**File**: `src/utils/annotationBundle.ts` (add)

- [ ] Create `BundleValidationError` class extending `Error`
- [ ] Implement `verifyZIMFingerprint(bundleBlob, expectedFingerprint): Promise<boolean>`:
  - Extract manifest without full import
  - Compare `manifest.zimFingerprint` with `expectedFingerprint`
  - Return boolean
  - Log to console: `console.log('Fingerprint match:', result)`

### 2.6 Create Export UI Component

**File**: `src/components/annotations/BundleExportDialog.tsx` (new)

- [ ] Create component with props:
  - `annotations: Annotation[]`
  - `zimFingerprint: string`
  - `zimSource?: string`
  - `articleUrl: string`
  - `articleTitle: string`
  - `onExport?: (blob: Blob) => void`
  - `onClose: () => void`
- [ ] Add state:
  - `isExporting: boolean`
  - `exportProgress: number` (0-100)
  - `exportStage: 'idle' | 'preparing' | 'packaging' | 'complete'`
- [ ] Implement `handleExport(): Promise<void>`:
  - Set `isExporting: true`
  - Set `exportStage: 'preparing'`
  - Call `exportAnnotationBundle(...)`
  - Set `exportStage: 'packaging'` during ZIP creation
  - On complete: `exportStage: 'complete'`
  - Trigger download:
    ```typescript
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = generateFilename();
    a.click();
    URL.revokeObjectURL(url);
    ```
  - Call `onExport?.(blob)`
- [ ] Implement `generateFilename(): string`:
  - Sanitize article title: replace non-alphanumeric with '-'
  - Format: `\${sanitizedTitle}--\${YYYY-MM-DD}.enzim`
- [ ] Add UI:
  - Dialog/modal container
  - Header: "Export Annotations"
  - Annotation count display
  - Estimated size (sum of content lengths)
  - Progress bar showing `exportProgress`
  - Stage label: preparing → packaging → complete
  - Download button (enabled on complete)
  - Close button

### 2.7 Create Import UI Component

**File**: `src/components/annotations/BundleImportDialog.tsx` (new)

- [ ] Create component with props:
  - `expectedZimFingerprint: string`
  - `onImport: (annotations: Annotation[]) => void`
  - `onClose: () => void`
- [ ] Add state:
  - `selectedFile: File | null`
  - `isImporting: boolean`
  - `importError: string | null`
  - `verificationStatus: 'pending' | 'verified' | 'mismatch'`
  - `manifestPreview: AnnotationManifest | null`
- [ ] Implement `handleFileSelect(e)`:
  - Get `e.target.files[0]`
  - Validate extension: `.enzim`
  - Set `selectedFile`
  - Auto-trigger verification
- [ ] Implement `verifyBundle(): Promise<void>`:
  - Call `verifyZIMFingerprint(selectedFile, expectedZimFingerprint)`
  - Set `verificationStatus` based on result
  - If mismatch, show warning with manifest zimSource info
- [ ] Implement `handleImport(): Promise<void>`:
  - Set `isImporting: true`
  - Call `importAnnotationBundle(selectedFile, expectedZimFingerprint)`
  - Call `onImport(annotations)`
  - Set `isImporting: false`
  - Call `onClose()`
- [ ] Add UI:
  - File input with `.enzim` accept filter
  - Verification status badge:
    - Pending: gray
    - Verified: green with checkmark
    - Mismatch: red with warning icon
  - If mismatch, show:
    - Warning message
    - Expected ZIM info
    - Bundle's ZIM source URL (if available)
  - Import button (disabled if mismatch, enabled with warning if forced)
  - Progress spinner during import

### 2.8 Integrate Bundle Export into ReaderView

**File**: `src/components/reader/ReaderView.tsx` (already modified in 1.11, ensure complete)

- [ ] Verify toolbar buttons exist:
  - Export button opens `BundleExportDialog`
  - Import button opens `BundleImportDialog`
- [ ] Verify handler functions exist:
  - `handleBundleImport(annotations)` iterates and calls `addAnnotation()`
  - Shows success toast with count

---

## Phase 3: Sharing Infrastructure

### 3.1 Create Share Intent Handler

**File**: `src/utils/shareIntent.ts` (new)

- [ ] Implement `shareViaIntent(title, text, files?): Promise<boolean>`:
  - Check `navigator.canShare?.({ files })` for file support
  - Try Web Share API:
    ```typescript
    try {
      await navigator.share({ title, text, files });
      return true;
    } catch (err) {
      if (err.name !== 'AbortError') {
        console.error('Share failed:', err);
      }
      return false;
    }
    ```
  - Fallback for unsupported platforms:
    - Copy to clipboard: `navigator.clipboard.writeText(text)`
    - Show toast: "Link copied to clipboard"
- [ ] Implement `shareViaEmail(bundleBlob, subject, body): Promise<void>`:
  - Create mailto link with attachment reference
  - Or use platform-specific email APIs via Tauri

### 3.2 Create QR Code Generator

**File**: `src/components/annotations/QRShareDialog.tsx` (new)

- [ ] Install dependency: `npm install qrcode` and `npm install @types/qrcode --save-dev`
- [ ] Create component with props:
  - `bundleData: Blob | string`
  - `onClose: () => void`
- [ ] Add state:
  - `qrDataUrl: string | null`
  - `isGenerating: boolean`
  - `error: string | null`
- [ ] Implement `generateQR(): Promise<void>`:
  - Convert bundle to data URL (if Blob)
  - Check size: `dataUrl.length < 2000` (QR limit ~2KB for standard scanning)
  - If oversized, set `error: 'Bundle too large for QR code'`
  - Generate QR: `const dataUrl = await QRCode.toDataURL(data, { width: 300 })`
  - Set `qrDataUrl`
- [ ] Implement `downloadQR(): void`:
  - Create anchor link with `qrDataUrl`
  - Set download filename: `annotations-qr.png`
  - Trigger click
- [ ] Add UI:
  - QR code display (300x300)
  - Download button
  - Copy data URL button
  - Size warning if bundle > 2KB
  - Close button

### 3.3 Create Annotation Share Menu

**File**: `src/components/annotations/ShareMenu.tsx` (new)

- [ ] Create component with props:
  - `bundleBlob: Blob`
  - `articleTitle: string`
  - `onShareViaEmail: () => void`
  - `onShareViaQR: () => void`
  - `onShareViaP2P?: () => void`
  - `onCopyLink: () => void`
  - `onClose: () => void`
- [ ] Add state: `isOpen: boolean` (controlled by parent)
- [ ] Implement share options:
  - "Email" - calls `onShareViaEmail()`
  - "QR Code" - calls `onShareViaQR()`
  - "Copy Link" - calls `onCopyLink()`
  - "Nearby Share" - calls `onShareViaP2P?.()` (disabled if not provided)
- [ ] Add UI:
  - Dropdown/popover menu
  - Share option buttons with icons
  - File size display
  - Close button

### 3.4 Integrate Share Menu into ReaderView

**File**: `src/components/reader/ReaderView.tsx` (verify from 1.11)

- [ ] Verify `handlePrepareShare()` creates bundle and opens share menu
- [ ] Implement share handlers:
  - `handleShareViaEmail()` - calls `shareViaEmail(bundleBlob, ...)`
  - `handleShareViaQR()` - opens QR dialog
  - `handleCopyLink()` - copies shareable description to clipboard

### 3.5 Create P2P Sync Service (Future Foundation)

**File**: `src/services/p2pSync.ts` (new - stub)

- [ ] Define interface `P2PSyncService`:
  ```typescript
  export interface P2PSyncService {
    initialize(): Promise<void>;
    createRoom(): string; // Returns room ID
    joinRoom(roomId: string): Promise<void>;
    broadcastAnnotations(annotations: Annotation[]): void;
    onAnnotationsReceived(callback: (annotations: Annotation[]) => void): void;
    disconnect(): void;
  }
  ```
- [ ] Create stub implementation:
  ```typescript
  export const p2pSync: P2PSyncService = {
    initialize: async () => { console.log('P2P: initialize (stub)'); },
    createRoom: () => 'stub-room-id',
    joinRoom: async () => { console.log('P2P: joinRoom (stub)'); },
    broadcastAnnotations: () => { console.log('P2P: broadcast (stub)'); },
    onAnnotationsReceived: () => { /* no-op */ },
    disconnect: () => { console.log('P2P: disconnect (stub)'); },
  };
  ```
- [ ] Add TODO comment: "Full WebRTC implementation in future phase"

---

## Phase 4: Additional Integration

### 4.1 Create Annotation Sidebar View

**File**: `src/components/views/AnnotationsView.tsx` (new)

- [ ] Create component (similar structure to `BookmarksView.tsx`)
- [ ] Add state:
  - `filterType: AnnotationType | 'all'`
  - `searchQuery: string`
  - `selectedIds: string[]` (for bulk actions)
- [ ] Get store data:
  - `const { annotations, removeAnnotation } = useAnnotationStore()`
  - `const { setCurrentArchive, setCurrentArticle } = useArchiveStore()`
- [ ] Implement filtered annotations:
  ```typescript
  const filteredAnnotations = annotations.filter(a => {
    const matchesType = filterType === 'all' || a.type === filterType;
    const matchesSearch = !searchQuery || 
      a.content.toLowerCase().includes(searchQuery.toLowerCase()) ||
      a.articleUrl.toLowerCase().includes(searchQuery.toLowerCase());
    return matchesType && matchesSearch;
  });
  ```
- [ ] Implement `handleAnnotationClick(annotation)`:
  - Find archive by `annotation.archiveId`
  - Call `setCurrentArchive(archive)`
  - Call `setCurrentArticle({ url: annotation.articleUrl, title: '...', archiveId: annotation.archiveId })`
  - Navigate to article with annotation ID in URL
- [ ] Add UI:
  - Header: "Annotations" title, count badge
  - Filter tabs: All, Text, Voice, Ink, Photo, Video
  - Search input with clear button
  - Annotation list with `AnnotationListItem` components
  - Empty state when no annotations

### 4.2 Add Annotation Navigation Router

**File**: `src/router.tsx` (modify)

- [ ] Add route with query param:
  ```typescript
  <Route path="/article/:archiveId/:articleUrl" element={<ArticleView />} />
  ```
- [ ] In `ArticleView` or `ReaderView`, check query params:
  ```typescript
  const [searchParams] = useSearchParams();
  const annotationId = searchParams.get('annotationId');
  ```
- [ ] Implement `handleAnnotationDeepLink()`:
  - If `annotationId` present, find annotation
  - Scroll to annotation position
  - Highlight the annotation temporarily (pulse animation)

### 4.3 Create Annotation List Item Component

**File**: `src/components/annotations/AnnotationListItem.tsx` (new)

- [ ] Create component with props:
  - `annotation: Annotation`
  - `onDelete: () => void`
  - `onNavigate: () => void`
  - `isSelected?: boolean`
  - `onSelect?: (selected: boolean) => void`
- [ ] Implement preview by type:
  - `text`: Truncated text (first 50 chars)
  - `voice`: `<audio controls src={annotation.content} />` (compact)
  - `ink`: `<img src={annotation.content} className="h-16 w-auto" />`
  - `photo`: `<img src={annotation.content} className="h-16 w-auto" />`
  - `video`: `<video src={annotation.content} className="h-16 w-auto" />`
  - `highlight`: Text snippet with highlight color background
- [ ] Add UI:
  - Checkbox for bulk selection (if `onSelect` provided)
  - Type icon (Lucide icons)
  - Preview content
  - Timestamp: `new Date(annotation.createdAt).toLocaleDateString()`
  - Article reference (if in global view)
  - Delete button with `<Trash2 />` icon

### 4.4 Add Annotation Search Function

**File**: `src/stores/annotationStore.ts` (add method)

- [ ] Implement `searchAnnotations(query: string): Annotation[]`:
  ```typescript
  searchAnnotations(query) {
    const lowerQuery = query.toLowerCase();
    return this.annotations.filter(a => {
      // Search in content (for text annotations)
      if (a.type === 'text' && a.content.toLowerCase().includes(lowerQuery)) {
        return true;
      }
      // Search in article URL
      if (a.articleUrl.toLowerCase().includes(lowerQuery)) {
        return true;
      }
      // Search in creation date (YYYY-MM-DD format)
      if (a.createdAt.includes(query)) {
        return true;
      }
      return false;
    });
  }
  ```

### 4.5 Create Bulk Operations UI

**File**: `src/components/annotations/BulkAnnotationActions.tsx` (new)

- [ ] Create component with props:
  - `selectedIds: string[]`
  - `selectedAnnotations: Annotation[]`
  - `onExportSelected: () => void`
  - `onDeleteSelected: () => void`
  - `onClearSelection: () => void`
- [ ] Add UI:
  - "X selected" count display
  - Export Selected button
  - Delete Selected button (red, with confirmation)
  - Clear Selection button

---

## Phase 5: Polish & Performance

### 5.1 Implement Lazy Loading for Media

**File**: `src/components/annotations/AnnotationMedia.tsx` (new)

- [ ] Create component with props:
  - `annotation: Annotation`
  - `lazy?: boolean` (default: true)
- [ ] Implement lazy loading:
  - For images: `<img loading="lazy" src={content} />`
  - For video: Use `IntersectionObserver` to load when in viewport
- [ ] Add placeholder:
  - Show spinner/placeholder while loading
  - Fade in when loaded

### 5.2 Add Annotation Compression

**File**: `src/utils/mediaCompression.ts` (new)

- [ ] Implement `compressImage(dataUrl, maxWidth, maxHeight, quality): Promise<string>`:
  - Create canvas element
  - Load image: `const img = new Image(); img.src = dataUrl;`
  - Calculate scale: `Math.min(maxWidth / img.width, maxHeight / img.height, 1)`
  - Draw scaled: `ctx.drawImage(img, 0, 0, scaledWidth, scaledHeight)`
  - Export: `canvas.toDataURL('image/jpeg', quality)`
  - Return compressed data URL
- [ ] Implement `compressAudio(blob, targetBitrate): Promise<Blob>`:
  - Use AudioContext to decode
  - Re-encode with MediaRecorder at lower bitrate
  - Return compressed blob

### 5.3 Create Annotation Storage Quota Check

**File**: `src/stores/annotationStore.ts` (add method)

- [ ] Implement `getStorageUsage(): Promise<{ used: number; quota: number; percentage: number }>`:
  ```typescript
  async getStorageUsage() {
    if ('storage' in navigator && 'estimate' in navigator.storage) {
      const estimate = await navigator.storage.estimate();
      const used = estimate.usage || 0;
      const quota = estimate.quota || 0;
      return {
        used,
        quota,
        percentage: quota > 0 ? (used / quota) * 100 : 0,
      };
    }
    return { used: 0, quota: 0, percentage: 0 };
  }
  ```
- [ ] Add warning threshold: 80%
- [ ] Add critical threshold: 90%

### 5.4 Add Export Progress Indicators

**File**: `src/components/annotations/BundleExportDialog.tsx` (enhance)

- [ ] Add state: `exportStage: 'idle' | 'preparing' | 'compressing' | 'packaging' | 'complete'`
- [ ] Update progress bar based on stage:
  - Preparing: 0-20%
  - Compressing: 20-60%
  - Packaging: 60-90%
  - Complete: 100%
- [ ] Show file size estimate before export

### 5.5 Implement Annotation Backup/Restore

**File**: `src/utils/annotationBackup.ts` (new)

- [ ] Implement `exportAllAnnotations(): Promise<Blob>`:
  - Get all annotations from store
  - Create full library bundle with `.enzim-full` extension
  - Include all articles, all annotations
  - Format similar to single-article bundle but with multiple articles
- [ ] Implement `importAllAnnotations(bundleBlob): Promise<void>`:
  - Validate bundle format
  - Extract all annotations
  - Merge with existing (prompt for conflict resolution)
- [ ] Add UI in `SettingsView`:
  - "Export All Annotations" button
  - "Import All Annotations" button with file picker
  - Warning about overwriting existing data

### 5.6 Add Annotation Keyboard Shortcuts

**File**: `src/hooks/useAnnotationShortcuts.ts` (new)

- [ ] Create hook:
  ```typescript
  export function useAnnotationShortcuts({
    onTogglePanel,
    onToggleHighlight,
    onTogglePlacement,
    onClosePanel,
  }) {
    useEffect(() => {
      const handler = (e: KeyboardEvent) => {
        if (e.ctrlKey || e.metaKey) {
          if (e.shiftKey && e.key === 'A') {
            e.preventDefault();
            onTogglePanel();
          }
          if (e.shiftKey && e.key === 'H') {
            e.preventDefault();
            onToggleHighlight();
          }
          if (e.shiftKey && e.key === 'P') {
            e.preventDefault();
            onTogglePlacement();
          }
        }
        if (e.key === 'Escape') {
          onClosePanel();
        }
      };
      window.addEventListener('keydown', handler);
      return () => window.removeEventListener('keydown', handler);
    }, []);
  }
  ```
- [ ] Integrate in `ReaderView.tsx` useEffect

### 5.7 Create Annotation Tour/Onboarding

**File**: `src/components/onboarding/AnnotationTour.tsx` (new)

- [ ] Create component with props:
  - `onComplete: () => void`
  - `onSkip?: () => void`
- [ ] Define tour steps:
  1. "Highlight text by selecting it"
  2. "Click the pin button to place annotations"
  3. "Record voice notes on any article"
  4. "Draw ink sketches with your finger or mouse"
  5. "Export annotations to share with others"
- [ ] Store completion: `localStorage.setItem('annotation-tour-completed', 'true')`
- [ ] Check on app load: skip if already completed

### 5.8 Final Testing Checklist

- [ ] Voice recording works on Chrome, Firefox, Safari, mobile browsers
- [ ] Ink drawing works with mouse and touch
- [ ] Photo/video capture works on mobile (camera API)
- [ ] Text highlighting works across article scroll positions
- [ ] Pins render at correct positions after page resize
- [ ] Bundle export produces valid .enzim file
- [ ] Bundle import validates ZIM fingerprint correctly
- [ ] Share menu opens on all platforms
- [ ] QR codes scan correctly and decode bundles
- [ ] Annotations persist across app restarts
- [ ] Storage quota warnings appear when near limit
- [ ] All keyboard shortcuts work
- [ ] Onboarding tour completes without errors

---

## Dependencies to Install

```bash
# Core
npm install jszip
npm install @types/jszip --save-dev

# QR Code
npm install qrcode
npm install @types/qrcode --save-dev

# UUID generation (if not using crypto.randomUUID)
npm install uuid
npm install @types/uuid --save-dev

# File type detection (optional)
npm install file-type
```

---

## Notes for Implementers

1. **Porting from zim-downloader**: The `AnnotationPanel.tsx` in zim-downloader has working implementations for voice, ink, and text. Use as reference but adapt to new data model with articleUrl/archiveId.

2. **ZIM Fingerprinting**: The partial hashing (first+last MB) is for performance. Consider full-file hashing for smaller ZIMs (< 100MB) for stronger verification.

3. **Mobile vs Desktop**: Use `capture="environment"` for mobile camera, fallback to file picker on desktop. Check `navigator.userAgent` or use feature detection.

4. **Storage Quotas**: Browser storage is limited (typically 50-60% of available disk). Implement cleanup suggestions when approaching limits.

5. **Proprietary Format**: The `.enzim` extension and bundle structure are proprietary. Document internally but don't publish as open standard.

---

**Last Updated**: 2026-02-08  
**Author**: Architecture generated by Cascade  
**Status**: Ready for implementation
