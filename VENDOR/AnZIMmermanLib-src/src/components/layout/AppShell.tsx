// Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
// All rights reserved.
// Unauthorized use without prior written consent is strictly prohibited.

// EnZIM - Offline ZIM Reader & Knowledge Explorer
// Copyright (C) 2025 Robin L. M. Cheung, MBA. All rights reserved.

import { useState, useCallback } from 'react';
import { Outlet, useNavigate } from 'react-router-dom';
import { zimService } from '../../services/zimService';
import { useArchiveStore } from '../../stores/archiveStore';
import { openExternalUrl } from '../../utils/platform';
import { Header } from './Header';
import { MenuBar } from './MenuBar';
import { Sidebar } from './Sidebar';
import { StatusBar } from './StatusBar';
import { AboutDialog } from '../dialogs/AboutDialog';

interface AppInfo {
  name: string;
  version: string;
  description: string;
  copyright: string;
  license: string;
}

interface AppShellProps {
  appInfo: AppInfo;
}

export function AppShell({ appInfo }: AppShellProps) {
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [aboutOpen, setAboutOpen] = useState(false);
  const navigate = useNavigate();
  const { addArchive } = useArchiveStore();

  const handleOpenFile = useCallback(async () => {
    try {
      const source = await zimService.openFileDialog();
      if (source) {
        const archiveInfo = await zimService.loadArchive(source);
        if (archiveInfo) {
          addArchive({
            id: archiveInfo.id,
            name: archiveInfo.name,
            title: archiveInfo.title,
            description: archiveInfo.description,
            language: archiveInfo.language,
            articleCount: archiveInfo.articleCount,
            mediaCount: 0,
            size: archiveInfo.size,
            path: archiveInfo.path,
            favicon: archiveInfo.favicon,
            addedAt: new Date(),
          });
        }
      }
    } catch (error) {
      console.error('Failed to open archive:', error);
    }
  }, [addArchive]);

  const handleDownload = useCallback(() => {
    openExternalUrl('https://library.kiwix.org/');
  }, []);

  const handleSettings = useCallback(() => {
    navigate('/settings');
  }, [navigate]);

  return (
    <div className="app-shell flex flex-col h-screen">
      {/* Menu Bar */}
      <div className="surface h-8 flex items-center border-b border-[var(--border)]">
        <MenuBar 
          onOpenFile={handleOpenFile}
          onDownloadArchives={handleDownload}
          onSettings={handleSettings}
          onShowAbout={() => setAboutOpen(true)}
          onToggleSidebar={() => setSidebarOpen(!sidebarOpen)}
        />
      </div>

      {/* Header */}
      <Header 
        appInfo={appInfo} 
        onMenuToggle={() => setSidebarOpen(!sidebarOpen)} 
      />
      
      {/* Main Layout */}
      <div className="flex flex-1 overflow-hidden">
        {/* Sidebar */}
        {sidebarOpen && (
          <Sidebar />
        )}
        
        {/* Main Content - Router Outlet */}
        <main className="flex-1 overflow-auto bg-[var(--bg-primary)]">
          <Outlet />
        </main>
      </div>
      
      {/* Status Bar */}
      <StatusBar version={appInfo.version} />

      {/* About Dialog */}
      <AboutDialog 
        appInfo={appInfo}
        isOpen={aboutOpen}
        onClose={() => setAboutOpen(false)}
      />
    </div>
  );
}
