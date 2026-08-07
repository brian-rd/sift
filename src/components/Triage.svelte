<script lang="ts">
  import { onMount } from 'svelte';
  import { ArrowDown, ArrowLeft, ArrowRight, ArrowUp, Check, ChevronLeft, Clock3, ExternalLink, FolderInput, Keyboard, Trash2, Undo2 } from '@lucide/svelte';
  import type { DownloadFile } from '../lib/types';
  import { formatBytes, timeAgo } from '../lib/format';
  import FileIcon from './FileIcon.svelte';

  export let files: DownloadFile[];
  export let onAction: (file: DownloadFile, action: 'trash' | 'keep' | 'move' | 'later', destination?: string) => void | Promise<void>;
  export let onBack: () => void;
  export let onOpen: (file: DownloadFile) => void;
  export let onUndo: () => void;

  let processed = 0;
  let total = files.length;
  let announcement = '';
  let completed = false;
  $: current = files[0];
  $: progress = total ? Math.round((processed / total) * 100) : 100;

  async function act(action: 'trash' | 'keep' | 'move' | 'later') {
    if (!current) return;
    const labels = { trash: 'Moved to Trash', keep: 'Kept in Downloads', move: `Moved to ${current.suggestedFolder}`, later: 'Saved for later' };
    await onAction(current, action, current.suggestedFolder);
    processed += 1;
    announcement = `${current.name}: ${labels[action]}`;
    if (processed >= total) completed = true;
  }

  function handleKey(event: KeyboardEvent) {
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'z') { event.preventDefault(); onUndo(); return; }
    if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) return;
    const map: Record<string, () => void> = {
      ArrowLeft: () => act('trash'), ArrowDown: () => act('keep'), ArrowRight: () => act('move'), ArrowUp: () => act('later'),
      Enter: () => current && onOpen(current)
    };
    if (map[event.key]) { event.preventDefault(); map[event.key](); }
  }

  onMount(() => {
    window.addEventListener('keydown', handleKey);
    return () => window.removeEventListener('keydown', handleKey);
  });
</script>

