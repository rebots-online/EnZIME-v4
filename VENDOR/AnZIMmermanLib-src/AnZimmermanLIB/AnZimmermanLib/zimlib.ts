// Copyright (C) 2025–2026 Robin L. M. Cheung, MBA
// All rights reserved.
// Unauthorized use without prior written consent is strictly prohibited.

/**
 * Clean-room ZIM (Zeno IMproved) file format reader/writer library for TypeScript/Node.js
 * 
 * Based on ZIM file format specification from openZIM
 * Supports reading and writing ZIM archives for offline content storage
 * 
 * Copyright (C) 2025 Robin L. M. Cheung, MBA. All rights reserved.
 */

import { readFileSync, writeFileSync, openSync, closeSync, readSync, writeSync, fstatSync } from 'fs';
import { createGunzip, createInflate, gunzipSync, inflateSync } from 'zlib';
import { promisify } from 'util';

/**
 * ZIM file format constants and magic numbers
 * @version 6.0 (ZIM format specification)
 */
export const ZIM_MAGIC_NUMBER = 0x044D495A; // 72173914 decimal — ZIM file format magic
export const ZIM_VERSION_MAJOR = 6;  // Current ZIM format major version
export const ZIM_VERSION_MINOR = 0;  // Current ZIM format minor version

export const ZIM_REDIRECT_MIMETYPE = 0xFFFF; // Special mimetype index for redirect entries
export const ZIM_LINKTARGEST_MIMETYPE = 0xFFFE; // Special mimetype for link target entries
export const ZIM_DELETED_MIMETYPE = 0xFFFD; // Special mimetype for deleted entries

/** Header size in bytes (80 bytes) */
export const ZIM_HEADER_SIZE = 80;

/** Minimum size for a valid ZIM file */
export const ZIM_MIN_FILE_SIZE = 80;

/**
 * Compression types used in ZIM clusters
 * @see https://openzim.org/wiki/ZIM_file_format#Clusters
 */
export enum CompressionType {
    /** Default compression (same as NONE in version 5, deprecated in version 6) */
    DEFAULT = 0,
    /** No compression - stored as-is */
    NONE = 1,
    /** ZLIB compression (deflate) */
    ZLIB = 2,
    /** BZIP2 compression (deprecated) */
    BZIP2 = 3,
    /** LZMA compression (lzma2/xz) - commonly used in modern ZIM files */
    LZMA = 4,
    /** Zstandard compression - fastest, commonly used in modern ZIM files */
    ZSTD = 5
}

export enum Namespace {
    MAIN_ARTICLE = 'A'.charCodeAt(0),
    IMAGE = 'I'.charCodeAt(0),
    METADATA = 'M'.charCodeAt(0),
    RAW_DATA = '-'.charCodeAt(0),
    STYLE = 'S'.charCodeAt(0),
    SCRIPT = 'J'.charCodeAt(0),
    FONT = 'T'.charCodeAt(0),
    TRANSLATION = 'U'.charCodeAt(0),
    VIDEO = 'V'.charCodeAt(0),
    AUDIO = 'W'.charCodeAt(0)
}

export interface ZIMHeader {
    magicNumber: number;
    majorVersion: number;
    minorVersion: number;
    uuid: Buffer;            // 16-byte archive UUID (offset 8)
    entryCount: number;      // offset 24
    clusterCount: number;    // offset 28
    pathPtrPos: bigint;      // offset 32 — directory pointers ordered by path
    titlePtrPos: bigint;     // offset 40 — directory pointers ordered by title
    clusterPtrPos: bigint;   // offset 48
    mimeListPos: bigint;     // offset 56 — also marks header size
    mainPage: number;        // offset 64 — 0xFFFFFFFF if none
    layoutPage: number;      // offset 68 — deprecated, always 0xFFFFFFFF
    checksumPos: bigint;     // offset 72
}

export interface DirectoryEntry {
    mimetypeIndex: number;
    namespace: number;
    revision: number;
    clusterNumber: number;
    blobNumber: number;
    url: string;
    title: string;
}

