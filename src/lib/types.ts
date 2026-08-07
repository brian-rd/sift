export type Screen = 'dashboard' | 'triage' | 'rules' | 'history' | 'settings';

export type FileKind = 'image' | 'pdf' | 'archive' | 'video' | 'audio' | 'text' | 'other';

export interface DownloadFile {
  path: string;
  name: string;
  extension: string;
  size: number;
  modifiedAt: number;
  kind: FileKind;
  suggestedFolder?: string;
  matchedRule?: string;
  previewUrl?: string;
}

export type RuleConditionType = 'extension' | 'contains' | 'startsWith' | 'endsWith' | 'glob' | 'regex' | 'size' | 'age';
export type RuleActionType = 'move' | 'rename' | 'trash' | 'ignore' | 'review';

export interface Rule {
  id: string;
  name: string;
  conditionType: RuleConditionType;
  conditionValue: string;
  actionType: RuleActionType;
  destination?: string;
  enabled: boolean;
  matches: number;
}

export interface HistoryItem {
  id: string;
  fileName: string;
  action: 'Moved' | 'Kept' | 'Trashed' | 'Review later' | 'Renamed';
  destination?: string;
  timestamp: number;
  session: string;
  undoable: boolean;
  backendOperationId?: number;
}

export interface ScanResult {
  folder: string;
  files: DownloadFile[];
  totalBytes: number;
  skippedIncomplete: number;
}
