// Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
// All rights reserved.
// Unauthorized use without prior written consent is strictly prohibited.

// EnZIM - Video Capture Component

import { useState, useCallback } from 'react';
import { Video, Trash2, Save, AlertCircle } from 'lucide-react';

interface VideoCaptureProps {
  onCapture: (dataUrl: string) => void;
  onCancel?: () => void;
}

const MAX_FILE_SIZE_MB = 10;
const MAX_FILE_SIZE_BYTES = MAX_FILE_SIZE_MB * 1024 * 1024;

export function VideoCapture({ onCapture, onCancel }: VideoCaptureProps) {
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [previewUrl, setPreviewUrl] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isConverting, setIsConverting] = useState(false);

  const handleFileSelect = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    setError(null);

    // Check file size
    if (file.size > MAX_FILE_SIZE_BYTES) {
      setError(`Video must be under ${MAX_FILE_SIZE_MB}MB`);
      return;
    }

    setSelectedFile(file);
    setIsConverting(true);

    // Convert to data URL
    const reader = new FileReader();
    reader.onloadend = () => {
      setPreviewUrl(reader.result as string);
      setIsConverting(false);
    };
    reader.onerror = () => {
      setError('Failed to read video file');
      setIsConverting(false);
    };
    reader.readAsDataURL(file);
  }, []);

  const handleSave = useCallback(() => {
    if (previewUrl) {
      onCapture(previewUrl);
      setSelectedFile(null);
      setPreviewUrl(null);
      setError(null);
    }
  }, [previewUrl, onCapture]);

  const handleDiscard = useCallback(() => {
    setSelectedFile(null);
    setPreviewUrl(null);
    setError(null);
    onCancel?.();
  }, [onCancel]);

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
  };

  return (
    <div className="flex flex-col items-center gap-4">
      {!previewUrl ? (
        <>
          <label className="w-24 h-24 rounded-full bg-[var(--accent)] hover:opacity-90 flex items-center justify-center cursor-pointer transition-all">
            <Video className="w-10 h-10 text-white" />
            <input
              type="file"
              accept="video/*"
              capture="environment"
              onChange={handleFileSelect}
              className="hidden"
            />
          </label>
          <p className="text-sm text-secondary text-center">
            Tap to record video<br />
            <span className="text-xs opacity-70">Max {MAX_FILE_SIZE_MB}MB</span>
          </p>

          {/* Desktop fallback */}
          <label className="hidden md:block">
            <span className="px-4 py-2 bg-[var(--bg-surface)] hover:bg-[var(--border)] border border-[var(--border)] rounded-lg cursor-pointer transition-colors text-sm">
              Choose video file
            </span>
            <input
              type="file"
              accept="video/*"
              onChange={handleFileSelect}
              className="hidden"
            />
          </label>

          {error && (
            <div className="flex items-center gap-2 text-red-500 text-sm">
              <AlertCircle className="w-4 h-4" />
              {error}
            </div>
          )}
        </>
      ) : (
        <>
          <div className="relative max-w-xs w-full">
            <video
              src={previewUrl}
              controls
              className="w-full rounded-lg border border-[var(--border)]"
            />
            {isConverting && (
              <div className="absolute inset-0 bg-black/50 flex items-center justify-center rounded-lg">
                <div className="animate-spin w-8 h-8 border-2 border-white border-t-transparent rounded-full" />
              </div>
            )}
          </div>

          {selectedFile && (
            <p className="text-xs text-secondary">
              Size: {formatFileSize(selectedFile.size)}
            </p>
          )}

          <div className="flex gap-2 w-full max-w-xs">
            <button
              onClick={handleDiscard}
              className="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-[var(--bg-surface)] hover:bg-[var(--border)] border border-[var(--border)] rounded-lg transition-colors"
            >
              <Trash2 className="w-4 h-4" />
              Retake
            </button>
            <button
              onClick={handleSave}
              disabled={isConverting}
              className="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-[var(--accent)] text-white hover:opacity-90 disabled:opacity-50 rounded-lg transition-colors"
            >
              <Save className="w-4 h-4" />
              Save
            </button>
          </div>
        </>
      )}
    </div>
  );
}
