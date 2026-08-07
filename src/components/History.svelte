<script lang="ts">
  import { ArchiveRestore, Check, Clock3, ExternalLink, FileInput, FolderInput, RotateCcw, Search, ShieldCheck, Trash2 } from '@lucide/svelte';
  import type { HistoryItem } from '../lib/types';
  import { formatDate } from '../lib/format';
  import PageHeader from './PageHeader.svelte';

  export let items: HistoryItem[];
  export let onUndo: (id: string) => void;
  export let onUndoSession: (session: string) => void;
  export let onOpenRecycleBin: () => void;
  let query = '';
  $: visible = items.filter((item) => item.fileName.toLowerCase().includes(query.toLowerCase()));
  $: sessions = [...new Set(visible.map((item) => item.session))];

  const iconFor = (action: HistoryItem['action']) => action === 'Trashed' ? Trash2 : action === 'Moved' ? FolderInput : action === 'Renamed' ? FileInput : action === 'Kept' ? Check : Clock3;
</script>

<PageHeader eyebrow="Your safety net" title="History" description="Staged Trash and file moves can be reversed here. Finalized Trash is managed by Windows." />

<div class="history-toolbar">
  <label><Search size={15} /><input bind:value={query} placeholder="Search file history" aria-label="Search file history" /></label>
  <span><ShieldCheck size={14} /> Actions are kept for 30 days</span>
</div>

{#if visible.length === 0}
  <section class="empty"><span><ArchiveRestore size={28} /></span><h2>No actions found</h2><p>{query ? 'Try a different file name.' : 'Your moves, renames, and Trash actions will appear here.'}</p></section>
{:else}
  {#each sessions as session}
    <section class="session">
      <header><div><p class="eyebrow">Sift session</p><h2>{session}</h2></div><button on:click={() => onUndoSession(session)} disabled={!items.some((item) => item.session === session && item.undoable)}><RotateCcw size={14} /> Undo session</button></header>
      <div class="history-list">
        {#each visible.filter((item) => item.session === session) as item}
          {@const Icon = iconFor(item.action)}
          <article>
            <span class:trash={item.action === 'Trashed'} class="action-icon"><Icon size={17} /></span>
            <div class="file"><strong>{item.fileName}</strong><span>{formatDate(item.timestamp)}</span></div>
            <div class="result"><span>{item.action}</span>{#if item.destination}<strong>{item.destination}</strong>{/if}</div>
            {#if item.undoable}<button class="undo" on:click={() => onUndo(item.id)}><RotateCcw size={13} /> Undo</button>{:else if item.trashState === 'recycled'}<button class="undo" on:click={onOpenRecycleBin}><ExternalLink size={13} /> Recycle Bin</button>{:else}<span class="settled">No change</span>{/if}
          </article>
        {/each}
      </div>
    </section>
  {/each}
{/if}

<style>
  .history-toolbar{height:56px;display:flex;align-items:center;justify-content:space-between;padding:0 14px;margin-bottom:22px;background:#fff;border:1px solid var(--border);border-radius:11px}.history-toolbar label{width:260px;height:34px;display:flex;align-items:center;gap:8px;padding:0 10px;background:#f2f2ef;border-radius:7px;color:var(--text-3)}.history-toolbar input{width:100%;border:0;outline:0;background:transparent;font:11px var(--font-ui)}.history-toolbar>span{display:flex;align-items:center;gap:6px;color:#617363;font-size:9px}.session{margin-bottom:25px}.session>header{display:flex;align-items:end;justify-content:space-between;margin-bottom:9px;padding:0 2px}.eyebrow{font-size:8px;text-transform:uppercase;letter-spacing:.12em;font-weight:750;color:var(--text-3);margin:0 0 3px}.session h2{font:650 16px var(--font-display);margin:0}.session>header button{display:flex;align-items:center;gap:6px;border:0;background:transparent;color:var(--text-2);font:600 9px var(--font-ui);cursor:pointer}.session>header button:disabled{opacity:.35}.history-list{border:1px solid var(--border);border-radius:11px;background:#fff;overflow:hidden}.history-list article{display:grid;grid-template-columns:40px minmax(180px,1.5fr) minmax(150px,1fr) 76px;gap:12px;align-items:center;min-height:65px;padding:8px 14px;border-top:1px solid var(--border)}.history-list article:first-child{border:0}.action-icon{width:36px;height:36px;border-radius:9px;display:grid;place-items:center;background:#e8f0e9;color:#4b7554}.action-icon.trash{background:#f3e9e6;color:#995245}.file,.result{display:flex;flex-direction:column;min-width:0}.file strong,.result strong{font-size:11px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}.file span,.result span{font-size:9px;color:var(--text-3);margin-top:3px}.result strong{margin-top:3px;color:var(--text-2)}.undo{height:29px;display:flex;align-items:center;justify-content:center;gap:5px;border:1px solid var(--border);border-radius:7px;background:#fafaf8;color:var(--text-2);font:600 9px var(--font-ui);cursor:pointer}.undo:hover{border-color:#b5b4ad}.settled{font-size:9px;color:var(--text-3);text-align:center}.empty{text-align:center;padding:80px 20px;border:1px dashed var(--border-strong);border-radius:12px}.empty>span{width:60px;height:60px;border-radius:50%;display:grid;place-items:center;margin:0 auto 16px;background:#e8e8e3;color:var(--text-3)}.empty h2{font:650 18px var(--font-display);margin:0}.empty p{font-size:11px;color:var(--text-3)}@media(max-width:650px){.history-toolbar>span{display:none}.history-toolbar label{width:100%}.history-list article{grid-template-columns:40px 1fr 68px}.result{display:none}}
</style>
