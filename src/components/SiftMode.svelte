<script lang="ts">
  import { ArrowDown, ArrowLeft, ArrowRight, ArrowUp, Check, ChevronLeft, ExternalLink, FolderInput, Keyboard, RotateCcw, Trash2, Undo2, X } from '@lucide/svelte';
  import type { DownloadFile, PinnedDestination, ShortcutBindings } from '../lib/types';
  import { formatBytes, formatDate } from '../lib/format';
  import FileIcon from './FileIcon.svelte';

  export let files: DownloadFile[];
  export let pinnedDestinations: PinnedDestination[];
  export let shortcuts: ShortcutBindings;
  export let getSuggestions: (file: DownloadFile) => PinnedDestination[];
  export let onAction: (file: DownloadFile, action: 'trash' | 'keep' | 'fileAway', destination?: PinnedDestination) => Promise<boolean>;
  export let onPickDestination: (file: DownloadFile) => Promise<PinnedDestination | null>;
  export let onBack: () => void;
  export let onOpen: (file: DownloadFile) => void;
  export let onUndo: () => Promise<DownloadFile | null>;

  let processed = 0;
  let total = files.length;
  let announcement = '';
  let restoredNotice = '';
  let completed = false;
  let busy = false;
  let showDestinations = false;
  let destinationOptions: PinnedDestination[] = [];
  let noticeTimer: ReturnType<typeof setTimeout>;

  $: current = files[0];
  $: progress = total ? Math.round((processed / total) * 100) : 100;
  $: currentSuggestions = current ? getSuggestions(current) : [];

  const keyLabel = (code: string) => ({ ArrowUp: '↑', ArrowDown: '↓', ArrowLeft: '←', ArrowRight: '→' })[code] ?? code.replace(/^Key/, '');

  async function commit(action: 'trash' | 'keep' | 'fileAway', destination?: PinnedDestination) {
    if (!current || busy) return;
    busy = true;
    const file = current;
    const succeeded = await onAction(file, action, destination);
    busy = false;
    if (!succeeded) return;
    processed += 1;
    const label = action === 'trash' ? 'Moved to Recycle Bin' : action === 'keep' ? 'Kept in Downloads' : `Filed in ${destination?.name}`;
    announcement = `${file.name}: ${label}`;
    showDestinations = false;
    if (processed >= total) completed = true;
  }

  async function fileAway() {
    if (!current || busy) return;
    const suggestions = getSuggestions(current);
    if (suggestions.length === 1) return commit('fileAway', suggestions[0]);
    if (suggestions.length > 1) {
      destinationOptions = suggestions;
      showDestinations = true;
      return;
    }
    const destination = await onPickDestination(current);
    if (destination) await commit('fileAway', destination);
  }

  async function undo() {
    if (busy) return;
    busy = true;
    const restored = await onUndo();
    busy = false;
    if (!restored) return;
    processed = Math.max(0, processed - 1);
    completed = false;
    restoredNotice = `Restored ${restored.name}`;
    announcement = restoredNotice;
    clearTimeout(noticeTimer);
    noticeTimer = setTimeout(() => restoredNotice = '', 2200);
  }

  function handleKey(event: KeyboardEvent) {
    if (event.repeat || busy || showDestinations) return;
    if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement || event.target instanceof HTMLSelectElement) return;
    const actions: Record<string, () => void> = {
      [shortcuts.keep]: () => void commit('keep'),
      [shortcuts.trash]: () => void commit('trash'),
      [shortcuts.undo]: () => void undo(),
      [shortcuts.fileAway]: () => void fileAway()
    };
    if (actions[event.code]) { event.preventDefault(); actions[event.code](); return; }
    if (/^Digit[1-9]$/.test(event.code)) {
      const destination = pinnedDestinations[Number(event.code.slice(-1)) - 1];
      if (destination) { event.preventDefault(); void commit('fileAway', destination); }
      return;
    }
    if ((event.ctrlKey || event.metaKey) && event.code === 'KeyZ') { event.preventDefault(); void undo(); }
    if (event.code === 'Enter' && current) { event.preventDefault(); onOpen(current); }
  }
