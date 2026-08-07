export type Screen = 'dashboard' | 'sift' | 'rules' | 'history' | 'settings';

export type FileKind = 'image' | 'pdf' | 'archive' | 'video' | 'audio' | 'text' | 'other';

export interface DownloadFile {
  path: string;
  name: string;
  extension: string;
  size: number;
  modifiedAt: number;
  createdAt: number;
  kind: FileKind;
  suggestedFolder?: string;
  suggestedFolders?: string[];
  matchedRule?: string;
  previewUrl?: string;
}

export interface PinnedDestination {
  name: string;
  path: string;
}

export interface TrashItem {
  operationId: number;
  originalPath: string;
  stagedPath: string;
  name: string;
  extension: string;
  size: number;
  modifiedAt: number;
  createdAt: number;
  kind: FileKind;
}

export type ShortcutAction = 'keep' | 'trash' | 'undo' | 'fileAway';
export type ShortcutBindings = Record<ShortcutAction, string>;
export type ThemePreference = 'system' | 'light' | 'dark';

export type RuleConditionType =
  'extension' | 'contains' | 'startsWith' | 'endsWith' | 'glob' | 'regex' | 'size' | 'age';
export type RuleActionType = 'move' | 'trash' | 'ignore';

export interface Rule {
  id: string;
  name: string;
  conditionType: RuleConditionType;
  conditionValue: string;
  actionType: RuleActionType;
  destination?: string;
  enabled: boolean;
}

export interface HistoryItem {
  id: string;
  fileName: string;
  action: 'Moved' | 'Kept' | 'Trashed';
  destination?: string;
  timestamp: number;
  session: string;
  undoable: boolean;
  backendOperationId?: number;
  file?: DownloadFile;
  trashState?: 'staged' | 'recycled';
}

export interface ScanResult {
  folder: string;
  files: DownloadFile[];
  totalBytes: number;
  skippedIncomplete: number;
}
