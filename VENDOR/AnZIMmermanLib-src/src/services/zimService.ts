// Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
// All rights reserved.
// Unauthorized use without prior written consent is strictly prohibited.

// EnZIM - Offline ZIM Reader & Knowledge Explorer
// Copyright (C) 2025 Robin L. M. Cheung, MBA. All rights reserved.

import { isTauri } from '../utils/platform';

// ZIM format constants per https://wiki.openzim.org/wiki/ZIM_file_format
const ZIM_MAGIC_NUMBER = 0x044D495A; // 72173914 decimal — "ZIM\x04" in little-endian
// ZIM special mimetype constants (defined in zimlib.ts, kept here as reference):
// REDIRECT = 0xFFFF, LINKTARGET = 0xFFFE, DELETED = 0xFFFD

export interface ZIMHeader {
  magicNumber: number;
  majorVersion: number;
  minorVersion: number;
  uuid: Uint8Array;       // 16-byte archive UUID (offset 8)
  entryCount: number;     // offset 24
  clusterCount: number;   // offset 28
  pathPtrPos: bigint;     // offset 32 — directory pointers ordered by path
  titlePtrPos: bigint;    // offset 40 — directory pointers ordered by title
  clusterPtrPos: bigint;  // offset 48
  mimeListPos: bigint;    // offset 56 — also marks header size
  mainPage: number;       // offset 64 — 0xFFFFFFFF if none
  layoutPage: number;     // offset 68 — deprecated, always 0xFFFFFFFF
  checksumPos: bigint;    // offset 72
}

export interface ZIMEntry {
  mimetypeIndex: number;
  namespace: number;
  url: string;
  title: string;
  isRedirect: boolean;
  clusterNumber?: number;
  blobNumber?: number;
  redirectIndex?: number;
}

export interface ZIMArchiveInfo {
  id: string;
  name: string;
  title: string;
  description: string;
  language: string;
  articleCount: number;
  size: number;
  path: string;
  favicon?: string;
}

class BinaryReader {
  private dataView: DataView;
  private position: number = 0;

  constructor(buffer: ArrayBuffer) {
    this.dataView = new DataView(buffer);
  }

  readUInt32LE(): number {
    const value = this.dataView.getUint32(this.position, true);
    this.position += 4;
    return value;
  }

  readUInt16LE(): number {
    const value = this.dataView.getUint16(this.position, true);
    this.position += 2;
    return value;
  }

  readUInt8(): number {
    const value = this.dataView.getUint8(this.position);
    this.position += 1;
    return value;
  }

  readUInt64LE(): bigint {
    const value = this.dataView.getBigUint64(this.position, true);
    this.position += 8;
    return value;
  }

  readNullTerminatedString(): string {
    const bytes: number[] = [];
    while (this.position < this.dataView.byteLength) {
      const byte = this.dataView.getUint8(this.position);
      this.position++;
      if (byte === 0) break;
      bytes.push(byte);
    }
    return new TextDecoder().decode(new Uint8Array(bytes));
  }

  seek(position: number): void {
    this.position = position;
  }

  getPosition(): number {
    return this.position;
  }

  readBytes(length: number): Uint8Array {
    const bytes = new Uint8Array(this.dataView.buffer, this.position, length);
    this.position += length;
    return bytes;
  }
}

export class ZIMService {
  private static instance: ZIMService;
  private openArchives: Map<string, { buffer: ArrayBuffer; header: ZIMHeader; entries: ZIMEntry[] }> = new Map();

  static getInstance(): ZIMService {
    if (!ZIMService.instance) {
      ZIMService.instance = new ZIMService();
    }
    return ZIMService.instance;
  }

  async openFileDialog(): Promise<string | File | null> {
    if (isTauri()) {
      try {
        const { open } = await import('@tauri-apps/plugin-dialog');
        const selected = await open({
          multiple: false,
          filters: [{
            name: 'ZIM Archives',
            extensions: ['zim']
          }]
        });
        
        if (selected && typeof selected === 'string') {
          return selected;
        }
        return null;
      } catch (error) {
        console.error('Failed to open file dialog:', error);
        return null;
      }
    }

    // Browser fallback: use a hidden <input type="file">
    return new Promise<File | null>((resolve) => {
      const input = document.createElement('input');
      input.type = 'file';
      input.accept = '.zim';
      input.style.display = 'none';
      input.addEventListener('change', () => {
        const file = input.files?.[0] ?? null;
        document.body.removeChild(input);
        resolve(file);
      });
      input.addEventListener('cancel', () => {
        document.body.removeChild(input);
        resolve(null);
      });
      document.body.appendChild(input);
      input.click();
    });
  }