<svelte:window on:keydown={(event) => { if (event.key === 'Escape') onBack(); }} />
<div class="triage-shell">
  <header class="triage-header">
    <button class="back" on:click={onBack}><ChevronLeft size={16} /> Overview</button>
    <div class="progress-meta"><strong>{Math.min(processed + 1, total)} of {total}</strong><div class="progress"><span style={`width:${progress}%`}></span></div></div>
    <button class="undo" on:click={onUndo}><Undo2 size={15} /> Undo <kbd>Ctrl Z</kbd></button>
  </header>

  {#if completed || !current}
    <section class="complete-card">
      <span class="complete-icon"><Check size={34} /></span>
      <p class="eyebrow">Queue cleared</p>
      <h1>That’s everything.</h1>
      <p>Your Downloads folder is sorted for now. Every action is available in History.</p>
      <button class="primary" on:click={onBack}>Back to overview</button>
    </section>
  {:else}
    <main class="triage-main">
      <section class="preview-card">
        <div class="preview-top">
          <span class="type-pill">{current.extension.toUpperCase()}</span>
          <button on:click={() => onOpen(current)} aria-label={`Open ${current.name}`}><ExternalLink size={16} /></button>
        </div>
        <div class="preview-content">
          {#if current.kind === 'image' && current.previewUrl}
            <img src={current.previewUrl} alt={`Preview of ${current.name}`} />
          {:else if current.kind === 'text'}
            <div class="text-preview"><span>MEETING NOTES</span><h3>Project checkpoint</h3><p>Review the open items from this week and prepare the next set of milestones.</p><p>Files and notes are kept together for quick reference...</p></div>
          {:else}
            <div class="generic-preview"><FileIcon kind={current.kind} extension={current.extension} size={54} /><strong>{current.extension.toUpperCase()}</strong><span>Preview available in the desktop app</span></div>
          {/if}
        </div>
      </section>

      <section class="file-details">
        <p class="eyebrow">Now reviewing</p>
        <h1>{current.name}</h1>
        <div class="metadata"><span>{formatBytes(current.size)}</span><i></i><span>{current.extension.toUpperCase()} file</span><i></i><span>{timeAgo(current.modifiedAt)}</span></div>
        {#if current.suggestedFolder}
          <div class="suggestion"><span class="suggestion-icon"><FolderInput size={19} /></span><div><span>Suggested destination</span><strong>{current.suggestedFolder}</strong>{#if current.matchedRule}<small>Based on “{current.matchedRule}” rule</small>{/if}</div></div>
        {/if}
        <div class="keyboard-hint"><Keyboard size={15} /><span>Use arrow keys to decide in seconds</span></div>
      </section>
    </main>

    <footer class="action-dock" aria-label="Triage actions">
      <button class="action trash" on:click={() => act('trash')}><span><ArrowLeft size={16} /></span><Trash2 size={18} /><div><strong>Trash</strong><small>Move to Recycle Bin</small></div></button>
      <button class="action keep" on:click={() => act('keep')}><span><ArrowDown size={16} /></span><Check size={18} /><div><strong>Keep here</strong><small>Leave in Downloads</small></div></button>
      <button class="action move" on:click={() => act('move')} disabled={!current.suggestedFolder}><span><ArrowRight size={16} /></span><FolderInput size={18} /><div><strong>Move</strong><small>{current.suggestedFolder ?? 'No suggestion'}</small></div></button>
      <button class="action later" on:click={() => act('later')}><span><ArrowUp size={16} /></span><Clock3 size={18} /><div><strong>Later</strong><small>Return to this file</small></div></button>
    </footer>
  {/if}
  <div class="sr-only" aria-live="polite">{announcement}</div>
</div>

<style>
  .triage-shell{min-height:100%;display:flex;flex-direction:column}.triage-header{height:54px;display:grid;grid-template-columns:1fr auto 1fr;align-items:center;border-bottom:1px solid var(--border);padding:0 24px}.back,.undo{display:flex;align-items:center;gap:6px;border:0;background:transparent;color:var(--text-2);font:600 11px var(--font-ui);cursor:pointer}.undo{justify-self:end}.undo kbd{margin-left:4px}.progress-meta{display:flex;align-items:center;gap:10px;font-size:10px;color:var(--text-3)}.progress-meta strong{color:var(--text-2)}.progress{width:130px;height:4px;border-radius:4px;background:#deded9;overflow:hidden}.progress span{display:block;height:100%;background:var(--accent);transition:width .25s}.triage-main{flex:1;display:grid;grid-template-columns:minmax(320px,1.25fr) minmax(300px,.75fr);gap:48px;align-items:center;max-width:1040px;width:100%;margin:0 auto;padding:40px 46px 28px}.preview-card{height:min(54vh,540px);min-height:340px;background:#e9e8e3;border-radius:16px;position:relative;overflow:hidden;box-shadow:inset 0 0 0 1px rgba(28,28,25,.04)}.preview-top{position:absolute;z-index:3;top:14px;left:14px;right:14px;display:flex;justify-content:space-between}.type-pill{padding:5px 8px;border-radius:6px;background:rgba(255,255,255,.88);font-size:9px;font-weight:750;letter-spacing:.08em}.preview-top button{width:32px;height:32px;display:grid;place-items:center;border:0;border-radius:8px;background:rgba(255,255,255,.88);cursor:pointer;color:var(--text-2)}.preview-content{height:100%;display:grid;place-items:center;padding:0}.preview-content img{width:100%;height:100%;object-fit:cover}.generic-preview{display:flex;flex-direction:column;align-items:center;gap:12px;color:var(--text-3)}.generic-preview :global(.file-icon){width:92px;height:92px;border-radius:18px}.generic-preview strong{font:650 17px var(--font-display);color:var(--text-2)}.generic-preview span{font-size:10px}.text-preview{width:70%;min-height:68%;background:#fff;padding:42px;box-shadow:0 12px 28px rgba(20,20,18,.08);transform:rotate(-1deg)}.text-preview span{font-size:8px;letter-spacing:.13em;color:var(--text-3)}.text-preview h3{font:650 21px var(--font-display);margin:18px 0}.text-preview p{font:12px/1.8 Georgia,serif;color:#65655e}.file-details .eyebrow,.complete-card .eyebrow{font-size:9px;letter-spacing:.13em;text-transform:uppercase;font-weight:750;color:var(--accent);margin:0 0 9px}.file-details h1,.complete-card h1{font:650 29px/1.15 var(--font-display);letter-spacing:-.035em;margin:0;overflow-wrap:anywhere}.metadata{display:flex;align-items:center;gap:8px;color:var(--text-3);font-size:10px;margin:12px 0 28px}.metadata i{width:3px;height:3px;border-radius:50%;background:#b6b5ae}.suggestion{display:flex;gap:12px;align-items:center;padding:14px;border:1px solid #d7dfd5;background:#f2f7f2;border-radius:11px}.suggestion-icon{width:38px;height:38px;display:grid;place-items:center;background:#dfece1;color:#4a7454;border-radius:9px}.suggestion div{display:flex;flex-direction:column}.suggestion span{font-size:9px;color:var(--text-3)}.suggestion strong{font-size:12px;margin:3px 0}.suggestion small{font-size:9px;color:#6d806f}.keyboard-hint{display:flex;align-items:center;gap:7px;color:var(--text-3);font-size:10px;margin-top:23px}.action-dock{display:grid;grid-template-columns:repeat(4,minmax(140px,1fr));gap:8px;padding:10px 18px 14px;border-top:1px solid var(--border);background:rgba(244,243,239,.92);backdrop-filter:blur(12px)}.action{height:61px;position:relative;display:flex;align-items:center;justify-content:center;gap:10px;border:1px solid var(--border);border-radius:10px;background:#fff;color:var(--text-2);cursor:pointer}.action:hover{border-color:#a9aaa4;transform:translateY(-1px)}.action>span{position:absolute;left:8px;top:8px;width:20px;height:20px;display:grid;place-items:center;border-radius:5px;background:#f0f0ed;color:var(--text-3)}.action div{display:flex;flex-direction:column;text-align:left}.action strong{font-size:11px}.action small{font-size:8px;color:var(--text-3);margin-top:2px}.action.trash:hover{border-color:#cfa89f;color:#9b4d3d}.action.move{background:var(--ink);color:#fff;border-color:var(--ink)}.action.move>span{background:rgba(255,255,255,.12);color:#fff}.action.move small{color:#b7b7b0}.action:disabled{opacity:.45;cursor:not-allowed}.complete-card{margin:auto;text-align:center;max-width:380px}.complete-icon{width:76px;height:76px;border-radius:50%;display:grid;place-items:center;background:#e1ede3;color:#4a7553;margin:0 auto 24px}.complete-card p:not(.eyebrow){color:var(--text-2);font-size:13px;line-height:1.6}.complete-card .primary{border:0;border-radius:8px;background:var(--ink);color:#fff;padding:11px 16px;font:650 12px var(--font-ui);cursor:pointer;margin-top:12px}@media(max-width:800px){.triage-main{grid-template-columns:1fr;gap:24px;padding:25px}.preview-card{height:42vh}.action-dock{grid-template-columns:repeat(2,1fr)}.keyboard-hint{display:none}}@media(max-width:560px){.triage-header{padding:0 12px}.progress{width:70px}.undo kbd{display:none}.triage-main{padding:18px 14px}.preview-card{min-height:260px}.file-details h1{font-size:23px}.action-dock{padding-bottom:72px}.action small{display:none}.action{height:50px}}
</style>
