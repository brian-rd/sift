import { describe, expect, it } from 'vitest';
import { DEFAULT_QUEUE_PREFERENCES, prepareSiftQueue } from './fileQueue';
import type { DownloadFile, SiftQueuePreferences } from './types';

const files: DownloadFile[] = [
  {
    path: 'C:\\Downloads\\photo10.jpg',
    name: 'photo10.jpg',
    extension: 'jpg',
    size: 500,
    modifiedAt: 300,
    createdAt: 100,
    kind: 'image',
  },
  {
    path: 'C:\\Downloads\\notes.txt',
    name: 'notes.txt',
    extension: 'txt',
    size: 100,
    modifiedAt: 100,
    createdAt: 300,
    kind: 'text',
  },
  {
    path: 'C:\\Downloads\\photo2.png',
    name: 'photo2.png',
    extension: 'png',
    size: 300,
    modifiedAt: 200,
    createdAt: 200,
    kind: 'image',
  },
];

function preferences(overrides: Partial<SiftQueuePreferences>): SiftQueuePreferences {
  return { ...DEFAULT_QUEUE_PREFERENCES, ...overrides };
}

describe('prepareSiftQueue', () => {
  it('sorts newest modified files first by default without mutating the scan results', () => {
    const result = prepareSiftQueue(files, DEFAULT_QUEUE_PREFERENCES);

    expect(result.map((file) => file.name)).toEqual(['photo10.jpg', 'photo2.png', 'notes.txt']);
    expect(files.map((file) => file.name)).toEqual(['photo10.jpg', 'notes.txt', 'photo2.png']);
  });

  it('sorts names naturally in either direction', () => {
    expect(prepareSiftQueue(files, preferences({ sortBy: 'name', direction: 'asc' }))[1].name).toBe(
      'photo2.png',
    );
    expect(prepareSiftQueue(files, preferences({ sortBy: 'name', direction: 'desc' }))[0].name).toBe(
      'photo10.jpg',
    );
  });

  it('sorts by size and creation date', () => {
    expect(
      prepareSiftQueue(files, preferences({ sortBy: 'size', direction: 'asc' })).map((file) => file.size),
    ).toEqual([100, 300, 500]);
    expect(
      prepareSiftQueue(files, preferences({ sortBy: 'createdAt', direction: 'desc' })).map(
        (file) => file.createdAt,
      ),
    ).toEqual([300, 200, 100]);
  });

  it('sorts file types by extension', () => {
    expect(
      prepareSiftQueue(files, preferences({ sortBy: 'type', direction: 'asc' })).map(
        (file) => file.extension,
      ),
    ).toEqual(['jpg', 'png', 'txt']);
  });

  it('includes only selected file categories', () => {
    expect(
      prepareSiftQueue(files, preferences({ includedKinds: ['image'] })).map((file) => file.name),
    ).toEqual(['photo10.jpg', 'photo2.png']);
  });
});