export interface RedirectEntry {
    mimetypeIndex: number;
    namespace: number;
    revision: number;
    redirectIndex: number;
    url: string;
    title: string;
}

export type Entry = DirectoryEntry | RedirectEntry;

/**
 * Binary reader utility for ZIM files
 */
class BinaryReader {
    private buffer: Buffer;
    private position: number = 0;

    constructor(buffer: Buffer) {
        this.buffer = buffer;
    }

    readUInt32LE(): number {
        const value = this.buffer.readUInt32LE(this.position);
        this.position += 4;
        return value;
    }

    readUInt16LE(): number {
        const value = this.buffer.readUInt16LE(this.position);
        this.position += 2;
        return value;
    }

    readUInt8(): number {
        const value = this.buffer.readUInt8(this.position);
        this.position += 1;
        return value;
    }

    readUInt64LE(): bigint {
        const value = this.buffer.readBigUInt64LE(this.position);
        this.position += 8;
        return value;
    }

    readNullTerminatedString(): string {
        const start = this.position;
        while (this.position < this.buffer.length && this.buffer[this.position] !== 0) {
            this.position++;
        }
        const str = this.buffer.toString('utf8', start, this.position);
        this.position++; // Skip null terminator
        return str;
    }

    seek(position: number): void {
        this.position = position;
    }

    getPosition(): number {
        return this.position;
    }

    readBytes(length: number): Buffer {
        const bytes = this.buffer.subarray(this.position, this.position + length);
        this.position += length;
        return bytes;
    }
}

/**
 * Binary writer utility for ZIM files
 */
class BinaryWriter {
    private buffer: Buffer;
    private position: number = 0;

    constructor(initialSize: number = 1024) {
        this.buffer = Buffer.alloc(initialSize);
    }

    private ensureCapacity(additionalBytes: number): void {
        if (this.position + additionalBytes > this.buffer.length) {
            const newSize = Math.max(this.buffer.length * 2, this.position + additionalBytes);
            const newBuffer = Buffer.alloc(newSize);
            this.buffer.copy(newBuffer);
            this.buffer = newBuffer;
        }
    }

    writeUInt32LE(value: number): void {
        this.ensureCapacity(4);
        this.buffer.writeUInt32LE(value, this.position);
        this.position += 4;
    }

    writeUInt16LE(value: number): void {
        this.ensureCapacity(2);
        this.buffer.writeUInt16LE(value, this.position);
        this.position += 2;
    }

    writeUInt8(value: number): void {
        this.ensureCapacity(1);
        this.buffer.writeUInt8(value, this.position);
        this.position += 1;
    }

    writeUInt64LE(value: bigint): void {
        this.ensureCapacity(8);
        this.buffer.writeBigUInt64LE(value, this.position);
        this.position += 8;
    }

    writeNullTerminatedString(str: string): void {
        const strBytes = Buffer.from(str, 'utf8');
        this.ensureCapacity(strBytes.length + 1);
        strBytes.copy(this.buffer, this.position);
        this.position += strBytes.length;
        this.buffer[this.position] = 0; // Null terminator
        this.position += 1;
    }

    writeBytes(bytes: Buffer): void {
        this.ensureCapacity(bytes.length);
        bytes.copy(this.buffer, this.position);
        this.position += bytes.length;
    }

    seek(position: number): void {
        this.position = position;
    }

    getPosition(): number {
        return this.position;
    }

    getBuffer(): Buffer {
        return this.buffer.subarray(0, this.position);
    }
}

/**
 * ZIM file reader class
 */
export class ZIMReader {
    private filePath: string;
    private fileHandle?: number;
    private fileBuffer?: Buffer;
    private header?: ZIMHeader;
    private mimeTypes: string[] = [];
    private directoryEntries: Entry[] = [];
    private clusterOffsets: bigint[] = [];

    constructor(filePath: string) {
        this.filePath = filePath;
    }

    open(): void {
        this.fileBuffer = readFileSync(this.filePath);
        this.readHeader();
        this.readMimeTypes();
        this.readDirectory();
        this.readClusterPointers();
    }

