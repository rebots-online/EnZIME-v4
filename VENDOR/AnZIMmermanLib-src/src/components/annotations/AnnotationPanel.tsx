// Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
// All rights reserved.
// Unauthorized use without prior written consent is strictly prohibited.

// EnZIM - Annotation Panel Component

import { useState } from 'react';
import { X, Mic, Pencil, StickyNote, Camera, Video, Trash2 } from 'lucide-react';
import { VoiceRecorder } from './VoiceRecorder';
import { InkCanvas } from './InkCanvas';
import { TextNoteEditor } from './TextNoteEditor';
import { PhotoCapture } from './PhotoCapture';
import { VideoCapture } from './VideoCapture';
import { ShareMenu } from './ShareMenu';
import { useAnnotationStore } from '../../stores/annotationStore';
import { useArchiveStore } from '../../stores/archiveStore';
import type { AnnotationType, AnnotationColor } from '../../types/annotation';

interface AnnotationPanelProps {
  articleUrl: string;
  archiveId: string;
  articleTitle?: string;
  onClose: () => void;
  initialPosition?: { x: number; y: number };
  initialType?: AnnotationType;
}

type TabType = 'text' | 'voice' | 'ink' | 'photo' | 'video';

const TABS: { id: TabType; label: string; icon: typeof StickyNote }[] = [
  { id: 'text', label: 'Note', icon: StickyNote },
  { id: 'voice', label: 'Voice', icon: Mic },
  { id: 'ink', label: 'Ink', icon: Pencil },
  { id: 'photo', label: 'Photo', icon: Camera },
  { id: 'video', label: 'Video', icon: Video },
];

