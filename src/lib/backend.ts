import type { PinnedDestination, ScanResult } from './types';

export const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

export interface OperationResult {
  operationId: number;
  source: string;
  destination?: string;
}

async function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
  return tauriInvoke<T>(command, args);
}

export async function scanDownloads(folder?: string): Promise<ScanResult> {
  return invoke<ScanResult>('scan_downloads', { folder: folder || null });
}

export async function moveDownload(source: string, destination: string) {
  return invoke<OperationResult>('move_download', { source, destination });
}

export async function trashDownload(path: string) {
  return invoke<OperationResult>('trash_download', { path });
}

export async function revealDownload(path: string) {
  return invoke('reveal_download', { path });
}

export async function undoOperation(operationId: number) {
  return invoke('undo_operation', { operationId });
}

export async function getDefaultDestinations() {
  return invoke<PinnedDestination[]>('default_destinations');
}