    close(): void {
        if (this.fileHandle !== undefined) {
            closeSync(this.fileHandle);
            this.fileHandle = undefined;
        }
        this.fileBuffer = undefined;
    }

    private readHeader(): void {
        if (!this.fileBuffer) throw new Error('File not opened');

        const reader = new BinaryReader(this.fileBuffer);
        this.header = {
            magicNumber: reader.readUInt32LE(),       // offset 0
            majorVersion: reader.readUInt16LE(),       // offset 4
            minorVersion: reader.readUInt16LE(),       // offset 6
            uuid: reader.readBytes(16),                // offset 8  (16 bytes)
            entryCount: reader.readUInt32LE(),          // offset 24
            clusterCount: reader.readUInt32LE(),        // offset 28
            pathPtrPos: reader.readUInt64LE(),           // offset 32
            titlePtrPos: reader.readUInt64LE(),          // offset 40
            clusterPtrPos: reader.readUInt64LE(),        // offset 48
            mimeListPos: reader.readUInt64LE(),          // offset 56
            mainPage: reader.readUInt32LE(),             // offset 64
            layoutPage: reader.readUInt32LE(),           // offset 68
            checksumPos: reader.readUInt64LE()           // offset 72
        };
        // reader is now at position 80 (end of header)

        if (this.header.magicNumber !== ZIM_MAGIC_NUMBER) {
            throw new Error(`Invalid ZIM file format: expected magic 0x${ZIM_MAGIC_NUMBER.toString(16).toUpperCase()} (${ZIM_MAGIC_NUMBER}), got 0x${this.header.magicNumber.toString(16).toUpperCase()} (${this.header.magicNumber})`);
        }
    }

    private readMimeTypes(): void {
        if (!this.header || !this.fileBuffer) throw new Error('Header not parsed or file not opened');

        const startPos = Number(this.header.mimeListPos);
        const reader = new BinaryReader(this.fileBuffer.subarray(startPos));
        
        const mimeTypesData: Buffer[] = [];
        let currentChunk = Buffer.alloc(0);
        
        // Read until double null terminator
        while (true) {
            const chunk = reader.readBytes(1024);
            if (chunk.length === 0) break;
            
            currentChunk = Buffer.concat([currentChunk, chunk]);
            
            // Check for double null terminator
            const nullNullPos = currentChunk.indexOf(Buffer.from([0, 0]));
            if (nullNullPos !== -1) {
                mimeTypesData.push(currentChunk.subarray(0, nullNullPos));
                break;
            }
        }

        // Split by null terminator and filter empty strings
        const mimeTypesString = mimeTypesData[0]?.toString('utf8') || '';
        this.mimeTypes = mimeTypesString.split('\x00').filter(mt => mt.length > 0);
    }

    private readDirectory(): void {
        if (!this.header || !this.fileBuffer) throw new Error('Header not parsed or file not opened');

        // Read path pointer list from header-specified position
        const pathPtrPos = Number(this.header.pathPtrPos);
        const urlPtrReader = new BinaryReader(this.fileBuffer.subarray(pathPtrPos));
        const indexPointers: bigint[] = [];
        for (let i = 0; i < this.header.entryCount; i++) {
            indexPointers.push(urlPtrReader.readUInt64LE());
        }

        // Read directory entries
        for (const ptr of indexPointers) {
            if (Number(ptr) >= this.fileBuffer.length) {
                console.warn(`Skipping invalid entry pointer: ${ptr} (file size: ${this.fileBuffer.length})`);
                continue;
            }
            
            const entryReader = new BinaryReader(this.fileBuffer.subarray(Number(ptr)));
            
            // Per ZIM spec: mimetype is uint16 (2 bytes), not uint32
            const mimetype = entryReader.readUInt16LE();
            entryReader.seek(0); // Reset position

            if (mimetype === ZIM_REDIRECT_MIMETYPE) { // Redirect entry (0xFFFF)
                const entry = this.readRedirectEntry(entryReader);
                this.directoryEntries.push(entry);
            } else if (mimetype === ZIM_LINKTARGEST_MIMETYPE || mimetype === ZIM_DELETED_MIMETYPE) {
                // Skip link target and deleted entries (not fully supported yet)
                console.warn(`Skipping special entry type: 0x${mimetype.toString(16)}`);
                continue;
            } else { // Article entry
                const entry = this.readDirectoryEntry(entryReader);
                this.directoryEntries.push(entry);
            }
        }
        
        console.log(`Loaded ${this.directoryEntries.length} directory entries`);
    }