  async loadArchive(source: string | File): Promise<ZIMArchiveInfo | null> {
    try {
      let buffer: ArrayBuffer;
      let fileSize: number;
      let fileName: string;
      let filePath: string;

      if (typeof source === 'string') {
        // Tauri path-based loading
        const { readFile, stat } = await import('@tauri-apps/plugin-fs');
        const fileInfo = await stat(source);
        const fileData = await readFile(source);
        buffer = fileData.buffer as ArrayBuffer;
        fileSize = fileInfo.size;
        fileName = source.split('/').pop() || source.split('\\').pop() || 'Unknown';
        filePath = source;
      } else {
        // Browser File-based loading
        buffer = await source.arrayBuffer();
        fileSize = source.size;
        fileName = source.name;
        filePath = source.name;
      }

      const reader = new BinaryReader(buffer);
      const header = this.readHeader(reader);

      // CORRECTED: Check for correct ZIM magic number
      if (header.magicNumber !== ZIM_MAGIC_NUMBER) {
        throw new Error(`Invalid ZIM file format: expected magic number 0x${ZIM_MAGIC_NUMBER.toString(16).toUpperCase()}, got 0x${header.magicNumber.toString(16).toUpperCase()}`);
      }

      const mimeTypes = this.readMimeTypes(reader, buffer, header);
      const entries = this.readDirectoryEntries(reader, buffer, header);

      const archiveId = this.generateArchiveId(filePath);
      this.openArchives.set(archiveId, { buffer, header, entries });

      const metadata = this.extractMetadata(entries, mimeTypes);

      return {
        id: archiveId,
        name: fileName,
        title: metadata.title || fileName.replace('.zim', ''),
        description: metadata.description || '',
        language: metadata.language || 'en',
        articleCount: header.entryCount,
        size: fileSize,
        path: filePath,
        favicon: metadata.favicon,
      };
    } catch (error) {
      console.error('Failed to load ZIM archive:', error);
      return null;
    }
  }