</script>

<svelte:window on:keydown={handleKey} />

<div class="sift-shell">
  <header class="sift-header">
    <button class="back" on:click={onBack}><ChevronLeft size={16} /> Overview</button>
    <div class="progress-meta">
      <div class="progress-copy"><strong>{Math.min(processed + 1, total)} of {total}</strong><span>{progress}% complete</span></div>
      <div class="progress" role="progressbar" aria-label="Sifting progress" aria-valuenow={progress} aria-valuemin="0" aria-valuemax="100"><span style={`width:${progress}%`}></span></div>
    </div>
    <button class="undo-top" on:click={undo} disabled={busy}><Undo2 size={15} /> Undo <kbd>{keyLabel(shortcuts.undo)}</kbd></button>
  </header>

  {#if restoredNotice}<div class="restore-toast" role="status"><RotateCcw size={15} /> {restoredNotice}</div>{/if}

  {#if completed || !current}
    <section class="complete-card">
      <span class="complete-icon"><Check size={34} /></span>
      <p class="eyebrow">Sift complete</p>
      <h1>Downloads cleared.</h1>
      <p>Every action is recorded in History and can be undone.</p>
      <button class="primary" on:click={onBack}>Back to overview</button>
    </section>
  {:else}
    <main class="sift-main">
      <section class="preview-card">
        <div class="preview-top">
          <span class="type-pill">{current.extension.toUpperCase() || 'FILE'}</span>
          <button on:click={() => onOpen(current)} aria-label={`Show ${current.name} in File Explorer`}><ExternalLink size={16} /></button>
        </div>
        <div class="preview-content">
          {#if current.kind === 'image' && current.previewUrl}
            <img src={current.previewUrl} alt={`Preview of ${current.name}`} />
          {:else if current.kind === 'text'}
            <div class="text-preview"><span>TEXT PREVIEW</span><h3>{current.name}</h3><p>Preview content is loading from the selected file.</p></div>
          {:else}
            <div class="generic-preview"><FileIcon kind={current.kind} extension={current.extension} size={54} /><strong>{current.extension.toUpperCase() || 'FILE'}</strong><span>No inline preview for this file type</span></div>
          {/if}
        </div>
      </section>

      <section class="file-details">
        <p class="eyebrow">Now sifting</p>
        <h1>{current.name}</h1>
        <dl class="metadata-grid">
          <div><dt>Size</dt><dd>{formatBytes(current.size)}</dd></div>
          <div><dt>File type</dt><dd>{current.extension.toUpperCase() || 'Unknown'}</dd></div>
          <div><dt>Date modified</dt><dd>{formatDate(current.modifiedAt)}</dd></div>
          <div><dt>Date created</dt><dd>{formatDate(current.createdAt)}</dd></div>
        </dl>

        {#if currentSuggestions.length > 0}
          <div class="suggestion"><span class="suggestion-icon"><FolderInput size={19} /></span><div><span>{currentSuggestions.length === 1 ? 'Suggested destination' : `${currentSuggestions.length} suggested destinations`}</span><strong>{currentSuggestions[0].name}</strong>{#if current.matchedRule}<small>Based on “{current.matchedRule}” rule</small>{/if}</div></div>
        {:else}
          <div class="suggestion empty"><span class="suggestion-icon"><FolderInput size={19} /></span><div><span>No suggestion yet</span><strong>File Away will open the folder picker</strong></div></div>
        {/if}

        <div class="pinned">
          <span>Pinned destinations</span>
          <div>{#each pinnedDestinations as destination, index}<button on:click={() => commit('fileAway', destination)} disabled={busy}><kbd>{index + 1}</kbd>{destination.name}</button>{/each}</div>
        </div>
        <div class="keyboard-hint"><Keyboard size={15} /><span>Use your arrow keys to sift without slowing down</span></div>
      </section>
    </main>

    <footer class="action-dock" aria-label="Sift actions">
      <button class="action keep" on:click={() => commit('keep')} disabled={busy}><span>{keyLabel(shortcuts.keep)}</span><ArrowUp size={18} /><div><strong>Keep here</strong><small>Leave in Downloads</small></div></button>
      <button class="action trash" on:click={() => commit('trash')} disabled={busy}><span>{keyLabel(shortcuts.trash)}</span><Trash2 size={18} /><div><strong>Trash</strong><small>Move to Recycle Bin</small></div></button>
      <button class="action undo-action" on:click={undo} disabled={busy}><span>{keyLabel(shortcuts.undo)}</span><RotateCcw size={18} /><div><strong>Undo</strong><small>Restore the last file</small></div></button>
      <button class="action file-away" on:click={fileAway} disabled={busy}><span>{keyLabel(shortcuts.fileAway)}</span><FolderInput size={18} /><div><strong>File Away</strong><small>{currentSuggestions.length === 1 ? currentSuggestions[0].name : currentSuggestions.length > 1 ? 'Choose a suggestion' : 'Choose a folder'}</small></div></button>
    </footer>
  {/if}

  {#if showDestinations}
    <div class="overlay" role="presentation" on:click={(event) => event.currentTarget === event.target && (showDestinations = false)}>
      <div class="destination-dialog" role="dialog" aria-modal="true" aria-labelledby="destination-title">
        <header><div><p>Quick destination</p><h2 id="destination-title">Where should this file go?</h2></div><button on:click={() => showDestinations = false} aria-label="Close destinations"><X size={18} /></button></header>
        <div class="destination-list">{#each destinationOptions as destination, index}<button on:click={() => commit('fileAway', destination)}><span>{index + 1}</span><FolderInput size={18} /><div><strong>{destination.name}</strong><small>{destination.path}</small></div><ArrowRight size={16} /></button>{/each}</div>
        <button class="choose-other" on:click={async () => { const destination = current && await onPickDestination(current); if (destination) await commit('fileAway', destination); }}>Choose another folder</button>
      </div>
    </div>
  {/if}
  <div class="sr-only" aria-live="polite">{announcement}</div>
</div>

<style>
  .sift-shell{min-height:100%;display:flex;flex-direction:column;background:var(--bg)}.sift-header{min-height:70px;display:grid;grid-template-columns:1fr auto 1fr;align-items:center;border-bottom:1px solid var(--border);padding:9px 28px;background:var(--bg)}.back,.undo-top{display:flex;align-items:center;gap:7px;border:0;background:transparent;color:var(--text-2);font:600 11px var(--font-ui);cursor:pointer}.undo-top{justify-self:end}.undo-top:disabled{opacity:.45}.progress-meta{display:flex;align-items:center;gap:15px}.progress-copy{display:flex;flex-direction:column;align-items:flex-end;font-size:9px;color:var(--text-3)}.progress-copy strong{font-size:11px;color:var(--ink)}.progress{width:min(34vw,420px);height:8px;border-radius:999px;background:var(--surface-3);overflow:hidden}.progress span{display:block;height:100%;border-radius:inherit;background:var(--accent);transition:width .2s}.sift-main{flex:1;display:grid;grid-template-columns:minmax(360px,1.22fr) minmax(340px,.78fr);gap:46px;align-items:center;max-width:1120px;width:100%;margin:0 auto;padding:34px 44px 26px}.preview-card{height:min(56vh,560px);min-height:360px;background:var(--surface-2);border:1px solid var(--border);border-radius:16px;position:relative;overflow:hidden}.preview-top{position:absolute;z-index:3;top:14px;left:14px;right:14px;display:flex;justify-content:space-between}.type-pill,.preview-top button{background:var(--surface-overlay);color:var(--ink);backdrop-filter:blur(8px)}.type-pill{padding:5px 8px;border-radius:6px;font-size:9px;font-weight:750;letter-spacing:.08em}.preview-top button{width:34px;height:34px;display:grid;place-items:center;border:1px solid var(--border);border-radius:8px;cursor:pointer}.preview-content{height:100%;display:grid;place-items:center}.preview-content img{width:100%;height:100%;object-fit:cover}.generic-preview{display:flex;flex-direction:column;align-items:center;gap:12px;color:var(--text-3)}.generic-preview :global(.file-icon){width:92px;height:92px;border-radius:18px}.generic-preview strong{font:650 17px var(--font-display);color:var(--text-2)}.generic-preview span{font-size:11px}.text-preview{width:72%;min-height:68%;background:var(--surface);padding:42px;box-shadow:var(--shadow-lg)}.text-preview span{font-size:8px;letter-spacing:.13em;color:var(--text-3)}.text-preview h3{font:650 21px var(--font-display);margin:18px 0}.text-preview p{font:12px/1.8 Georgia,serif;color:var(--text-2)}.file-details .eyebrow,.complete-card .eyebrow{font-size:9px;letter-spacing:.13em;text-transform:uppercase;font-weight:750;color:var(--accent);margin:0 0 9px}.file-details h1,.complete-card h1{font:650 28px/1.18 var(--font-display);letter-spacing:-.035em;margin:0;overflow-wrap:anywhere}.metadata-grid{display:grid;grid-template-columns:1fr 1fr;gap:1px;margin:20px 0;background:var(--border);border:1px solid var(--border);border-radius:10px;overflow:hidden}.metadata-grid div{padding:11px 12px;background:var(--surface)}.metadata-grid dt{font-size:8px;text-transform:uppercase;letter-spacing:.08em;color:var(--text-3);font-weight:700}.metadata-grid dd{margin:4px 0 0;font-size:11px;font-weight:650;color:var(--ink)}.suggestion{display:flex;gap:12px;align-items:center;padding:13px;border:1px solid var(--success-border);background:var(--success-bg);border-radius:10px}.suggestion.empty{border-style:dashed;background:var(--surface-2);border-color:var(--border-strong)}.suggestion-icon{width:38px;height:38px;display:grid;place-items:center;background:var(--success-strong);color:var(--success-text);border-radius:9px}.suggestion div{display:flex;flex-direction:column}.suggestion span{font-size:9px;color:var(--text-3)}.suggestion strong{font-size:11px;margin:3px 0}.suggestion small{font-size:9px;color:var(--success-text)}.pinned{margin-top:17px}.pinned>span{font-size:8px;text-transform:uppercase;letter-spacing:.09em;color:var(--text-3);font-weight:700}.pinned>div{display:flex;gap:6px;flex-wrap:wrap;margin-top:7px}.pinned button{height:30px;display:flex;align-items:center;gap:6px;padding:0 9px;border:1px solid var(--border);border-radius:7px;background:var(--surface);color:var(--text-2);font:600 9px var(--font-ui);cursor:pointer}.pinned button:hover{border-color:var(--border-strong);color:var(--ink)}.pinned kbd{font-size:8px}.keyboard-hint{display:flex;align-items:center;gap:7px;color:var(--text-3);font-size:10px;margin-top:16px}.action-dock{display:grid;grid-template-columns:repeat(4,minmax(150px,1fr));gap:8px;padding:10px 18px 14px;border-top:1px solid var(--border);background:var(--dock-bg);backdrop-filter:blur(12px)}.action{height:64px;position:relative;display:flex;align-items:center;justify-content:center;gap:10px;border:1px solid var(--border);border-radius:10px;background:var(--surface);color:var(--text-2);cursor:pointer;transition:border-color .16s,background .16s,color .16s}.action:hover{border-color:var(--border-strong);background:var(--surface-2)}.action>span{position:absolute;left:8px;top:8px;min-width:21px;height:21px;padding:0 5px;display:grid;place-items:center;border-radius:5px;background:var(--surface-3);color:var(--text-3);font:600 11px var(--font-mono)}.action div{display:flex;flex-direction:column;text-align:left}.action strong{font-size:11px}.action small{font-size:8px;color:var(--text-3);margin-top:2px}.action.trash:hover{border-color:var(--danger-border);color:var(--danger-text)}.action.file-away{background:var(--ink);color:var(--bg);border-color:var(--ink)}.action.file-away>span{background:rgba(255,255,255,.13);color:#fff}.action.file-away small{color:var(--primary-muted)}.action:disabled{opacity:.48;cursor:not-allowed}.restore-toast{position:fixed;z-index:30;top:82px;left:50%;transform:translateX(-50%);display:flex;align-items:center;gap:7px;padding:9px 12px;border-radius:8px;background:var(--success-text);color:#fff;font-size:10px;box-shadow:var(--shadow-lg)}.complete-card{margin:auto;text-align:center;max-width:380px}.complete-icon{width:76px;height:76px;border-radius:50%;display:grid;place-items:center;background:var(--success-bg);color:var(--success-text);margin:0 auto 24px}.complete-card>p:not(.eyebrow){color:var(--text-2);font-size:13px;line-height:1.6}.primary{border:0;border-radius:8px;background:var(--ink);color:var(--bg);padding:11px 16px;font:650 12px var(--font-ui);cursor:pointer;margin-top:12px}.overlay{position:fixed;inset:0;z-index:100;display:grid;place-items:center;padding:24px;background:rgba(10,11,9,.58);backdrop-filter:blur(3px)}.destination-dialog{width:min(480px,100%);padding:22px;background:var(--surface);border:1px solid var(--border);border-radius:15px;box-shadow:var(--shadow-lg)}.destination-dialog>header{display:flex;justify-content:space-between;align-items:flex-start;margin-bottom:16px}.destination-dialog header p{margin:0 0 4px;color:var(--accent);font-size:8px;text-transform:uppercase;letter-spacing:.11em;font-weight:750}.destination-dialog h2{margin:0;font:650 20px var(--font-display)}.destination-dialog header button{width:34px;height:34px;border:0;border-radius:8px;background:var(--surface-2);color:var(--text-2);display:grid;place-items:center;cursor:pointer}.destination-list{display:flex;flex-direction:column;gap:6px}.destination-list>button{min-height:58px;display:grid;grid-template-columns:24px 22px 1fr auto;gap:9px;align-items:center;padding:8px 10px;border:1px solid var(--border);border-radius:9px;background:var(--surface);color:var(--text-2);text-align:left;cursor:pointer}.destination-list>button:hover{border-color:var(--accent);background:var(--surface-2)}.destination-list>button>span{width:22px;height:22px;display:grid;place-items:center;border-radius:5px;background:var(--surface-3);font-size:9px}.destination-list div{display:flex;flex-direction:column;min-width:0}.destination-list strong{font-size:11px;color:var(--ink)}.destination-list small{font-size:8px;color:var(--text-3);white-space:nowrap;overflow:hidden;text-overflow:ellipsis}.choose-other{width:100%;height:36px;margin-top:10px;border:1px dashed var(--border-strong);border-radius:8px;background:transparent;color:var(--text-2);font:600 10px var(--font-ui);cursor:pointer}@media(max-width:850px){.sift-main{grid-template-columns:1fr;gap:22px;padding:24px}.preview-card{height:40vh}.action-dock{grid-template-columns:repeat(2,1fr)}.keyboard-hint{display:none}}@media(max-width:600px){.sift-header{padding:8px 12px}.progress{width:34vw}.progress-copy span,.undo-top kbd{display:none}.sift-main{padding:18px 14px}.preview-card{min-height:250px}.file-details h1{font-size:22px}.action-dock{padding-bottom:72px}.action small{display:none}.action{height:52px}}
</style>
