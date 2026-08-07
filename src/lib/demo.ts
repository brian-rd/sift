import type { DownloadFile, HistoryItem, Rule } from './types';

const now = Date.now();

export const demoFiles: DownloadFile[] = [
  { path: 'C:\\Users\\you\\Downloads\\mountain-lake.jpg', name: 'mountain-lake.jpg', extension: 'jpg', size: 4_820_000, modifiedAt: now - 38 * 60_000, createdAt: now - 2 * 86_400_000, kind: 'image', suggestedFolders: ['Pictures / Wallpapers', 'Pictures'], previewUrl: 'https://images.unsplash.com/photo-1500534314209-a25ddb2bd429?auto=format&fit=crop&w=1400&q=85' },
  { path: 'C:\\Users\\you\\Downloads\\Q2-expense-report.pdf', name: 'Q2-expense-report.pdf', extension: 'pdf', size: 1_860_000, modifiedAt: now - 2 * 86_400_000, createdAt: now - 6 * 86_400_000, kind: 'pdf', suggestedFolders: ['Documents / Reports'], matchedRule: 'Expense reports' },
  { path: 'C:\\Users\\you\\Downloads\\sift-logo-final-04.png', name: 'sift-logo-final-04.png', extension: 'png', size: 8_200_000, modifiedAt: now - 4 * 86_400_000, createdAt: now - 8 * 86_400_000, kind: 'image', suggestedFolders: ['Pictures / Design'], previewUrl: 'https://images.unsplash.com/photo-1558655146-d09347e92766?auto=format&fit=crop&w=1400&q=85' },
  { path: 'C:\\Users\\you\\Downloads\\project-assets.zip', name: 'project-assets.zip', extension: 'zip', size: 184_200_000, modifiedAt: now - 9 * 86_400_000, createdAt: now - 15 * 86_400_000, kind: 'archive', suggestedFolders: ['Documents / Projects'] },
  { path: 'C:\\Users\\you\\Downloads\\meeting-notes.txt', name: 'meeting-notes.txt', extension: 'txt', size: 18_600, modifiedAt: now - 12 * 86_400_000, createdAt: now - 12 * 86_400_000, kind: 'text', suggestedFolders: ['Documents / Notes'] },
  { path: 'C:\\Users\\you\\Downloads\\screen-recording.mov', name: 'screen-recording.mov', extension: 'mov', size: 684_000_000, modifiedAt: now - 34 * 86_400_000, createdAt: now - 34 * 86_400_000, kind: 'video', suggestedFolders: ['Videos / Recordings'] },
  { path: 'C:\\Users\\you\\Downloads\\invoice-1048.pdf', name: 'invoice-1048.pdf', extension: 'pdf', size: 482_000, modifiedAt: now - 46 * 86_400_000, createdAt: now - 48 * 86_400_000, kind: 'pdf', suggestedFolders: ['Documents / Finance'], matchedRule: 'Invoices' },
  { path: 'C:\\Users\\you\\Downloads\\focus-mix.mp3', name: 'focus-mix.mp3', extension: 'mp3', size: 12_400_000, modifiedAt: now - 62 * 86_400_000, createdAt: now - 62 * 86_400_000, kind: 'audio', suggestedFolders: ['Music'] },
  { path: 'C:\\Users\\you\\Downloads\\installer.exe', name: 'installer.exe', extension: 'exe', size: 92_600_000, modifiedAt: now - 91 * 86_400_000, createdAt: now - 91 * 86_400_000, kind: 'other' },
  { path: 'C:\\Users\\you\\Downloads\\reading-list.pdf', name: 'reading-list.pdf', extension: 'pdf', size: 3_100_000, modifiedAt: now - 3 * 86_400_000, createdAt: now - 5 * 86_400_000, kind: 'pdf', suggestedFolders: ['Documents / Reading'] },
  { path: 'C:\\Users\\you\\Downloads\\portrait-raw.jpg', name: 'portrait-raw.jpg', extension: 'jpg', size: 26_200_000, modifiedAt: now - 8 * 86_400_000, createdAt: now - 8 * 86_400_000, kind: 'image', suggestedFolders: ['Pictures'], previewUrl: 'https://images.unsplash.com/photo-1500648767791-00dcc994a43e?auto=format&fit=crop&w=1400&q=85' },
  { path: 'C:\\Users\\you\\Downloads\\sample-data.csv', name: 'sample-data.csv', extension: 'csv', size: 840_000, modifiedAt: now - 16 * 86_400_000, createdAt: now - 20 * 86_400_000, kind: 'text', suggestedFolders: ['Documents / Data'] }
];

export const demoRules: Rule[] = [
  { id: 'rule-1', name: 'Expense reports', conditionType: 'contains', conditionValue: 'expense-report', actionType: 'move', destination: 'Documents / Reports', enabled: true, matches: 1 },
  { id: 'rule-2', name: 'Invoices', conditionType: 'startsWith', conditionValue: 'invoice-', actionType: 'move', destination: 'Documents / Finance', enabled: true, matches: 1 },
  { id: 'rule-3', name: 'Screenshots', conditionType: 'startsWith', conditionValue: 'Screenshot', actionType: 'move', destination: 'Pictures / Screenshots', enabled: true, matches: 0 },
  { id: 'rule-4', name: 'Old installers', conditionType: 'age', conditionValue: '60', actionType: 'review', enabled: false, matches: 1 }
];

export const demoHistory: HistoryItem[] = [
  { id: 'history-1', fileName: 'tax-statement.pdf', action: 'Moved', destination: 'Documents / Finance', timestamp: now - 18 * 60_000, session: 'Today, 09:42', undoable: true },
  { id: 'history-2', fileName: 'wallpaper-08.jpg', action: 'Moved', destination: 'Pictures / Wallpapers', timestamp: now - 19 * 60_000, session: 'Today, 09:42', undoable: true },
  { id: 'history-3', fileName: 'setup-old.exe', action: 'Trashed', timestamp: now - 21 * 60_000, session: 'Today, 09:42', undoable: false, trashState: 'recycled' },
  { id: 'history-4', fileName: 'notes-draft.txt', action: 'Kept', timestamp: now - 22 * 60_000, session: 'Today, 09:42', undoable: false },
  { id: 'history-5', fileName: 'project-brief.pdf', action: 'Review later', timestamp: now - 24 * 60_000, session: 'Today, 09:42', undoable: true }
];
