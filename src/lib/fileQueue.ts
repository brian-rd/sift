import type { DownloadFile, FileKind, FileSortField, SiftQueuePreferences, SortDirection } from './types';

export const FILE_KINDS: FileKind[] = ['image', 'pdf', 'archive', 'video', 'audio', 'text', 'other'];

export const DEFAULT_QUEUE_PREFERENCES: SiftQueuePreferences = {
  sortBy: 'modifiedAt',
  direction: 'desc',
  includedKinds: [...FILE_KINDS],
};

const nameCollator = new Intl.Collator(undefined, { numeric: true, sensitivity: 'base' });

export function prepareSiftQueue(files: DownloadFile[], preferences: SiftQueuePreferences): DownloadFile[] {
  const includedKinds = new Set(preferences.includedKinds);
  return files
    .filter((file) => includedKinds.has(file.kind))
    .sort((left, right) => compareFiles(left, right, preferences.sortBy, preferences.direction));
}

export function isSiftQueuePreferences(value: unknown): value is SiftQueuePreferences {
  if (!value || typeof value !== 'object') return false;
  const candidate = value as Partial<SiftQueuePreferences>;
  return (
    ['modifiedAt', 'createdAt', 'name', 'size', 'type'].includes(candidate.sortBy ?? '') &&
    ['asc', 'desc'].includes(candidate.direction ?? '') &&
    Array.isArray(candidate.includedKinds) &&
    candidate.includedKinds.length > 0 &&
    candidate.includedKinds.every((kind) => FILE_KINDS.includes(kind)) &&
    new Set(candidate.includedKinds).size === candidate.includedKinds.length
  );
}

function compareFiles(
  left: DownloadFile,
  right: DownloadFile,
  sortBy: FileSortField,
  direction: SortDirection,
) {
  const multiplier = direction === 'asc' ? 1 : -1;
  let comparison = 0;

  switch (sortBy) {
    case 'name':
      comparison = nameCollator.compare(left.name, right.name);
      break;
    case 'size':
      comparison = left.size - right.size;
      break;
    case 'type':
      comparison = nameCollator.compare(left.extension || left.kind, right.extension || right.kind);
      break;
    case 'createdAt':
      comparison = left.createdAt - right.createdAt;
      break;
    case 'modifiedAt':
      comparison = left.modifiedAt - right.modifiedAt;
      break;
  }

  return comparison * multiplier || nameCollator.compare(left.name, right.name);
}
