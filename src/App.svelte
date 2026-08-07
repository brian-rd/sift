<script lang="ts">
  import { onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import Sidebar from './components/Sidebar.svelte';
  import Dashboard from './components/Dashboard.svelte';
  import Triage from './components/Triage.svelte';
  import Rules from './components/Rules.svelte';
  import History from './components/History.svelte';
  import Settings from './components/Settings.svelte';
  import type { DownloadFile, HistoryItem, Rule, Screen } from './lib/types';
  import type { RuleMatch } from './lib/rules';
  import { demoFiles, demoHistory, demoRules } from './lib/demo';
  import { isTauri, moveDownload, revealDownload, scanDownloads, trashDownload, undoOperation } from './lib/backend';

  let active: Screen = 'dashboard';
  let files: DownloadFile[] = isTauri ? [] : demoFiles;
  let rules: Rule[] = isTauri ? [] : demoRules;
  let history: HistoryItem[] = isTauri ? [] : demoHistory;
  let watchedFolder = isTauri ? '' : 'C:\\Users\\you\\Downloads';
  let scanning = false;
  let toast = '';
  let toastTimer: ReturnType<typeof setTimeout>;

  function notify(message: string) { toast = message; clearTimeout(toastTimer); toastTimer = setTimeout(() => toast = '', 2600); }

  async function scan() {
    scanning = true;
    if (!isTauri) { setTimeout(() => { scanning = false; notify('Scan complete — 12 files found'); }, 700); return; }
    try {
      const result = await scanDownloads(watchedFolder);
      files = result.files;
      watchedFolder = result.folder;
      notify(`Scan complete — ${result.files.length} files found${result.skippedIncomplete ? `, ${result.skippedIncomplete} still downloading` : ''}`);
    } catch (error) { notify(`Could not scan folder: ${error}`); }
    finally { scanning = false; }
  }

  function record(file: DownloadFile, action: HistoryItem['action'], destination?: string, backendOperationId?: number) {
    history = [{ id: crypto.randomUUID(), fileName: file.name, action, destination, timestamp: Date.now(), session: 'Just now', undoable: action !== 'Kept', backendOperationId }, ...history];
  }

  async function triageAction(file: DownloadFile, action: 'trash' | 'keep' | 'move' | 'later', destination?: string) {
    const labels = { trash: 'Trashed', keep: 'Kept', move: 'Moved', later: 'Review later' } as const;
    try {
      let operationId: number | undefined;
      if (isTauri && action === 'trash') operationId = (await trashDownload(file.path)).operationId;
      if (isTauri && action === 'move' && destination) operationId = (await moveDownload(file.path, destination)).operationId;
      record(file, labels[action], action === 'move' ? destination : undefined, operationId);
    } catch (error) {
      notify(`Action failed: ${error}`);
      return;
    }
    files = files.filter((item) => item.path !== file.path);
  }

  async function undoLatest() {
    const item = history.find((entry) => entry.undoable);
    if (!item) return notify('Nothing to undo');
    await undoItem(item.id);
  }

  async function undoItem(id: string) {
    const item = history.find((entry) => entry.id === id);
    if (!item) return;
    try {
      if (isTauri && item.backendOperationId) await undoOperation(item.backendOperationId);
    } catch (error) { notify(`Could not undo ${item.fileName}: ${error}`); return; }
    history = history.filter((entry) => entry.id !== id);
    notify(`Restored “${item.fileName}”`);
  }

  async function undoSession(session: string) {
    const entries = history.filter((item) => item.session === session && item.undoable);
    try {
      for (const item of entries) {
        if (isTauri && item.backendOperationId) await undoOperation(item.backendOperationId);
      }
    } catch (error) { notify(`Session undo stopped: ${error}`); return; }
    history = history.filter((item) => item.session !== session || !item.undoable);
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

  async function pickFolder() {
    if (!isTauri) return notify('Folder picker is available in the desktop build');
    const selected = await open({ directory: true, multiple: false, title: 'Choose a folder for Sift to watch' });
    if (typeof selected === 'string') { watchedFolder = selected; await scan(); }
  }

  async function openFile(file: DownloadFile) {
    if (!isTauri) return notify(`Desktop preview: ${file.name}`);
    try { await revealDownload(file.path); } catch (error) { notify(`Could not open file: ${error}`); }
  }

  onMount(() => { if (isTauri) void scan(); });
</script>

<div class="app-shell">
  {#if active !== 'triage'}<Sidebar {active} onNavigate={(screen) => active = screen} reviewCount={files.length} />{/if}
  <div class:triage-view={active === 'triage'} class="content-shell">
    {#if active === 'dashboard'}
      <main class="page"><Dashboard {files} {scanning} isDemo={!isTauri} onScan={scan} onTriage={() => active = 'triage'} onRules={() => active = 'rules'} onPreviewRules={() => active = 'rules'} /></main>
    {:else if active === 'triage'}
      <Triage {files} onAction={triageAction} onBack={() => active = 'dashboard'} onOpen={openFile} onUndo={undoLatest} />
    {:else if active === 'rules'}
      <main class="page"><Rules {rules} {files} onUpdate={(next) => rules = next} onRun={runRules} /></main>
    {:else if active === 'history'}
      <main class="page"><History items={history} onUndo={undoItem} onUndoSession={undoSession} /></main>
    {:else if active === 'settings'}
      <main class="page"><Settings {watchedFolder} onFolderChange={(folder) => watchedFolder = folder} onPickFolder={pickFolder} /></main>
    {/if}
  </div>
</div>

{#if toast}<div class="toast" role="status">{toast}</div>{/if}