    private readDirectoryEntry(reader: BinaryReader): DirectoryEntry {
        const mimetypeIndex = reader.readUInt16LE(); // uint16 per ZIM spec
        reader.readUInt8();    // parameterLen (1 byte, unused)
        const namespace = reader.readUInt8();
        const revision = reader.readUInt32LE();
        const clusterNumber = reader.readUInt32LE();
        const blobNumber = reader.readUInt32LE();
        const url = reader.readNullTerminatedString();
        const title = reader.readNullTerminatedString();

        return {
            mimetypeIndex,
            namespace,
            revision,
            clusterNumber,
            blobNumber,
            url,
            title
        };
    }

    private readRedirectEntry(reader: BinaryReader): RedirectEntry {
        const mimetypeIndex = reader.readUInt16LE(); // uint16 per ZIM spec
        reader.readUInt8();    // parameterLen (1 byte, unused)
        const namespace = reader.readUInt8();
        const revision = reader.readUInt32LE();
        const redirectIndex = reader.readUInt32LE();
        const url = reader.readNullTerminatedString();
        const title = reader.readNullTerminatedString();

        return {
            mimetypeIndex,
            namespace,
            revision,
            redirectIndex,
            url,
            title
        };
    }

    private readClusterPointers(): void {
        if (!this.header || !this.fileBuffer) throw new Error('Header not parsed or file not opened');

        const startPos = Number(this.header.clusterPtrPos);
        const reader = new BinaryReader(this.fileBuffer.subarray(startPos));

        for (let i = 0; i < this.header.clusterCount; i++) {
            this.clusterOffsets.push(reader.readUInt64LE());
        }
    }

    getEntryByPath(path: string): Entry | undefined {
        const namespace = path.charCodeAt(0);
        const url = path.length > 2 ? path.substring(2) : '';

        return this.directoryEntries.find(entry => 
            entry.namespace === namespace && entry.url === url
        );
    }

    getArticleContent(entry: DirectoryEntry): Buffer {
        if (!this.fileBuffer || !this.header) {
            throw new Error('File not opened or header not parsed');
        }

        // Get cluster offset
        const clusterOffset = Number(this.clusterOffsets[entry.clusterNumber]);
        const clusterReader = new BinaryReader(this.fileBuffer.subarray(clusterOffset));

        // Read cluster header (compression type)
        const compressionByte = clusterReader.readUInt8();

        // Read blob offsets
        const blobOffsets: number[] = [];
        while (true) {
            const offset = clusterReader.readUInt32LE();
            blobOffsets.push(offset);
            if (offset === 0) break; // Last offset
        }

        // Calculate blob size and position
        if (entry.blobNumber >= blobOffsets.length - 1) {
            throw new Error('Invalid blob number');
        }

        const blobStart = blobOffsets[entry.blobNumber];
        const blobEnd = blobOffsets[entry.blobNumber + 1];
        const blobSize = blobEnd - blobStart;

        // Read blob data
        const currentPos = clusterReader.getPosition();
        clusterReader.seek(currentPos + blobStart);

        let blobData = clusterReader.readBytes(blobSize);

        // Decompress if needed
        if (compressionByte === CompressionType.ZLIB) {
            blobData = inflateSync(blobData);
        } else if (compressionByte === CompressionType.LZMA) {
            throw new Error('LZMA compression not implemented in this clean-room version');
        }

        return blobData;
    }

    getMainPage(): Entry | undefined {
        if (!this.header) return undefined;

        if (this.header.mainPage !== 0xFFFFFFFF && this.header.mainPage < this.directoryEntries.length) {
            return this.directoryEntries[this.header.mainPage];
        }

        return undefined;
    }

