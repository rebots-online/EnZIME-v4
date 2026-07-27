// Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
// All rights reserved.
// Unauthorized use without prior written consent is strictly prohibited.

// EnZIM - Offline ZIM Reader & Knowledge Explorer
// Copyright (C) 2025 Robin L. M. Cheung, MBA. All rights reserved.

import { createHashRouter } from 'react-router-dom';
import { LibraryView } from './components/library/LibraryView';
import { ReaderView } from './components/reader/ReaderView';
import { SearchView } from './components/search/SearchView';
import { HistoryView } from './components/views/HistoryView';
import { BookmarksView } from './components/views/BookmarksView';
import { SettingsView } from './components/views/SettingsView';
import { AnnotationBrowser } from './components/annotations/AnnotationBrowser';
import { AppShell } from './components/layout/AppShell';

interface AppInfo {
  name: string;
  version: string;
  description: string;
  copyright: string;
  license: string;
}

export function createRouter(appInfo: AppInfo) {
  return createHashRouter([
    {
      path: '/',
      element: <AppShell appInfo={appInfo} />,
      children: [
        {
          index: true,
          element: <LibraryView />,
        },
        {
          path: 'library',
          element: <LibraryView />,
        },
        {
          path: 'reader',
          element: <ReaderView />,
        },
        {
          path: 'reader/:archiveId',
          element: <ReaderView />,
        },
        {
          path: 'reader/:archiveId/*',
          element: <ReaderView />,
        },
        {
          path: 'search',
          element: <SearchView />,
        },
        {
          path: 'history',
          element: <HistoryView />,
        },
        {
          path: 'bookmarks',
          element: <BookmarksView />,
        },
        {
          path: 'annotations',
          element: <AnnotationBrowser />,
        },
        {
          path: 'annotations/:archiveId',
          element: <AnnotationBrowser />,
        },
        {
          path: 'settings',
          element: <SettingsView />,
        },
      ],
    },
  ]);
}
