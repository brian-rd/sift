<script lang="ts">
  import { onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import Sidebar from './components/Sidebar.svelte';
  import Dashboard from './components/Dashboard.svelte';
  import SiftMode from './components/SiftMode.svelte';
  import Rules from './components/Rules.svelte';
  import History from './components/History.svelte';
  import Settings from './components/Settings.svelte';
  import type { DownloadFile, HistoryItem, PinnedDestination, Rule, Screen, ShortcutBindings } from './lib/types';
  import type { RuleMatch } from './lib/rules';
  import { demoFiles, demoHistory, demoRules } from './lib/demo';
  import { getDefaultDestinations, isTauri, moveDownload, revealDownload, scanDownloads, trashDownload, undoOperation } from './lib/backend';

  const defaultShortcuts: ShortcutBindings = { keep: 'ArrowUp', trash: 'ArrowDown', undo: 'ArrowLeft', fileAway: 'ArrowRight' };
  const demoDestinations: PinnedDestination[] = [
    { name: 'Documents', path: 'Documents' },
    { name: 'Pictures', path: 'Pictures' },
    { name: 'Work', path: 'Documents / Work' },
    { name: 'Receipts', path: 'Documents / Receipts' }
  ];

  let active: Screen = 'dashboard';
  let files: DownloadFile[] = isTauri ? [] : demoFiles;
  let rules: Rule[] = isTauri ? [] : demoRules;
  let history: HistoryItem[] = isTauri ? [] : demoHistory;
  let pinnedDestinations: PinnedDestination[] = isTauri ? [] : demoDestinations;
  let shortcuts: ShortcutBindings = defaultShortcuts;
  let rememberedDestinations: Record<string, PinnedDestination> = {};
  let watchedFolder = isTauri ? '' : 'C:\\Users\\you\\Downloads';
  let scanning = false;
  let toast = '';
  let toastTimer: ReturnType<typeof setTimeout>;

  function notify(message: string) {
    toast = message;
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => toast = '', 2600);
  }

  function destinationName(path: string) {
    return path.replace(/[\\/]+$/, '').split(/[\\/]/).pop() || path;
  }

  function getSuggestions(file: DownloadFile): PinnedDestination[] {
    const remembered = rememberedDestinations[file.extension.toLowerCase()];
    const paths = [remembered?.path, ...(file.suggestedFolders ?? []), file.suggestedFolder].filter((path): path is string => Boolean(path));
    return [...new Set(paths)].map((path) => pinnedDestinations.find((destination) => destination.path === path) ?? { name: destinationName(path), path });
  }

  function rememberDestination(file: DownloadFile, destination: PinnedDestination) {
    if (!file.extension) return;
    rememberedDestinations = { ...rememberedDestinations, [file.extension.toLowerCase()]: destination };
    localStorage.setItem('sift:remembered-destinations', JSON.stringify(rememberedDestinations));
  }

  async function scan() {
    scanning = true;
    if (!isTauri) {
      setTimeout(() => { scanning = false; notify(`Scan complete — ${files.length} files found`); }, 700);
      return;
    }
    try {
      const result = await scanDownloads(watchedFolder);
      files = result.files;
      watchedFolder = result.folder;
      notify(`Scan complete — ${result.files.length} files found${result.skippedIncomplete ? `, ${result.skippedIncomplete} still downloading` : ''}`);
    } catch (error) { notify(`Could not scan folder: ${error}`); }
    finally { scanning = false; }
  }

  function record(file: DownloadFile, action: HistoryItem['action'], destination?: string, backendOperationId?: number) {
    history = [{ id: crypto.randomUUID(), fileName: file.name, action, destination, timestamp: Date.now(), session: 'Just now', undoable: true, backendOperationId, file }, ...history];
  }

  async function siftAction(file: DownloadFile, action: 'trash' | 'keep' | 'fileAway', destination?: PinnedDestination): Promise<boolean> {
    try {
      let operationId: number | undefined;
      if (isTauri && action === 'trash') operationId = (await trashDownload(file.path)).operationId;
      if (action === 'fileAway') {
        if (!destination) return false;
        if (isTauri) operationId = (await moveDownload(file.path, destination.path)).operationId;
        rememberDestination(file, destination);
      }
      const historyAction: HistoryItem['action'] = action === 'trash' ? 'Trashed' : action === 'keep' ? 'Kept' : 'Moved';
      record(file, historyAction, action === 'fileAway' ? destination?.path : undefined, operationId);
      files = files.filter((item) => item.path !== file.path);
      return true;
    } catch (error) {
      notify(`Action failed: ${error}`);
      return false;
    }
  }

  async function undoLatest(): Promise<DownloadFile | null> {
    const item = history.find((entry) => entry.undoable);
    if (!item) { notify('Nothing to undo'); return null; }
    return undoItem(item.id);
  }

  async function undoItem(id: string): Promise<DownloadFile | null> {
    const item = history.find((entry) => entry.id === id);
    if (!item) return null;
    try {
      if (isTauri && item.backendOperationId) await undoOperation(item.backendOperationId);
    } catch (error) { notify(`Could not undo ${item.fileName}: ${error}`); return null; }
    history = history.filter((entry) => entry.id !== id);
    if (item.file && !files.some((file) => file.path === item.file?.path)) files = [item.file, ...files];
    notify(`Restored “${item.fileName}”`);
    return item.file ?? null;
  }

  async function undoSession(session: string) {
    const entries = history.filter((item) => item.session === session && item.undoable);
    const restored: DownloadFile[] = [];
    try {
      for (const item of entries) {
        if (isTauri && item.backendOperationId) await undoOperation(item.backendOperationId);
        if (item.file) restored.push(item.file);
      }
    } catch (error) { notify(`Session undo stopped: ${error}`); return; }
    history = history.filter((item) => item.session !== session || !item.undoable);
    files = [...restored.filter((restoredFile) => !files.some((file) => file.path === restoredFile.path)), ...files];
    notify(`${entries.length} session ${entries.length === 1 ? 'action' : 'actions'} restored`);
  }

  async function runRules(matches: RuleMatch[]) {
    const completed: string[] = [];
    for (const { file, rule } of matches) {
      try {
        let operationId: number | undefined;
        if (isTauri && rule.actionType === 'move' && rule.destination) operationId = (await moveDownload(file.path, rule.destination)).operationId;
        if (isTauri && rule.actionType === 'trash') operationId = (await trashDownload(file.path)).operationId;
        const action: HistoryItem['action'] = rule.actionType === 'trash' ? 'Trashed' : rule.actionType === 'review' ? 'Review later' : rule.actionType === 'rename' ? 'Renamed' : rule.actionType === 'ignore' ? 'Kept' : 'Moved';
        record(file, action, rule.actionType === 'move' ? rule.destination : undefined, operationId);
        completed.push(file.path);
      } catch (error) { notify(`Stopped at ${file.name}: ${error}`); break; }
    }
    files = files.filter((file) => !completed.includes(file.path));
    if (completed.length) notify(`${completed.length} ${completed.length === 1 ? 'file' : 'files'} processed safely`);
  }

  async function pickWatchedFolder() {
    if (!isTauri) return notify('Folder picker is available in the Windows app');
    const selected = await open({ directory: true, multiple: false, title: 'Choose a folder for Sift to watch' });
    if (typeof selected === 'string') { watchedFolder = selected; await scan(); }
  }

  async function pickDestination(file: DownloadFile): Promise<PinnedDestination | null> {
    if (!isTauri) { notify(`The Windows folder picker would open for ${file.name}`); return null; }
    const selected = await open({ directory: true, multiple: false, title: `File away ${file.name}` });
    return typeof selected === 'string' ? { name: destinationName(selected), path: selected } : null;
  }

  async function openFile(file: DownloadFile) {
    if (!isTauri) return notify(`File Explorer would reveal ${file.name}`);
    try { await revealDownload(file.path); } catch (error) { notify(`Could not show file: ${error}`); }
  }

  onMount(async () => {
    try { rememberedDestinations = JSON.parse(localStorage.getItem('sift:remembered-destinations') ?? '{}'); } catch { rememberedDestinations = {}; }
    if (isTauri) {
      try { pinnedDestinations = await getDefaultDestinations(); } catch { pinnedDestinations = []; }
      await scan();
    }
  });