    listArticles(): DirectoryEntry[] {
        return this.directoryEntries.filter(entry => 
            'clusterNumber' in entry && 'blobNumber' in entry
        ) as DirectoryEntry[];
    }

    getMimeTypes(): string[] {
        return [...this.mimeTypes];
    }

    getHeader(): ZIMHeader | undefined {
        return this.header;
    }
}

/**
 * ZIM file writer class
 */
export class ZIMWriter {
    private filePath: string;
    private mimeTypes: string[] = [];
    private directoryEntries: Entry[] = [];
    private clusters: Buffer[] = [];
    private mainPageIndex: number = 0;

    constructor(filePath: string) {
        this.filePath = filePath;
    }

    create(): void {
        // Write placeholder header (80 bytes per ZIM spec)
        const writer = new BinaryWriter(80);
        writer.writeUInt32LE(ZIM_MAGIC_NUMBER);     // offset 0:  magic number (0x044D495A)
        writer.writeUInt16LE(6);                     // offset 4:  major version
        writer.writeUInt16LE(0);                     // offset 6:  minor version
        writer.writeBytes(Buffer.alloc(16));         // offset 8:  UUID placeholder (16 bytes)
        writer.writeUInt32LE(0);                     // offset 24: entry count
        writer.writeUInt32LE(0);                     // offset 28: cluster count
        writer.writeUInt64LE(BigInt(0));              // offset 32: pathPtrPos
        writer.writeUInt64LE(BigInt(0));              // offset 40: titlePtrPos
        writer.writeUInt64LE(BigInt(0));              // offset 48: clusterPtrPos
        writer.writeUInt64LE(BigInt(0));              // offset 56: mimeListPos
        writer.writeUInt32LE(0xFFFFFFFF);            // offset 64: mainPage (none)
        writer.writeUInt32LE(0xFFFFFFFF);            // offset 68: layoutPage (deprecated)
        writer.writeUInt64LE(BigInt(0));              // offset 72: checksumPos

        writeFileSync(this.filePath, writer.getBuffer());
    }

    addMimeType(mimeType: string): number {
        if (!this.mimeTypes.includes(mimeType)) {
            this.mimeTypes.push(mimeType);
        }
        return this.mimeTypes.indexOf(mimeType);
    }

    addArticle(namespace: number, url: string, title: string, content: Buffer, mimeType: string = 'text/html'): void {
        const mimetypeIndex = this.addMimeType(mimeType);

        // Create cluster with content (for simplicity, one blob per cluster)
        const clusterData = this.createCluster([content], CompressionType.DEFAULT);
        const clusterNumber = this.clusters.length;
        this.clusters.push(clusterData);

        const entry: DirectoryEntry = {
            mimetypeIndex,
            namespace,
            revision: 0,
            clusterNumber,
            blobNumber: 0,
            url,
            title
        };

        this.directoryEntries.push(entry);
    }

    addRedirect(namespace: number, url: string, title: string, redirectIndex: number): void {
        const entry: RedirectEntry = {
            mimetypeIndex: 0xFFFF, // Redirect marker
            namespace,
            revision: 0,
            redirectIndex,
            url,
            title
        };

        this.directoryEntries.push(entry);
    }

    private createCluster(blobs: Buffer[], compression: CompressionType): Buffer {
        // Calculate blob offsets
        const offsets: number[] = [0];
        let currentOffset = 0;
        for (const blob of blobs) {
            currentOffset += blob.length;
            offsets.push(currentOffset);
        }

        const writer = new BinaryWriter();
        writer.writeUInt8(compression);

        // Add offsets
        for (const offset of offsets) {
            writer.writeUInt32LE(offset);
        }

        // Add blob data
        for (let blob of blobs) {
            if (compression === CompressionType.ZLIB) {
                blob = Buffer.from(deflateSync(blob));
            }
            writer.writeBytes(blob);
        }

        return writer.getBuffer();
    }

