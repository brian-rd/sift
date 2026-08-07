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
  import type {
    DownloadFile,
    HistoryItem,
    PinnedDestination,
    Rule,
    Screen,
    ShortcutBindings,
    SiftQueuePreferences,
    ThemePreference,
    TrashItem,
  } from './lib/types';
  import type { RuleMatch } from './lib/rules';
  import { demoFiles, demoHistory, demoRules } from './lib/demo';
  import {
    finalizeTrash,
    getDefaultDestinations,
    getUserDisplayName,
    isTauri,
    listTrash,
    moveDownload,
    openDownload,
    openRecycleBin,
    readTextPreview,
    revealDownload,
    scanDownloads,
    trashDownload,
    undoOperation,
  } from './lib/backend';
  import { DEFAULT_SHORTCUTS } from './lib/shortcuts';
  import { personalisedGreeting } from './lib/greeting';
  import { readStoredJson, writeStoredJson } from './lib/storage';
  import { DEFAULT_QUEUE_PREFERENCES, isSiftQueuePreferences } from './lib/fileQueue';

  const demoDestinations: PinnedDestination[] = [
    { name: 'Documents', path: 'Documents' },
    { name: 'Pictures', path: 'Pictures' },
    { name: 'Work', path: 'Documents / Work' },
    { name: 'Receipts', path: 'Documents / Receipts' },
  ];

  let active: Screen = 'dashboard';
  let files: DownloadFile[] = isTauri ? [] : demoFiles;
  let rules: Rule[] = isTauri ? [] : demoRules;
  let history: HistoryItem[] = isTauri ? [] : demoHistory;
  let pinnedDestinations: PinnedDestination[] = isTauri ? [] : demoDestinations;
  let shortcuts: ShortcutBindings = { ...DEFAULT_SHORTCUTS };
  let theme: ThemePreference = 'system';
  let watchEnabled = true;
  let trashImmediately = false;
  let autoplayMedia = false;
  let queuePreferences: SiftQueuePreferences = {
    ...DEFAULT_QUEUE_PREFERENCES,
    includedKinds: [...DEFAULT_QUEUE_PREFERENCES.includedKinds],
  };
  let rememberedDestinations: Record<string, PinnedDestination> = {};
  let watchedFolder = isTauri ? '' : 'C:\\Users\\you\\Downloads';
  let scanning = false;
  let toast = '';
  let toastTimer: ReturnType<typeof setTimeout>;
  let trashItems: TrashItem[] = [];
  let showTrash = false;
  let trashIntent: 'review' | 'overview' | 'exit' = 'review';
  let trashBusy = false;
  let closing = false;
  let userName = '';
  let currentHour = new Date().getHours();
  let knownFilePaths = new Set<string>();

  $: greeting = personalisedGreeting(currentHour, userName);

  const previewKinds = new Set(['image', 'pdf', 'video', 'audio']);

  function withPreview(file: DownloadFile, path = file.path): DownloadFile {
    return isTauri && previewKinds.has(file.kind) ? { ...file, previewUrl: convertFileSrc(path) } : file;
  }

  function isRuleList(value: unknown): value is Rule[] {
    return (
      Array.isArray(value) &&
      value.every((rule) => {
        if (!rule || typeof rule !== 'object') return false;
        const candidate = rule as Partial<Rule>;
        return (
          typeof candidate.id === 'string' &&
          typeof candidate.name === 'string' &&
          ['extension', 'contains', 'startsWith', 'endsWith', 'glob', 'regex', 'size', 'age'].includes(
            candidate.conditionType ?? '',
          ) &&
          typeof candidate.conditionValue === 'string' &&
          ['move', 'trash', 'ignore'].includes(candidate.actionType ?? '') &&
          (candidate.actionType !== 'move' ||
            (typeof candidate.destination === 'string' && Boolean(candidate.destination.trim()))) &&
          typeof candidate.enabled === 'boolean'
        );
      })
    );
  }

  function notify(message: string) {
    toast = message;
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => (toast = ''), 2600);
  }

  function applyTheme(preference: ThemePreference) {
    const isDark =
      preference === 'dark' ||
      (preference === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);
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
    writeStoredJson('sift:shortcuts', next);
  }

  function updateRules(next: Rule[]) {
    rules = next;
    writeStoredJson('sift:rules', next);
  }

  function updateWatchEnabled(enabled: boolean) {
    watchEnabled = enabled;
    localStorage.setItem('sift:watch-enabled', String(enabled));
    notify(enabled ? 'Watching for new files' : 'Automatic file watching paused');
  }

  function updateTrashImmediately(enabled: boolean) {
    trashImmediately = enabled;
    localStorage.setItem('sift:trash-immediately', String(enabled));
  }

  function updateAutoplayMedia(enabled: boolean) {
    autoplayMedia = enabled;
    localStorage.setItem('sift:autoplay-media', String(enabled));
  }

  function updateQueuePreferences(next: SiftQueuePreferences) {
    queuePreferences = next;
    writeStoredJson('sift:queue-preferences', next);
  }

  function updatePinnedDestinations(next: PinnedDestination[]) {
    pinnedDestinations = next.slice(0, 9);
    writeStoredJson('sift:pinned-destinations', pinnedDestinations);
  }

  function destinationName(path: string) {
    return (
      path
        .replace(/[\\/]+$/, '')
        .split(/[\\/]/)
        .pop() || path
    );
  }

  function getSuggestions(file: DownloadFile): PinnedDestination[] {
    const remembered = rememberedDestinations[file.extension.toLowerCase()];
    const paths = [remembered?.path, ...(file.suggestedFolders ?? []), file.suggestedFolder].filter(
      (path): path is string => Boolean(path),
    );
    return [...new Set(paths)].map(
      (path) =>
        pinnedDestinations.find((destination) => destination.path === path) ?? {
          name: destinationName(path),
          path,
        },
    );
  }

  function rememberDestination(file: DownloadFile, destination: PinnedDestination) {
    if (!file.extension) return;
    rememberedDestinations = { ...rememberedDestinations, [file.extension.toLowerCase()]: destination };
    writeStoredJson('sift:remembered-destinations', rememberedDestinations);
  }

  async function scan(options: { silent?: boolean } = {}) {
    if (scanning) return;
    scanning = true;
    if (!isTauri) {
      setTimeout(() => {
        scanning = false;
        if (!options.silent) notify(`Scan complete — ${files.length} files found`);
      }, 700);
      return;
    }
    try {
      const result = await scanDownloads(watchedFolder);
      const scannedFiles = result.files.map((file) => withPreview(file));
      const scannedPaths = new Set(scannedFiles.map((file) => file.path));
      const newFiles = scannedFiles.filter((file) => !knownFilePaths.has(file.path));
      files = options.silent
        ? scannedFiles.filter(
            (file) =>
              files.some((existing) => existing.path === file.path) ||
              newFiles.some((newFile) => newFile.path === file.path),
          )
        : scannedFiles;
      knownFilePaths = scannedPaths;
      watchedFolder = result.folder;
      if (options.silent) {
        if (newFiles.length)
          notify(`${newFiles.length} new ${newFiles.length === 1 ? 'file is' : 'files are'} ready to Sift`);
      } else {
        notify(
          `Scan complete — ${result.files.length} files found${result.skippedIncomplete ? `, ${result.skippedIncomplete} still downloading` : ''}`,
        );
      }
    } catch (error) {
      notify(`Could not scan folder: ${error}`);
    } finally {
      scanning = false;
    }
  }

  function record(
    file: DownloadFile,
    action: HistoryItem['action'],
    destination?: string,
    backendOperationId?: number,
    trashState?: HistoryItem['trashState'],
    undoable = true,
  ) {
    history = [
      {
        id: crypto.randomUUID(),
        fileName: file.name,
        action,
        destination,
        timestamp: Date.now(),
        session: 'Just now',
        undoable,
        backendOperationId,
        file,
        trashState,
      },
      ...history,
    ];
  }

  function addToTrash(file: DownloadFile, operationId: number, stagedPath: string) {
    trashItems = [
      {
        operationId,
        originalPath: file.path,
        stagedPath,
        name: file.name,
        extension: file.extension,
        size: file.size,
        modifiedAt: file.modifiedAt,
        createdAt: file.createdAt,
        kind: file.kind,
      },
      ...trashItems,
    ];
  }

  async function processTrash(
    file: DownloadFile,
  ): Promise<{ operationId: number; trashState: NonNullable<HistoryItem['trashState']> }> {
    const result = isTauri
      ? await trashDownload(file.path)
      : { operationId: Date.now(), source: file.path, destination: file.path };
    if (trashImmediately) {
      try {
        if (isTauri) await finalizeTrash(result.operationId);
        return { operationId: result.operationId, trashState: 'recycled' };
      } catch (error) {
        addToTrash(file, result.operationId, result.destination ?? file.path);
        notify(`Recycle Bin unavailable; ${file.name} is safe in Sift Trash for review`);
        return { operationId: result.operationId, trashState: 'staged' };
      }
    }
    addToTrash(file, result.operationId, result.destination ?? file.path);
    return { operationId: result.operationId, trashState: 'staged' };
  }

  async function siftAction(
    file: DownloadFile,
    action: 'trash' | 'keep' | 'fileAway',
    destination?: PinnedDestination,
  ): Promise<boolean> {
    try {
      let operationId: number | undefined;
      let trashState: HistoryItem['trashState'];
      let undoable = true;
      if (action === 'trash') {
        const result = await processTrash(file);
        operationId = result.operationId;
        trashState = result.trashState;
      }
      if (action === 'fileAway') {
        if (!destination) return false;
        if (isTauri) operationId = (await moveDownload(file.path, destination.path)).operationId;
        rememberDestination(file, destination);
      }
      const historyAction: HistoryItem['action'] =
        action === 'trash' ? 'Trashed' : action === 'keep' ? 'Kept' : 'Moved';
      record(
        file,
        historyAction,
        action === 'fileAway' ? destination?.path : undefined,
        operationId,
        trashState,
        undoable,
      );
      files = files.filter((item) => item.path !== file.path);
      return true;
    } catch (error) {
      notify(`Action failed: ${error}`);
      return false;
    }
  }

  async function undoLatest(): Promise<DownloadFile | null> {
    const item = history[0];
    if (!item && trashItems.length) return (await restoreTrash([trashItems[0].operationId]))[0] ?? null;
    if (!item) {
      notify('Nothing to undo');
      return null;
    }
    if (!item.undoable) {
      notify(
        item.action === 'Trashed'
          ? 'This older Trash entry cannot be matched to its Recycle Bin item'
          : 'The previous action cannot be undone',
      );
      return null;
    }
    return undoItem(item.id);
  }

  async function undoItem(id: string): Promise<DownloadFile | null> {
    const item = history.find((entry) => entry.id === id);
    if (!item) return null;
    if (item.action === 'Trashed') {
      const operationId = item.backendOperationId;
      if (!operationId) {
        notify('This Trash entry is missing its restore information.');
        return null;
      }
      if (item.trashState === 'staged' && trashItems.some((entry) => entry.operationId === operationId)) {
        return (await restoreTrash([operationId]))[0] ?? null;
      }
    }
    try {
      if (isTauri && item.backendOperationId) await undoOperation(item.backendOperationId);
    } catch (error) {
      notify(`Could not undo ${item.fileName}: ${error}`);
      return null;
    }
    history = history.filter((entry) => entry.id !== id);
    if (item.backendOperationId)
      trashItems = trashItems.filter((trashItem) => trashItem.operationId !== item.backendOperationId);
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
    } catch (error) {
      notify(`Session undo stopped: ${error}`);
      return;
    }
    history = history.filter((item) => item.session !== session || !item.undoable);
    const restoredIds = new Set(
      entries.map((item) => item.backendOperationId).filter((id): id is number => typeof id === 'number'),
    );
    trashItems = trashItems.filter((item) => !restoredIds.has(item.operationId));
    files = [
      ...restored.filter((restoredFile) => !files.some((file) => file.path === restoredFile.path)),
      ...files,
    ];
    notify(`${entries.length} session ${entries.length === 1 ? 'action' : 'actions'} restored`);
  }

  async function runRules(matches: RuleMatch[]) {
    const completed: string[] = [];
    for (const { file, rule } of matches) {
      try {
        let operationId: number | undefined;
        let trashState: HistoryItem['trashState'];
        if (rule.actionType === 'move') {
          if (!rule.destination) throw new Error(`Rule “${rule.name}” needs a destination`);
          if (isTauri) operationId = (await moveDownload(file.path, rule.destination)).operationId;
        }
        if (rule.actionType === 'trash') {
          const result = await processTrash(file);
          operationId = result.operationId;
          trashState = result.trashState;
        }
        const action: HistoryItem['action'] =
          rule.actionType === 'trash' ? 'Trashed' : rule.actionType === 'ignore' ? 'Kept' : 'Moved';
        record(
          file,
          action,
          rule.actionType === 'move' ? rule.destination : undefined,
          operationId,
          trashState,
        );
        completed.push(file.path);
      } catch (error) {
        notify(`Stopped at ${file.name}: ${error}`);
        break;
      }
    }
    files = files.filter((file) => !completed.includes(file.path));
    if (completed.length)
      notify(`${completed.length} ${completed.length === 1 ? 'file' : 'files'} processed safely`);
  }

  async function pickWatchedFolder() {
    if (!isTauri) return notify('Folder picker is available in the Windows app');
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Choose a folder for Sift to watch',
    });
    if (typeof selected === 'string') {
      watchedFolder = selected;
      await scan();
    }
  }

  async function pickDestination(file: DownloadFile): Promise<PinnedDestination | null> {
    if (!isTauri) {
      notify(`The Windows folder picker would open for ${file.name}`);
      return null;
    }
    const selected = await open({ directory: true, multiple: false, title: `File away ${file.name}` });
    return typeof selected === 'string' ? { name: destinationName(selected), path: selected } : null;
  }

  async function addPinnedDestination() {
    if (!isTauri) return notify('Pinned folders can be selected in the Windows app');
    const selected = await open({ directory: true, multiple: false, title: 'Pin a destination in Sift' });
    if (typeof selected !== 'string') return;
    if (pinnedDestinations.some((destination) => destination.path.toLowerCase() === selected.toLowerCase()))
      return notify('That destination is already pinned');
    updatePinnedDestinations([...pinnedDestinations, { name: destinationName(selected), path: selected }]);
  }

  function removePinnedDestination(destination: PinnedDestination) {
    updatePinnedDestinations(pinnedDestinations.filter((item) => item.path !== destination.path));
  }

  function trashFileSnapshot(item: TrashItem): DownloadFile {
    const file: DownloadFile = {
      path: item.originalPath,
      name: item.name,
      extension: item.extension,
      size: item.size,
      modifiedAt: item.modifiedAt,
      createdAt: item.createdAt,
      kind: item.kind,
    };
    return withPreview(file, item.originalPath);
  }

  async function restoreTrash(operationIds: number[]): Promise<DownloadFile[]> {
    if (!operationIds.length || trashBusy) return [];
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
    } catch (error) {
      failure = error;
    } finally {
      files = [
        ...restored.filter((restoredFile) => !files.some((file) => file.path === restoredFile.path)),
        ...files,
      ];
      trashBusy = false;
    }
    if (failure)
      notify(
        `${restored.length ? `Restored ${restored.length}; ` : ''}could not restore every file: ${failure}`,
      );
    else if (restored.length)
      notify(`Restored ${restored.length} ${restored.length === 1 ? 'file' : 'files'} from Trash`);
    return restored;
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
    } catch (error) {
      failure = error;
    } finally {
      const recycledIds = new Set(recycled);
      history = history.map((entry) =>
        recycledIds.has(entry.backendOperationId ?? -1)
          ? { ...entry, undoable: true, trashState: 'recycled' }
          : entry,
      );
      trashBusy = false;
    }
    if (failure)
      notify(
        `${recycled.length ? `Recycled ${recycled.length}; ` : ''}could not move every file: ${failure}`,
      );
    else if (recycled.length)
      notify(
        `Moved ${recycled.length} ${recycled.length === 1 ? 'file' : 'files'} to the Windows Recycle Bin`,
      );
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
    if (trashIntent === 'exit') await closeApp();
  }

  async function closeApp() {
    if (!isTauri || closing) return;
    closing = true;
    try {
      await getCurrentWindow().destroy();
    } catch (error) {
      closing = false;
      notify(`Could not close Sift: ${error}`);
    }
  }

  async function revealFile(file: DownloadFile) {
    if (!isTauri) return notify(`File Explorer would reveal ${file.name}`);
    try {
      await revealDownload(file.path);
    } catch (error) {
      notify(`Could not show file: ${error}`);
    }
  }

  async function openFile(file: DownloadFile) {
    if (!isTauri) return notify(`${file.name} would open in its default Windows app`);
    try {
      await openDownload(file.path);
    } catch (error) {
      notify(`Could not open file: ${error}`);
    }
  }

  async function showRecycleBin() {
    if (!isTauri) return notify('The Windows Recycle Bin would open here');
    try {
      await openRecycleBin();
    } catch (error) {
      notify(`Could not open the Recycle Bin: ${error}`);
    }
  }

  async function loadTextPreview(file: DownloadFile) {
    if (!isTauri)
      return file.extension === 'md'
        ? '# Sift preview\n\nMarkdown is rendered with **headings**, lists, links, quotes, and code blocks.\n\n- Local only\n- Up to 256 KB\n- Safe, escaped content\n\n> Your files never leave this computer.'
        : `This is example text shown in browser demo mode.\n\nThe installed Windows app reads up to 256 KB directly from the selected file, preserving line breaks and making more of the content visible without repeating the file name.`;
    return readTextPreview(file.path);
  }

  onMount(() => {
    const colourQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleSystemTheme = () => {
      if (theme === 'system') applyTheme('system');
    };
    const greetingTimer = setInterval(() => (currentHour = new Date().getHours()), 60_000);
    const watchTimer = setInterval(() => {
      if (isTauri && watchEnabled && !scanning) void scan({ silent: true });
    }, 3_000);
    let removeCloseListener: (() => void) | undefined;
    colourQuery.addEventListener('change', handleSystemTheme);

    if (isTauri) {
      void getCurrentWindow()
        .onCloseRequested((event) => {
          event.preventDefault();
          if (trashItems.length) reviewTrash('exit');
          else void closeApp();
        })
        .then((unlisten) => (removeCloseListener = unlisten));
    }

    const storedTheme = localStorage.getItem('sift:theme');
    theme = storedTheme === 'light' || storedTheme === 'dark' ? storedTheme : 'system';
    watchEnabled = localStorage.getItem('sift:watch-enabled') !== 'false';
    trashImmediately = localStorage.getItem('sift:trash-immediately') === 'true';
    autoplayMedia = localStorage.getItem('sift:autoplay-media') === 'true';
    queuePreferences = readStoredJson('sift:queue-preferences', queuePreferences, isSiftQueuePreferences);
    applyTheme(theme);
    rememberedDestinations = readStoredJson(
      'sift:remembered-destinations',
      {},
      (value): value is Record<string, PinnedDestination> => Boolean(value) && typeof value === 'object',
    );
    shortcuts = readStoredJson(
      'sift:shortcuts',
      { ...DEFAULT_SHORTCUTS },
      (value): value is ShortcutBindings => {
        if (!value || typeof value !== 'object') return false;
        const candidate = value as Record<string, unknown>;
        return (
          ['keep', 'trash', 'undo', 'fileAway'].every((action) => typeof candidate[action] === 'string') &&
          new Set(Object.values(candidate)).size === 4
        );
      },
    );
    rules = readStoredJson('sift:rules', isTauri ? [] : demoRules, isRuleList);

    void (async () => {
      const storedPinned =
        readStoredJson<PinnedDestination[] | null>(
          'sift:pinned-destinations',
          null,
          (value): value is PinnedDestination[] | null =>
            value === null ||
            (Array.isArray(value) &&
              value.every((item) => typeof item?.name === 'string' && typeof item?.path === 'string')),
        )?.slice(0, 9) ?? null;
      if (storedPinned) pinnedDestinations = storedPinned;
      else if (isTauri) {
        try {
          pinnedDestinations = await getDefaultDestinations();
        } catch {
          pinnedDestinations = [];
        }
      }
      if (isTauri) {
        try {
          userName = await getUserDisplayName();
        } catch {
          userName = '';
        }
        try {
          trashItems = await listTrash();
        } catch {
          trashItems = [];
        }
        await scan();
      }
    })();

    return () => {
      colourQuery.removeEventListener('change', handleSystemTheme);
      clearInterval(greetingTimer);
      clearInterval(watchTimer);
      removeCloseListener?.();
    };
  });