</script>

<div class="app-shell">
  {#if active !== 'sift'}<Sidebar {active} onNavigate={(screen) => active = screen} reviewCount={files.length} />{/if}
  <div class:sift-view={active === 'sift'} class="content-shell">
    {#if active === 'dashboard'}
      <main class="page"><Dashboard {files} {scanning} isDemo={!isTauri} onScan={scan} onSift={() => active = 'sift'} onRules={() => active = 'rules'} onPreviewRules={() => active = 'rules'} /></main>
    {:else if active === 'sift'}
      <SiftMode {files} {pinnedDestinations} {shortcuts} {getSuggestions} onAction={siftAction} onPickDestination={pickDestination} onBack={() => active = 'dashboard'} onOpen={openFile} onUndo={undoLatest} />
    {:else if active === 'rules'}
      <main class="page"><Rules {rules} {files} onUpdate={(next) => rules = next} onRun={runRules} /></main>
    {:else if active === 'history'}
      <main class="page"><History items={history} onUndo={undoItem} onUndoSession={undoSession} /></main>
    {:else if active === 'settings'}
      <main class="page"><Settings {watchedFolder} onFolderChange={(folder) => watchedFolder = folder} onPickFolder={pickWatchedFolder} /></main>
    {/if}
  </div>
</div>

{#if toast}<div class="toast" role="status">{toast}</div>{/if}