    finalize(): void {
        // Calculate positions
        let currentPos = 80; // After header

        // Write MIME type list
        const mimeTypePos = currentPos;
        const mimeTypeData = Buffer.concat([
            Buffer.from(this.mimeTypes.join('\x00'), 'utf8'),
            Buffer.from([0, 0]) // Double null terminator
        ]);
        currentPos += mimeTypeData.length;

        // Write directory entries
        const directoryPos = currentPos;
        const indexPointers: bigint[] = [];

        for (const entry of this.directoryEntries) {
            indexPointers.push(BigInt(currentPos));
            const entryData = this.serializeEntry(entry);
            currentPos += entryData.length;
        }

        // Write index pointer list
        const indexPtrPos = currentPos;
        currentPos += indexPointers.length * 8;

        // Write cluster pointer list
        const clusterPtrPos = currentPos;
        const clusterOffsets: bigint[] = [];

        for (const cluster of this.clusters) {
            clusterOffsets.push(BigInt(currentPos));
            currentPos += cluster.length;
        }

        // Update header with real values (80 bytes per ZIM spec)
        const headerWriter = new BinaryWriter(80);
        headerWriter.writeUInt32LE(ZIM_MAGIC_NUMBER);                // offset 0:  magic
        headerWriter.writeUInt16LE(6);                                // offset 4:  major version
        headerWriter.writeUInt16LE(0);                                // offset 6:  minor version
        headerWriter.writeBytes(Buffer.alloc(16));                   // offset 8:  UUID placeholder
        headerWriter.writeUInt32LE(this.directoryEntries.length);    // offset 24: entry count
        headerWriter.writeUInt32LE(this.clusters.length);            // offset 28: cluster count
        headerWriter.writeUInt64LE(BigInt(indexPtrPos));              // offset 32: pathPtrPos
        headerWriter.writeUInt64LE(BigInt(0));                       // offset 40: titlePtrPos (not used)
        headerWriter.writeUInt64LE(BigInt(clusterPtrPos));           // offset 48: clusterPtrPos
        headerWriter.writeUInt64LE(BigInt(mimeTypePos));             // offset 56: mimeListPos
        headerWriter.writeUInt32LE(this.mainPageIndex);              // offset 64: mainPage
        headerWriter.writeUInt32LE(0xFFFFFFFF);                      // offset 68: layoutPage (deprecated)
        headerWriter.writeUInt64LE(BigInt(currentPos));              // offset 72: checksumPos

        // Write all data to file
        const finalWriter = new BinaryWriter();
        finalWriter.writeBytes(headerWriter.getBuffer());
        finalWriter.writeBytes(mimeTypeData);

        // Write directory entries
        for (const entry of this.directoryEntries) {
            finalWriter.writeBytes(this.serializeEntry(entry));
        }

        // Write index pointer list
        for (const ptr of indexPointers) {
            finalWriter.writeUInt64LE(ptr);
        }

        // Write cluster pointer list
        for (const offset of clusterOffsets) {
            finalWriter.writeUInt64LE(offset);
        }

        // Write clusters
        for (const cluster of this.clusters) {
            finalWriter.writeBytes(cluster);
        }

        // Write placeholder checksum
        finalWriter.writeBytes(Buffer.alloc(16)); // 16-byte checksum placeholder

        writeFileSync(this.filePath, finalWriter.getBuffer());
    }

    private serializeEntry(entry: Entry): Buffer {
        const writer = new BinaryWriter();

        if ('redirectIndex' in entry) {
            // Redirect entry
            const redirectEntry = entry as RedirectEntry;
            writer.writeUInt16LE(redirectEntry.mimetypeIndex); // uint16 per ZIM spec
            writer.writeUInt8(0);                              // parameterLen (unused)
            writer.writeUInt8(redirectEntry.namespace);
            writer.writeUInt32LE(redirectEntry.revision);
            writer.writeUInt32LE(redirectEntry.redirectIndex);
            writer.writeNullTerminatedString(redirectEntry.url);
            writer.writeNullTerminatedString(redirectEntry.title);
        } else {
            // Directory entry
            const dirEntry = entry as DirectoryEntry;
            writer.writeUInt16LE(dirEntry.mimetypeIndex);      // uint16 per ZIM spec
            writer.writeUInt8(0);                              // parameterLen (unused)
            writer.writeUInt8(dirEntry.namespace);
            writer.writeUInt32LE(dirEntry.revision);
            writer.writeUInt32LE(dirEntry.clusterNumber);
            writer.writeUInt32LE(dirEntry.blobNumber);
            writer.writeNullTerminatedString(dirEntry.url);
            writer.writeNullTerminatedString(dirEntry.title);
        }

        return writer.getBuffer();
    }

