<script lang="ts">
  import { onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import Sidebar from './components/Sidebar.svelte';
  import Dashboard from './components/Dashboard.svelte';
  import SiftMode from './components/SiftMode.svelte';
  import Rules from './components/Rules.svelte';
  import History from './components/History.svelte';
  import Settings from './components/Settings.svelte';
  import TrashReview from './components/TrashReview.svelte';
  import type { DownloadFile, HistoryItem, PinnedDestination, Rule, Screen, ShortcutBindings, ThemePreference, TrashItem } from './lib/types';
  import type { RuleMatch } from './lib/rules';
  import { demoFiles, demoHistory, demoRules } from './lib/demo';
  import { finalizeTrash, getDefaultDestinations, getUserDisplayName, isTauri, listTrash, moveDownload, openDownload, openRecycleBin, readTextPreview, revealDownload, scanDownloads, trashDownload, undoOperation } from './lib/backend';
  import { DEFAULT_SHORTCUTS } from './lib/shortcuts';
  import { personalisedGreeting } from './lib/greeting';

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
  let shortcuts: ShortcutBindings = { ...DEFAULT_SHORTCUTS };
  let theme: ThemePreference = 'system';
  let rememberedDestinations: Record<string, PinnedDestination> = {};
  let watchedFolder = isTauri ? '' : 'C:\\Users\\you\\Downloads';
  let scanning = false;
  let toast = '';
  let toastTimer: ReturnType<typeof setTimeout>;
  let trashItems: TrashItem[] = [];
  let showTrash = false;
  let trashIntent: 'review' | 'overview' | 'exit' = 'review';
  let trashBusy = false;
  let allowWindowClose = false;
  let userName = '';
  let currentHour = new Date().getHours();

  $: greeting = personalisedGreeting(currentHour, userName);

  function notify(message: string) {
    toast = message;
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => toast = '', 2600);
  }

  function applyTheme(preference: ThemePreference) {
    const isDark = preference === 'dark' || (preference === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);
    document.documentElement.dataset.theme = preference;
    document.documentElement.dataset.colorScheme = isDark ? 'dark' : 'light';
  }

  function updateTheme(preference: ThemePreference) {
    theme = preference;
    localStorage.setItem('sift:theme', preference);
    applyTheme(preference);
  }

  function updateShortcuts(next: ShortcutBindings) {
    shortcuts = next;
    localStorage.setItem('sift:shortcuts', JSON.stringify(next));
  }

  function updatePinnedDestinations(next: PinnedDestination[]) {
    pinnedDestinations = next.slice(0, 9);
    localStorage.setItem('sift:pinned-destinations', JSON.stringify(pinnedDestinations));
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
      files = result.files.map((file) => ['image', 'pdf', 'video', 'audio'].includes(file.kind) ? { ...file, previewUrl: convertFileSrc(file.path) } : file);
      watchedFolder = result.folder;
      notify(`Scan complete — ${result.files.length} files found${result.skippedIncomplete ? `, ${result.skippedIncomplete} still downloading` : ''}`);
    } catch (error) { notify(`Could not scan folder: ${error}`); }
    finally { scanning = false; }
  }

  function record(file: DownloadFile, action: HistoryItem['action'], destination?: string, backendOperationId?: number, trashState?: HistoryItem['trashState']) {
    history = [{ id: crypto.randomUUID(), fileName: file.name, action, destination, timestamp: Date.now(), session: 'Just now', undoable: true, backendOperationId, file, trashState }, ...history];
  }

  async function siftAction(file: DownloadFile, action: 'trash' | 'keep' | 'fileAway', destination?: PinnedDestination): Promise<boolean> {
    try {
      let operationId: number | undefined;
      if (action === 'trash') {
        const result = isTauri ? await trashDownload(file.path) : { operationId: Date.now(), source: file.path, destination: file.path };
        operationId = result.operationId;
        trashItems = [{ operationId, originalPath: file.path, stagedPath: result.destination ?? file.path, name: file.name, extension: file.extension, size: file.size, modifiedAt: file.modifiedAt, createdAt: file.createdAt, kind: file.kind }, ...trashItems];
      }
      if (action === 'fileAway') {
        if (!destination) return false;
        if (isTauri) operationId = (await moveDownload(file.path, destination.path)).operationId;
        rememberDestination(file, destination);
      }
      const historyAction: HistoryItem['action'] = action === 'trash' ? 'Trashed' : action === 'keep' ? 'Kept' : 'Moved';
      record(file, historyAction, action === 'fileAway' ? destination?.path : undefined, operationId, action === 'trash' ? 'staged' : undefined);
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
    if (item.backendOperationId) trashItems = trashItems.filter((trashItem) => trashItem.operationId !== item.backendOperationId);
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
    const restoredIds = new Set(entries.map((item) => item.backendOperationId).filter((id): id is number => typeof id === 'number'));
    trashItems = trashItems.filter((item) => !restoredIds.has(item.operationId));
    files = [...restored.filter((restoredFile) => !files.some((file) => file.path === restoredFile.path)), ...files];
    notify(`${entries.length} session ${entries.length === 1 ? 'action' : 'actions'} restored`);
  }

  async function runRules(matches: RuleMatch[]) {
    const completed: string[] = [];
    for (const { file, rule } of matches) {
      try {
        let operationId: number | undefined;
        if (isTauri && rule.actionType === 'move' && rule.destination) operationId = (await moveDownload(file.path, rule.destination)).operationId;
        if (rule.actionType === 'trash') {
          const result = isTauri ? await trashDownload(file.path) : { operationId: Date.now(), source: file.path, destination: file.path };
          operationId = result.operationId;
          trashItems = [{ operationId, originalPath: file.path, stagedPath: result.destination ?? file.path, name: file.name, extension: file.extension, size: file.size, modifiedAt: file.modifiedAt, createdAt: file.createdAt, kind: file.kind }, ...trashItems];
        }
        const action: HistoryItem['action'] = rule.actionType === 'trash' ? 'Trashed' : rule.actionType === 'review' ? 'Review later' : rule.actionType === 'rename' ? 'Renamed' : rule.actionType === 'ignore' ? 'Kept' : 'Moved';
        record(file, action, rule.actionType === 'move' ? rule.destination : undefined, operationId, rule.actionType === 'trash' ? 'staged' : undefined);
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

  async function addPinnedDestination() {
    if (!isTauri) return notify('Pinned folders can be selected in the Windows app');
    const selected = await open({ directory: true, multiple: false, title: 'Pin a destination in Sift' });
    if (typeof selected !== 'string') return;
    if (pinnedDestinations.some((destination) => destination.path.toLowerCase() === selected.toLowerCase())) return notify('That destination is already pinned');
    updatePinnedDestinations([...pinnedDestinations, { name: destinationName(selected), path: selected }]);
  }

  function removePinnedDestination(destination: PinnedDestination) {
    updatePinnedDestinations(pinnedDestinations.filter((item) => item.path !== destination.path));
  }

  function trashFileSnapshot(item: TrashItem): DownloadFile {
    const file: DownloadFile = { path: item.originalPath, name: item.name, extension: item.extension, size: item.size, modifiedAt: item.modifiedAt, createdAt: item.createdAt, kind: item.kind };
    return isTauri && ['image', 'pdf', 'video', 'audio'].includes(item.kind) ? { ...file, previewUrl: convertFileSrc(item.originalPath) } : file;
  }

  async function restoreTrash(operationIds: number[]) {
    if (!operationIds.length || trashBusy) return;
    trashBusy = true;
    const restored: DownloadFile[] = [];
    let failure: unknown;
    try {
      for (const operationId of operationIds) {
        const item = trashItems.find((entry) => entry.operationId === operationId);
        if (!item) continue;
        if (isTauri) await undoOperation(operationId);
        restored.push(trashFileSnapshot(item));
        trashItems = trashItems.filter((entry) => entry.operationId !== operationId);
        history = history.filter((entry) => entry.backendOperationId !== operationId);
      }
    } catch (error) { failure = error; }
    finally {
      files = [...restored.filter((restoredFile) => !files.some((file) => file.path === restoredFile.path)), ...files];
      trashBusy = false;
    }
    if (failure) notify(`${restored.length ? `Restored ${restored.length}; ` : ''}could not restore every file: ${failure}`);
    else if (restored.length) notify(`Restored ${restored.length} ${restored.length === 1 ? 'file' : 'files'} from Trash`);
  }

  async function recycleTrash(operationIds: number[]) {
    if (!operationIds.length || trashBusy) return;
    trashBusy = true;
    const recycled: number[] = [];
    let failure: unknown;
    try {
      for (const operationId of operationIds) {
        if (isTauri) await finalizeTrash(operationId);
        recycled.push(operationId);
        trashItems = trashItems.filter((entry) => entry.operationId !== operationId);
      }
    } catch (error) { failure = error; }
    finally {
      const recycledIds = new Set(recycled);
      history = history.map((entry) => recycledIds.has(entry.backendOperationId ?? -1) ? { ...entry, undoable: false, trashState: 'recycled' } : entry);
      trashBusy = false;
    }
    if (failure) notify(`${recycled.length ? `Recycled ${recycled.length}; ` : ''}could not move every file: ${failure}`);
    else if (recycled.length) notify(`Moved ${recycled.length} ${recycled.length === 1 ? 'file' : 'files'} to the Windows Recycle Bin`);
  }

  function reviewTrash(intent: 'review' | 'overview' | 'exit' = 'review') {
    trashIntent = intent;
    showTrash = true;
  }

  function requestOverview() {
    if (trashItems.length) reviewTrash('overview');
    else active = 'dashboard';
  }

  async function continueAfterTrash() {
    showTrash = false;
    if (trashIntent === 'overview') active = 'dashboard';
    if (trashIntent === 'exit' && isTauri) {
      allowWindowClose = true;
      await getCurrentWindow().close();
    }
  }

  async function revealFile(file: DownloadFile) {
    if (!isTauri) return notify(`File Explorer would reveal ${file.name}`);
    try { await revealDownload(file.path); } catch (error) { notify(`Could not show file: ${error}`); }
  }

  async function openFile(file: DownloadFile) {
    if (!isTauri) return notify(`${file.name} would open in its default Windows app`);
    try { await openDownload(file.path); } catch (error) { notify(`Could not open file: ${error}`); }
  }

  async function showRecycleBin() {
    if (!isTauri) return notify('The Windows Recycle Bin would open here');
    try { await openRecycleBin(); } catch (error) { notify(`Could not open the Recycle Bin: ${error}`); }
  }

  async function loadTextPreview(file: DownloadFile) {
    if (!isTauri) return file.extension === 'md' ? '# Sift preview\n\nMarkdown is rendered with **headings**, lists, links, quotes, and code blocks.\n\n- Local only\n- Up to 256 KB\n- Safe, escaped content\n\n> Your files never leave this computer.' : `This is example text shown in browser demo mode.\n\nThe installed Windows app reads up to 256 KB directly from the selected file, preserving line breaks and making more of the content visible without repeating the file name.`;
    return readTextPreview(file.path);
  }

  onMount(() => {
    const colourQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleSystemTheme = () => { if (theme === 'system') applyTheme('system'); };
    const greetingTimer = setInterval(() => currentHour = new Date().getHours(), 60_000);
    let removeCloseListener: (() => void) | undefined;
    colourQuery.addEventListener('change', handleSystemTheme);

    if (isTauri) {
      void getCurrentWindow().onCloseRequested((event) => {
        if (allowWindowClose || trashItems.length === 0) return;
        event.preventDefault();
        reviewTrash('exit');
      }).then((unlisten) => removeCloseListener = unlisten);
    }

    const storedTheme = localStorage.getItem('sift:theme');
    theme = storedTheme === 'light' || storedTheme === 'dark' ? storedTheme : 'system';
    applyTheme(theme);
    try { rememberedDestinations = JSON.parse(localStorage.getItem('sift:remembered-destinations') ?? '{}'); } catch { rememberedDestinations = {}; }
    try {
      const storedShortcuts = JSON.parse(localStorage.getItem('sift:shortcuts') ?? 'null');
      if (storedShortcuts && ['keep', 'trash', 'undo', 'fileAway'].every((action) => typeof storedShortcuts[action] === 'string') && new Set(Object.values(storedShortcuts)).size === 4) shortcuts = storedShortcuts;
    } catch { shortcuts = { ...DEFAULT_SHORTCUTS }; }

    void (async () => {
      let storedPinned: PinnedDestination[] | null = null;
      try {
        const parsed = JSON.parse(localStorage.getItem('sift:pinned-destinations') ?? 'null');
        if (Array.isArray(parsed) && parsed.every((item) => typeof item?.name === 'string' && typeof item?.path === 'string')) storedPinned = parsed.slice(0, 9);
      } catch { storedPinned = null; }
      if (storedPinned) pinnedDestinations = storedPinned;
      else if (isTauri) {
        try { pinnedDestinations = await getDefaultDestinations(); } catch { pinnedDestinations = []; }
      }
      if (isTauri) {
        try { userName = await getUserDisplayName(); } catch { userName = ''; }
        try { trashItems = await listTrash(); } catch { trashItems = []; }
        await scan();
      }
    })();

    return () => {
      colourQuery.removeEventListener('change', handleSystemTheme);
      clearInterval(greetingTimer);
      removeCloseListener?.();
    };
  });
</script>

<div class="app-shell">
  {#if active !== 'sift'}<Sidebar {active} onNavigate={(screen) => active = screen} onWatchedFolder={() => active = 'settings'} reviewCount={files.length} />{/if}
  <div class:sift-view={active === 'sift'} class="content-shell">
    {#if active === 'dashboard'}
      <main class="page"><Dashboard {files} {scanning} isDemo={!isTauri} {greeting} onScan={scan} onSift={() => active = 'sift'} onRules={() => active = 'rules'} onPreviewRules={() => active = 'rules'} /></main>
    {:else if active === 'sift'}
      <SiftMode {files} {pinnedDestinations} {shortcuts} trashCount={trashItems.length} {getSuggestions} onAction={siftAction} onPickDestination={pickDestination} onBack={requestOverview} onOpen={openFile} onReveal={revealFile} onUndo={undoLatest} onViewTrash={() => reviewTrash('review')} onLoadText={loadTextPreview} />
    {:else if active === 'rules'}
      <main class="page"><Rules {rules} {files} onUpdate={(next) => rules = next} onRun={runRules} /></main>
    {:else if active === 'history'}
      <main class="page"><History items={history} onUndo={undoItem} onUndoSession={undoSession} onOpenRecycleBin={showRecycleBin} /></main>
    {:else if active === 'settings'}
      <main class="page"><Settings {watchedFolder} {theme} {shortcuts} {pinnedDestinations} onFolderChange={(folder) => watchedFolder = folder} onPickFolder={pickWatchedFolder} onThemeChange={updateTheme} onShortcutsChange={updateShortcuts} onAddPinned={addPinnedDestination} onRemovePinned={removePinnedDestination} /></main>
    {/if}
  </div>
</div>

{#if toast}<div class="toast" role="status">{toast}</div>{/if}
{#if showTrash}<TrashReview items={trashItems} intent={trashIntent} busy={trashBusy} onRestore={restoreTrash} onRecycle={recycleTrash} onClose={() => showTrash = false} onContinue={continueAfterTrash} />{/if}
