# EnZIM Project Overview

## Purpose
EnZIM is an offline-first knowledge companion that transforms static ZIM archives (like Wikipedia) into an interactive, AI-powered exploration experience.

## Tech Stack
- **Frontend**: React + TypeScript + TailwindCSS
- **Desktop Framework**: Tauri (Rust + WebView)
- **State Management**: Zustand
- **Build Tool**: Vite
- **Testing**: Vitest

## Key Features
- Offline ZIM archive reader
- Library management for multiple ZIM files
- Full-text search
- Semantic mesh (knowledge graph)
- Bookmarks & history
- Annotations (text, voice, ink, photo, video, highlight)
- AI Assistant
- Text-to-Speech

## Architecture
```
Frontend Layer: React + TypeScript + Tailwind
State Layer: Zustand
Service Layer: ZIM, Search, Semantic, AI, TTS
Core Layer: AnZimmermanLib (Clean-Room ZIM)
```

## Key Files
- `AnZimmermanLIB/AnZimmermanLib/zimlib.ts` - Core ZIM library
- `src/services/zimService.ts` - Application ZIM service
- `src/stores/archiveStore.ts` - Archive state management
- `src/components/reader/ReaderView.tsx` - Article reader

## Code Style
- TypeScript strict mode
- Functional React components with hooks
- Zustand for state management
- Lucide-react for icons

## Important Commands
- `npm run dev` - Start dev server
- `npm run build` - Build project
- `npm run test` - Run tests
- `npm run lint` - Run ESLint
- `npm run tauri dev` - Start Tauri dev
- `npm run tauri build` - Build Tauri app