    close(): void {
        // Nothing to do for file-based writer
    }
}

// Utility functions
export function readZIMFile(filePath: string): ZIMReader {
    return new ZIMReader(filePath);
}

export function createZIMFile(filePath: string): ZIMWriter {
    return new ZIMWriter(filePath);
}

// Browser-compatible version (for Chrome extensions)
export class ZIMReaderBrowser {
    private arrayBuffer: ArrayBuffer;
    private header?: ZIMHeader;
    private mimeTypes: string[] = [];
    private directoryEntries: Entry[] = [];
    private clusterOffsets: bigint[] = [];

    constructor(arrayBuffer: ArrayBuffer) {
        this.arrayBuffer = arrayBuffer;
    }

    async open(): Promise<void> {
        const buffer = new Uint8Array(this.arrayBuffer);
        const reader = new BinaryReader(Buffer.from(buffer));
        
        this.readHeader(reader);
        await this.readMimeTypes(reader);
        await this.readDirectory(reader);
        this.readClusterPointers(reader);
    }

    private readHeader(reader: BinaryReader): void {
        this.header = {
            magicNumber: reader.readUInt32LE(),       // offset 0
            majorVersion: reader.readUInt16LE(),       // offset 4
            minorVersion: reader.readUInt16LE(),       // offset 6
            uuid: reader.readBytes(16),                // offset 8  (16 bytes)
            entryCount: reader.readUInt32LE(),          // offset 24
            clusterCount: reader.readUInt32LE(),        // offset 28
            pathPtrPos: reader.readUInt64LE(),           // offset 32
            titlePtrPos: reader.readUInt64LE(),          // offset 40
            clusterPtrPos: reader.readUInt64LE(),        // offset 48
            mimeListPos: reader.readUInt64LE(),          // offset 56
            mainPage: reader.readUInt32LE(),             // offset 64
            layoutPage: reader.readUInt32LE(),           // offset 68
            checksumPos: reader.readUInt64LE()           // offset 72
        };
        // reader is now at position 80 (end of header)

        if (this.header.magicNumber !== ZIM_MAGIC_NUMBER) {
            throw new Error(`Invalid ZIM file format: expected magic 0x${ZIM_MAGIC_NUMBER.toString(16).toUpperCase()}, got 0x${this.header.magicNumber.toString(16).toUpperCase()}`);
        }
    }

    private async readMimeTypes(reader: BinaryReader): Promise<void> {
        if (!this.header) throw new Error('Header not parsed');

        reader.seek(Number(this.header.mimeListPos));
        
        const mimeTypesData: number[] = [];
        let prevByte = 0;
        let nullNullCount = 0;

        while (nullNullCount < 2 && reader.getPosition() < this.arrayBuffer.byteLength) {
            const byte = reader.readUInt8();
            mimeTypesData.push(byte);
            
            if (byte === 0 && prevByte === 0) {
                nullNullCount++;
            }
            prevByte = byte;
        }

        const mimeTypesString = Buffer.from(mimeTypesData).toString('utf8');
        this.mimeTypes = mimeTypesString.split('\x00').filter(mt => mt.length > 0);
    }

    private async readDirectory(reader: BinaryReader): Promise<void> {
        if (!this.header) throw new Error('Header not parsed');

        // Read path pointer list from header-specified position
        reader.seek(Number(this.header.pathPtrPos));
        
        const indexPointers: bigint[] = [];
        for (let i = 0; i < this.header.entryCount; i++) {
            indexPointers.push(reader.readUInt64LE());
        }

        // Read directory entries
        for (const ptr of indexPointers) {
            reader.seek(Number(ptr));
            
            // Per ZIM spec: mimetype is uint16 (2 bytes), not uint32
            const mimetype = reader.readUInt16LE();
            reader.seek(Number(ptr)); // Reset

            if (mimetype === 0xFFFF) {
                const entry = this.readRedirectEntry(reader);
                this.directoryEntries.push(entry);
            } else {
                const entry = this.readDirectoryEntry(reader);
                this.directoryEntries.push(entry);
            }
        }
    }