</script>

<div class="app-shell">
  {#if active !== 'sift'}<Sidebar
      {active}
      {watchEnabled}
      onNavigate={(screen) => (active = screen)}
      onWatchedFolder={() => (active = 'settings')}
      reviewCount={files.length}
    />{/if}
  <div class:sift-view={active === 'sift'} class="content-shell">
    {#if active === 'dashboard'}
      <main class="page">
        <Dashboard
          {files}
          {rules}
          {scanning}
          isDemo={!isTauri}
          {greeting}
          onScan={() => scan()}
          onSift={() => (active = 'sift')}
          onRules={() => (active = 'rules')}
          onPreviewRules={() => (active = 'rules')}
        />
      </main>
    {:else if active === 'sift'}
      <SiftMode
        {files}
        {pinnedDestinations}
        {shortcuts}
        {trashImmediately}
        {autoplayMedia}
        {queuePreferences}
        trashCount={trashItems.length}
        {getSuggestions}
        onAction={siftAction}
        onPickDestination={pickDestination}
        onBack={requestOverview}
        onOpen={openFile}
        onReveal={revealFile}
        onUndo={undoLatest}
        onViewTrash={() => reviewTrash('review')}
        onLoadText={loadTextPreview}
        onQueuePreferencesChange={updateQueuePreferences}
        onAddPinned={addPinnedDestination}
      />
    {:else if active === 'rules'}
      <main class="page"><Rules {rules} {files} onUpdate={updateRules} onRun={runRules} /></main>
    {:else if active === 'history'}
      <main class="page">
        <History
          items={history}
          onUndo={undoItem}
          onUndoSession={undoSession}
          onOpenRecycleBin={showRecycleBin}
        />
      </main>
    {:else if active === 'settings'}
      <main class="page">
        <Settings
          {watchedFolder}
          {watchEnabled}
          {trashImmediately}
          {autoplayMedia}
          {theme}
          {shortcuts}
          {pinnedDestinations}
          onFolderChange={(folder) => (watchedFolder = folder)}
          onPickFolder={pickWatchedFolder}
          onWatchEnabledChange={updateWatchEnabled}
          onTrashImmediatelyChange={updateTrashImmediately}
          onAutoplayMediaChange={updateAutoplayMedia}
          onThemeChange={updateTheme}
          onShortcutsChange={updateShortcuts}
          onAddPinned={addPinnedDestination}
          onRemovePinned={removePinnedDestination}
        />
      </main>
    {/if}
  </div>
</div>

{#if toast}<div class="toast" role="status">{toast}</div>{/if}
{#if showTrash}<TrashReview
    items={trashItems}
    intent={trashIntent}
    busy={trashBusy}
    onRestore={restoreTrash}
    onRecycle={recycleTrash}
    onClose={() => (showTrash = false)}
    onContinue={continueAfterTrash}
  />{/if}