  private generateArchiveId(_filePath: string): string {
    return `zim-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  private readHeader(reader: BinaryReader): ZIMHeader {
    const magicNumber = reader.readUInt32LE();       // offset 0
    const majorVersion = reader.readUInt16LE();       // offset 4
    const minorVersion = reader.readUInt16LE();       // offset 6
    const uuid = reader.readBytes(16);                // offset 8  (16 bytes)
    const entryCount = reader.readUInt32LE();          // offset 24
    const clusterCount = reader.readUInt32LE();        // offset 28
    const pathPtrPos = reader.readUInt64LE();           // offset 32
    const titlePtrPos = reader.readUInt64LE();          // offset 40
    const clusterPtrPos = reader.readUInt64LE();        // offset 48
    const mimeListPos = reader.readUInt64LE();          // offset 56
    const mainPage = reader.readUInt32LE();             // offset 64
    const layoutPage = reader.readUInt32LE();           // offset 68
    const checksumPos = reader.readUInt64LE();          // offset 72
    // reader is now at position 80 (end of header)

    return {
      magicNumber,
      majorVersion,
      minorVersion,
      uuid,
      entryCount,
      clusterCount,
      pathPtrPos,
      titlePtrPos,
      clusterPtrPos,
      mimeListPos,
      mainPage,
      layoutPage,
      checksumPos,
    };
  }

  private readMimeTypes(reader: BinaryReader, buffer: ArrayBuffer, header: ZIMHeader): string[] {
    const mimeTypePos = Number(header.mimeListPos);
    reader.seek(mimeTypePos);
    const mimeTypes: string[] = [];
    let prevWasNull = false;

    while (reader.getPosition() < buffer.byteLength) {
      const str = reader.readNullTerminatedString();
      if (str.length === 0) {
        if (prevWasNull) break;
        prevWasNull = true;
      } else {
        mimeTypes.push(str);
        prevWasNull = false;
      }
    }

    return mimeTypes;
  }

  private readDirectoryEntries(reader: BinaryReader, _buffer: ArrayBuffer, header: ZIMHeader): ZIMEntry[] {
    const entries: ZIMEntry[] = [];
    
    // Read path pointer list from header-specified position
    const pathPtrPos = Number(header.pathPtrPos);
    reader.seek(pathPtrPos);
    const indexPointers: bigint[] = [];
    
    for (let i = 0; i < Math.min(header.entryCount, 10000); i++) { // Limit for performance
      indexPointers.push(reader.readUInt64LE());
    }

    for (const ptr of indexPointers) {
      try {
        reader.seek(Number(ptr));
        // Per ZIM spec: mimetype is uint16 (2 bytes), not uint32
        const mimetypeIndex = reader.readUInt16LE();
        
        reader.seek(Number(ptr)); // Reset
        
        if (mimetypeIndex === 0xFFFF) {
          // Redirect entry
          reader.readUInt16LE(); // mimetype (2 bytes)
          reader.readUInt8();    // parameterLen (1 byte, unused)
          const namespace = reader.readUInt8(); // namespace (1 byte)
          reader.readUInt32LE(); // revision
          const redirectIndex = reader.readUInt32LE();
          const url = reader.readNullTerminatedString();
          const title = reader.readNullTerminatedString();
          
          entries.push({
            mimetypeIndex,
            namespace,
            url,
            title: title || url,
            isRedirect: true,
            redirectIndex,
          });
        } else {
          // Content entry
          reader.readUInt16LE(); // mimetype (2 bytes)
          reader.readUInt8();    // parameterLen (1 byte, unused)
          const namespace = reader.readUInt8(); // namespace (1 byte)
          reader.readUInt32LE(); // revision
          const clusterNumber = reader.readUInt32LE();
          const blobNumber = reader.readUInt32LE();
          const url = reader.readNullTerminatedString();
          const title = reader.readNullTerminatedString();
          
          entries.push({
            mimetypeIndex,
            namespace,
            url,
            title: title || url,
            isRedirect: false,
            clusterNumber,
            blobNumber,
          });
        }
      } catch {
        // Skip malformed entries
        continue;
      }
    }

    return entries;
  }

  private extractMetadata(entries: ZIMEntry[], _mimeTypes: string[]): { title?: string; description?: string; language?: string; favicon?: string } {
    const metadata: { title?: string; description?: string; language?: string; favicon?: string } = {};
    
    const metadataNamespace = 'M'.charCodeAt(0);
    const metadataEntries = entries.filter(e => e.namespace === metadataNamespace);

    for (const entry of metadataEntries) {
      if (entry.url === 'Title') {
        metadata.title = entry.title;
      } else if (entry.url === 'Description') {
        metadata.description = entry.title;
      } else if (entry.url === 'Language') {
        metadata.language = entry.title;
      }
    }

    return metadata;
  }

  getArticles(archiveId: string): ZIMEntry[] {
    const archive = this.openArchives.get(archiveId);
    if (!archive) return [];

    // ZIMv5 uses namespace 'A' for articles, ZIMv6 uses 'C' for content
    const nsA = 'A'.charCodeAt(0);
    const nsC = 'C'.charCodeAt(0);
    return archive.entries.filter(e => (e.namespace === nsA || e.namespace === nsC) && !e.isRedirect);
  }

  searchArticles(archiveId: string, query: string, limit: number = 50): ZIMEntry[] {
    const articles = this.getArticles(archiveId);
    const lowerQuery = query.toLowerCase();
    
    return articles
      .filter(a => 
        a.title.toLowerCase().includes(lowerQuery) ||
        a.url.toLowerCase().includes(lowerQuery)
      )
      .slice(0, limit);
  }

  async getArticleContent(archiveId: string, entry: ZIMEntry): Promise<string | null> {
    const archive = this.openArchives.get(archiveId);
    if (!archive || entry.isRedirect || entry.clusterNumber === undefined || entry.blobNumber === undefined) {
      return null;
    }

    try {
      const reader = new BinaryReader(archive.buffer);
      
      // Use clusterPtrPos from stored header
      const clusterPtrPos = Number(archive.header.clusterPtrPos);
      
      // Read cluster offset
      reader.seek(clusterPtrPos + entry.clusterNumber * 8);
      const clusterOffset = Number(reader.readUInt64LE());
      
      // Read cluster data
      reader.seek(clusterOffset);
      const compressionByte = reader.readUInt8();
      
      // Read blob offsets
      const blobOffsets: number[] = [];
      let prevOffset = 0;
      for (let i = 0; i <= entry.blobNumber + 1; i++) {
        const offset = reader.readUInt32LE();
        if (offset === 0 && i > 0) break;
        blobOffsets.push(offset);
        if (offset < prevOffset) break; // Safety check
        prevOffset = offset;
      }
      
      if (entry.blobNumber >= blobOffsets.length - 1) {
        return null;
      }
      
      const blobStart = blobOffsets[entry.blobNumber];
      const blobEnd = blobOffsets[entry.blobNumber + 1];
      const blobSize = blobEnd - blobStart;
      
      // Read blob data
      const dataStartPos = reader.getPosition();
      reader.seek(dataStartPos + blobStart - (blobOffsets.length * 4));
      const blobData = reader.readBytes(blobSize);
      
      // Handle decompression if needed (basic support for uncompressed)
      if (compressionByte === 1 || compressionByte === 0) {
        // Uncompressed
        return new TextDecoder().decode(blobData);
      } else {
        // Compressed - would need decompression library
        console.warn('Compressed content not yet supported, compression type:', compressionByte);
        return null;
      }
    } catch (error) {
      console.error('Failed to read article content:', error);
      return null;
    }
  }

  getEntryByUrl(archiveId: string, url: string): ZIMEntry | undefined {
    const archive = this.openArchives.get(archiveId);
    if (!archive) return undefined;
    
    return archive.entries.find(e => e.url === url);
  }

  closeArchive(archiveId: string): void {
    this.openArchives.delete(archiveId);
  }
}

export const zimService = ZIMService.getInstance();