    private readDirectoryEntry(reader: BinaryReader): DirectoryEntry {
        const mimetypeIndex = reader.readUInt16LE(); // uint16 per ZIM spec
        reader.readUInt8();    // parameterLen (1 byte, unused)
        const namespace = reader.readUInt8();
        const revision = reader.readUInt32LE();
        const clusterNumber = reader.readUInt32LE();
        const blobNumber = reader.readUInt32LE();
        const url = reader.readNullTerminatedString();
        const title = reader.readNullTerminatedString();

        return {
            mimetypeIndex,
            namespace,
            revision,
            clusterNumber,
            blobNumber,
            url,
            title
        };
    }

    private readRedirectEntry(reader: BinaryReader): RedirectEntry {
        const mimetypeIndex = reader.readUInt16LE(); // uint16 per ZIM spec
        reader.readUInt8();    // parameterLen (1 byte, unused)
        const namespace = reader.readUInt8();
        const revision = reader.readUInt32LE();
        const redirectIndex = reader.readUInt32LE();
        const url = reader.readNullTerminatedString();
        const title = reader.readNullTerminatedString();

        return {
            mimetypeIndex,
            namespace,
            revision,
            redirectIndex,
            url,
            title
        };
    }

    private readClusterPointers(reader: BinaryReader): void {
        if (!this.header) throw new Error('Header not parsed');

        reader.seek(Number(this.header.clusterPtrPos));

        for (let i = 0; i < this.header.clusterCount; i++) {
            this.clusterOffsets.push(reader.readUInt64LE());
        }
    }

    getEntryByPath(path: string): Entry | undefined {
        const namespace = path.charCodeAt(0);
        const url = path.length > 2 ? path.substring(2) : '';

        return this.directoryEntries.find(entry => 
            entry.namespace === namespace && entry.url === url
        );
    }

    async getArticleContent(entry: DirectoryEntry): Promise<Uint8Array> {
        if (!this.header) throw new Error('Header not parsed');

        const buffer = new Uint8Array(this.arrayBuffer);
        const reader = new BinaryReader(Buffer.from(buffer));

        // Get cluster offset
        const clusterOffset = Number(this.clusterOffsets[entry.clusterNumber]);
        reader.seek(clusterOffset);

        // Read cluster header (compression type)
        const compressionByte = reader.readUInt8();

        // Read blob offsets
        const blobOffsets: number[] = [];
        while (true) {
            const offset = reader.readUInt32LE();
            blobOffsets.push(offset);
            if (offset === 0) break;
        }

        // Calculate blob size and position
        if (entry.blobNumber >= blobOffsets.length - 1) {
            throw new Error('Invalid blob number');
        }

        const blobStart = blobOffsets[entry.blobNumber];
        const blobEnd = blobOffsets[entry.blobNumber + 1];
        const blobSize = blobEnd - blobStart;

        // Read blob data
        const currentPos = reader.getPosition();
        reader.seek(currentPos + blobStart);

        let blobData = reader.readBytes(blobSize);

        // Decompress if needed
        if (compressionByte === CompressionType.ZLIB) {
            blobData = Buffer.from(inflateSync(blobData));
        }

        return new Uint8Array(blobData);
    }

    getMainPage(): Entry | undefined {
        if (!this.header) return undefined;

        if (this.header.mainPage !== 0xFFFFFFFF && this.header.mainPage < this.directoryEntries.length) {
            return this.directoryEntries[this.header.mainPage];
        }

        return undefined;
    }

    listArticles(): DirectoryEntry[] {
        return this.directoryEntries.filter(entry => 
            'clusterNumber' in entry && 'blobNumber' in entry
        ) as DirectoryEntry[];
    }
}