export function AnnotationPanel({
  articleUrl,
  archiveId,
  articleTitle,
  onClose,
  initialPosition,
  initialType = 'text',
}: AnnotationPanelProps) {
  const [activeTab, setActiveTab] = useState<TabType>(initialType as TabType);
  const { addAnnotation, getAnnotationsForArticle, removeAnnotation } = useAnnotationStore();
  const archive = useArchiveStore((s) => s.archives.find((a) => a.id === archiveId));
  const archiveName = archive?.title || archive?.name || archiveId;

  const annotations = getAnnotationsForArticle(articleUrl, archiveId);

  const handleVoiceSave = (base64Audio: string) => {
    addAnnotation({
      type: 'voice',
      articleUrl,
      archiveId,
      content: base64Audio,
      position: initialPosition,
    });
  };

  const handleInkSave = (dataUrl: string) => {
    addAnnotation({
      type: 'ink',
      articleUrl,
      archiveId,
      content: dataUrl,
      position: initialPosition,
      color: 'orange',
    });
  };

  const handleTextSave = (text: string, color: AnnotationColor) => {
    addAnnotation({
      type: 'text',
      articleUrl,
      archiveId,
      content: text,
      position: initialPosition,
      color,
    });
  };

  const handlePhotoSave = (dataUrl: string) => {
    addAnnotation({
      type: 'photo',
      articleUrl,
      archiveId,
      content: dataUrl,
      position: initialPosition,
    });
  };

  const handleVideoSave = (dataUrl: string) => {
    addAnnotation({
      type: 'video',
      articleUrl,
      archiveId,
      content: dataUrl,
      position: initialPosition,
    });
  };

  const handleDeleteAnnotation = (id: string) => {
    removeAnnotation(id);
  };

  const getAnnotationIcon = (type: AnnotationType) => {
    switch (type) {
      case 'text': return <StickyNote className="w-4 h-4" />;
      case 'voice': return <Mic className="w-4 h-4" />;
      case 'ink': return <Pencil className="w-4 h-4" />;
      case 'photo': return <Camera className="w-4 h-4" />;
      case 'video': return <Video className="w-4 h-4" />;
      case 'highlight': return <StickyNote className="w-4 h-4" />;
      default: return <StickyNote className="w-4 h-4" />;
    }
  };

  return (
    <aside className="w-80 bg-[var(--bg-surface)] border-l border-[var(--border)] flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-[var(--border)]">
        <h2 className="font-semibold text-[var(--text-primary)]">Annotations</h2>
        <button
          onClick={onClose}
          className="p-1.5 hover:bg-[var(--border)] rounded-lg transition-colors"
          aria-label="Close panel"
        >
          <X className="w-5 h-5 text-secondary" />
        </button>
      </div>

      {/* Tabs */}
      <div className="flex border-b border-[var(--border)]">
        {TABS.map((tab) => {
          const Icon = tab.icon;
          return (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`flex-1 flex items-center justify-center gap-1.5 py-3 text-sm transition-colors ${
                activeTab === tab.id
                  ? 'bg-[var(--bg-primary)] text-[var(--accent)] border-b-2 border-[var(--accent)]'
                  : 'text-secondary hover:text-[var(--text-primary)]'
              }`}
            >
              <Icon className="w-4 h-4" />
              <span className="hidden sm:inline">{tab.label}</span>
            </button>
          );
        })}
      </div>

      {/* Tab Content */}
      <div className="flex-1 overflow-auto p-4">
        {activeTab === 'text' && (
          <TextNoteEditor onSave={handleTextSave} />
        )}

        {activeTab === 'voice' && (
          <VoiceRecorder onRecordingComplete={handleVoiceSave} />
        )}

        {activeTab === 'ink' && (
          <InkCanvas onSave={handleInkSave} />
        )}

        {activeTab === 'photo' && (
          <PhotoCapture onCapture={handlePhotoSave} />
        )}

        {activeTab === 'video' && (
          <VideoCapture onCapture={handleVideoSave} />
        )}
      </div>

      {/* Saved Annotations List */}
      <div className="border-t border-[var(--border)] p-4 max-h-64 overflow-auto">
        <h3 className="text-sm font-medium text-secondary mb-3">
          Saved ({annotations.length})
        </h3>
        
        {annotations.length === 0 ? (
          <p className="text-sm text-secondary text-center py-4">
            No annotations yet
          </p>
        ) : (
          <div className="space-y-2">
            {annotations.map((ann) => (
              <div
                key={ann.id}
                className="p-3 rounded-lg bg-[var(--bg-primary)] border border-[var(--border)] flex items-start gap-3"
              >
                <div className="text-[var(--accent)] mt-0.5">
                  {getAnnotationIcon(ann.type)}
                </div>
                
                <div className="flex-1 min-w-0">
                  {ann.type === 'text' && (
                    <p className="text-sm text-[var(--text-primary)] truncate">
                      {ann.content}
                    </p>
                  )}
                  
                  {ann.type === 'voice' && (
                    <audio controls className="w-full h-8">
                      <source src={ann.content} type="audio/webm" />
                    </audio>
                  )}
                  
                  {ann.type === 'ink' && (
                    <img 
                      src={ann.content} 
                      alt="Ink annotation" 
                      className="w-full h-16 object-contain bg-white rounded"
                    />
                  )}
                  
                  {ann.type === 'photo' && (
                    <img 
                      src={ann.content} 
                      alt="Photo annotation" 
                      className="w-full h-16 object-cover rounded"
                    />
                  )}
                  
                  {ann.type === 'video' && (
                    <video 
                      src={ann.content} 
                      className="w-full h-16 object-cover rounded"
                      preload="metadata"
                    />
                  )}
                  
                  <p className="text-xs text-secondary mt-1">
                    {new Date(ann.createdAt).toLocaleString()}
                  </p>
                </div>
                
                <div className="flex items-center gap-0.5 flex-shrink-0">
                  <ShareMenu
                    annotation={ann}
                    archiveName={archiveName}
                    articleTitle={articleTitle || articleUrl}
                  />
                  <button
                    onClick={() => handleDeleteAnnotation(ann.id)}
                    className="p-1.5 hover:bg-red-500/10 hover:text-red-500 rounded transition-colors"
                    aria-label="Delete annotation"
                  >
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </aside>
  );
}
