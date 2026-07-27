// Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
// All rights reserved.
// Unauthorized use without prior written consent is strictly prohibited.

// EnZIM - Photo Capture Component

import { useCallback, useState } from 'react';
import { Camera, Trash2, Save } from 'lucide-react';

interface PhotoCaptureProps {
  onCapture: (dataUrl: string) => void;
  onCancel?: () => void;
}

const MAX_WIDTH = 1024;
const MAX_HEIGHT = 1024;
const JPEG_QUALITY = 0.8;

export function PhotoCapture({ onCapture, onCancel }: PhotoCaptureProps) {
  const [previewUrl, setPreviewUrl] = useState<string | null>(null);
  const [isProcessing, setIsProcessing] = useState(false);

  const compressImage = useCallback((dataUrl: string): Promise<string> => {
    return new Promise((resolve, reject) => {
      const img = new window.Image();
      img.onload = () => {
        let { width, height } = img;

        // Calculate scaled dimensions
        if (width > MAX_WIDTH || height > MAX_HEIGHT) {
          const ratio = Math.min(MAX_WIDTH / width, MAX_HEIGHT / height);
          width *= ratio;
          height *= ratio;
        }

        const canvas = document.createElement('canvas');
        canvas.width = width;
        canvas.height = height;
        const ctx = canvas.getContext('2d');
        if (!ctx) {
          reject(new Error('Could not get canvas context'));
          return;
        }

        ctx.drawImage(img, 0, 0, width, height);
        resolve(canvas.toDataURL('image/jpeg', JPEG_QUALITY));
      };
      img.onerror = () => reject(new Error('Failed to load image'));
      img.src = dataUrl;
    });
  }, []);

  const handleFileSelect = useCallback(async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    setIsProcessing(true);
    try {
      const reader = new FileReader();
      reader.onloadend = async () => {
        const dataUrl = reader.result as string;
        const compressed = await compressImage(dataUrl);
        setPreviewUrl(compressed);
        setIsProcessing(false);
      };
      reader.readAsDataURL(file);
    } catch (error) {
      console.error('Failed to process image:', error);
      alert('Failed to process image. Please try again.');
      setIsProcessing(false);
    }
  }, [compressImage]);

  const handleSave = useCallback(() => {
    if (previewUrl) {
      onCapture(previewUrl);
      setPreviewUrl(null);
    }
  }, [previewUrl, onCapture]);

  const handleDiscard = useCallback(() => {
    setPreviewUrl(null);
    onCancel?.();
  }, [onCancel]);

  return (
    <div className="flex flex-col items-center gap-4">
      {!previewUrl ? (
        <>
          <label className="w-24 h-24 rounded-full bg-[var(--accent)] hover:opacity-90 flex items-center justify-center cursor-pointer transition-all">
            <Camera className="w-10 h-10 text-white" />
            <input
              type="file"
              accept="image/*"
              capture="environment"
              onChange={handleFileSelect}
              className="hidden"
            />
          </label>
          <p className="text-sm text-secondary text-center">
            Tap to take a photo<br />
            <span className="text-xs opacity-70">or select from gallery</span>
          </p>
          
          {/* Desktop fallback - hidden on mobile */}
          <label className="hidden md:block">
            <span className="px-4 py-2 bg-[var(--bg-surface)] hover:bg-[var(--border)] border border-[var(--border)] rounded-lg cursor-pointer transition-colors text-sm">
              Choose from files
            </span>
            <input
              type="file"
              accept="image/*"
              onChange={handleFileSelect}
              className="hidden"
            />
          </label>
        </>
      ) : (
        <>
          <div className="relative max-w-xs">
            <img 
              src={previewUrl} 
              alt="Captured" 
              className="w-full rounded-lg border border-[var(--border)]"
            />
            {isProcessing && (
              <div className="absolute inset-0 bg-black/50 flex items-center justify-center rounded-lg">
                <div className="animate-spin w-8 h-8 border-2 border-white border-t-transparent rounded-full" />
              </div>
            )}
          </div>

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
              disabled={isProcessing}
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
