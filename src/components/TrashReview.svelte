<script lang="ts">
  import { CheckSquare, FolderClock, MousePointer2, RotateCcw, Trash2, X } from '@lucide/svelte';
  import type { TrashItem } from '../lib/types';
  import { formatBytes, formatDate } from '../lib/format';
  import FileIcon from './FileIcon.svelte';
  import Tooltip from './Tooltip.svelte';

  export let items: TrashItem[];
  export let intent: 'review' | 'overview' | 'exit';
  export let busy: boolean;
  export let onRestore: (operationIds: number[]) => void;
  export let onRecycle: (operationIds: number[]) => void;
  export let onClose: () => void;
  export let onContinue: () => void;

  let selected: number[] = [];
  let context: { item: TrashItem; x: number; y: number } | null = null;

  $: selected = selected.filter((id) => items.some((item) => item.operationId === id));
  $: allSelected = items.length > 0 && selected.length === items.length;

  function toggle(operationId: number) {
    selected = selected.includes(operationId) ? selected.filter((id) => id !== operationId) : [...selected, operationId];
  }

  function toggleAll() {
    selected = allSelected ? [] : items.map((item) => item.operationId);
  }

  function showContextMenu(event: MouseEvent, item: TrashItem) {
    event.preventDefault();
    selected = selected.includes(item.operationId) ? selected : [item.operationId];
    context = { item, x: Math.min(event.clientX, window.innerWidth - 210), y: Math.min(event.clientY, window.innerHeight - 150) };
  }

  function closeOnEscape(event: KeyboardEvent) {
    if (event.key !== 'Escape') return;
    if (context) context = null;
    else onClose();
  }
</script>

<svelte:window on:click={() => context = null} on:keydown={closeOnEscape} />

<div class="trash-overlay" role="presentation">
  <div class="trash-dialog" role="dialog" aria-modal="true" aria-labelledby="trash-title">
    <header class="dialog-header">
      <div class="title-block"><span class="title-icon"><Trash2 size={20} /></span><div><p>{intent === 'review' ? 'Sift Trash' : 'Before you go'}</p><h2 id="trash-title">Review Trash</h2><span>{items.length} {items.length === 1 ? 'file is' : 'files are'} waiting for your decision.</span></div></div>
      <Tooltip text="Close Trash" placement="bottom"><button class="icon-button" on:click={onClose} aria-label="Close Trash"><X size={18} /></button></Tooltip>
    </header>

    {#if items.length > 0}
      <div class="trash-toolbar">
        <label><input type="checkbox" checked={allSelected} on:change={toggleAll} /><span>Select all</span></label>
        <span><MousePointer2 size={13} /> Right-click a file for quick actions</span>
      </div>
      <div class="trash-list">
        {#each items as item (item.operationId)}
          <article role="group" aria-label={item.name} on:contextmenu={(event) => showContextMenu(event, item)}>
            <label class="select-file" aria-label={`Select ${item.name}`}><input type="checkbox" checked={selected.includes(item.operationId)} on:change={() => toggle(item.operationId)} /></label>
            <FileIcon kind={item.kind} extension={item.extension} />
            <div class="file-copy"><Tooltip text={item.name}><strong>{item.name}</strong></Tooltip><span>{formatBytes(item.size)} · Modified {formatDate(item.modifiedAt)}</span><Tooltip text={item.originalPath}><small>{item.originalPath}</small></Tooltip></div>
            <button class="restore" on:click={() => onRestore([item.operationId])} disabled={busy}><RotateCcw size={14} /> Undo</button>
          </article>
        {/each}
      </div>
      <div class="selection-actions">
        <span>{selected.length ? `${selected.length} selected` : 'Select files to act on them together'}</span>
        <button class="restore-selected" on:click={() => onRestore(selected)} disabled={busy || selected.length === 0}><RotateCcw size={15} /> Restore selected</button>
        <button class="recycle-selected" on:click={() => onRecycle(selected)} disabled={busy || selected.length === 0}><Trash2 size={15} /> Move to Recycle Bin</button>
      </div>
    {:else}
      <div class="empty-trash"><span><CheckSquare size={30} /></span><h3>Trash is empty</h3><p>Every staged file has been restored or moved to the Windows Recycle Bin.</p></div>
    {/if}

    <footer>
      <div><FolderClock size={15} /><span>Files stay in Sift Trash until you review them.</span></div>
      {#if intent === 'review'}
        <button class="continue" on:click={onClose}>Done</button>
      {:else}
        <button class="secondary" on:click={onClose}>{intent === 'exit' ? 'Keep Sift open' : 'Keep sifting'}</button>
        <button class="continue" on:click={onContinue}>{intent === 'exit' ? 'Exit Sift' : 'Continue to overview'}</button>
      {/if}
    </footer>
  </div>

  {#if context}
    <div class="context-menu" style={`left:${context.x}px;top:${context.y}px`} role="menu" tabindex="-1">
      <button role="menuitem" on:click={() => { onRestore([context!.item.operationId]); context = null; }}><RotateCcw size={14} /> Undo trash</button>
      <button role="menuitem" class="danger" on:click={() => { onRecycle([context!.item.operationId]); context = null; }}><Trash2 size={14} /> Move to Recycle Bin</button>
    </div>
  {/if}
</div>

<style>
  .trash-overlay{position:fixed;inset:0;z-index:180;display:grid;place-items:center;padding:28px;background:rgba(0,0,0,.6);backdrop-filter:blur(4px)}.trash-dialog{width:min(780px,100%);max-height:min(760px,92vh);display:flex;flex-direction:column;background:var(--surface);border:1px solid var(--border);border-radius:16px;box-shadow:var(--shadow-lg);overflow:hidden}.dialog-header{display:flex;align-items:flex-start;justify-content:space-between;padding:22px 24px 18px;border-bottom:1px solid var(--border)}.title-block{display:flex;gap:13px}.title-icon{width:42px;height:42px;display:grid;place-items:center;border-radius:10px;background:var(--danger-bg);color:var(--danger-text)}.title-block p{margin:0 0 3px;color:var(--danger-text);font-size:9px;font-weight:750;text-transform:uppercase;letter-spacing:.11em}.title-block h2{margin:0;font:650 22px var(--font-display)}.title-block div>span{display:block;margin-top:4px;color:var(--text-3);font-size:10px}.icon-button{width:36px;height:36px;display:grid;place-items:center;border:0;border-radius:8px;background:var(--surface-2);color:var(--text-2);cursor:pointer}.trash-toolbar{height:44px;display:flex;align-items:center;justify-content:space-between;padding:0 20px;background:var(--surface-2);border-bottom:1px solid var(--border)}.trash-toolbar label,.trash-toolbar>span{display:flex;align-items:center;gap:7px;color:var(--text-2);font-size:9px}.trash-toolbar input,.select-file input{accent-color:var(--accent)}.trash-list{min-height:120px;max-height:410px;overflow:auto}.trash-list article{min-height:70px;display:grid;grid-template-columns:22px 42px minmax(0,1fr) 84px;gap:11px;align-items:center;padding:9px 20px;border-bottom:1px solid var(--border)}.trash-list article:hover{background:var(--surface-2)}.select-file{display:grid;place-items:center}.file-copy{min-width:0;display:flex;flex-direction:column}.file-copy strong{font-size:11px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}.file-copy span{margin-top:3px;color:var(--text-2);font-size:9px}.file-copy small{margin-top:3px;color:var(--text-3);font-size:8px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}.restore{height:32px;display:flex;align-items:center;justify-content:center;gap:5px;border:1px solid var(--border-strong);border-radius:7px;background:var(--surface);color:var(--text-2);font:650 9px var(--font-ui);cursor:pointer}.restore:disabled{opacity:.45}.selection-actions{min-height:62px;display:flex;align-items:center;gap:8px;padding:10px 20px;border-top:1px solid var(--border);background:var(--surface-2)}.selection-actions>span{margin-right:auto;color:var(--text-3);font-size:9px}.selection-actions button,.continue,.secondary{height:36px;display:flex;align-items:center;justify-content:center;gap:6px;padding:0 12px;border-radius:8px;font:650 10px var(--font-ui);cursor:pointer}.restore-selected,.secondary{border:1px solid var(--border-strong);background:var(--surface);color:var(--text-2)}.recycle-selected{border:1px solid var(--danger-border);background:var(--danger-bg);color:var(--danger-text)}.selection-actions button:disabled{opacity:.42;cursor:not-allowed}.empty-trash{display:flex;flex-direction:column;align-items:center;padding:62px 24px;text-align:center}.empty-trash>span{width:64px;height:64px;display:grid;place-items:center;border-radius:50%;background:var(--success-bg);color:var(--success-text)}.empty-trash h3{margin:16px 0 5px;font:650 18px var(--font-display)}.empty-trash p{margin:0;color:var(--text-3);font-size:10px}.trash-dialog>footer{min-height:64px;display:flex;align-items:center;gap:8px;padding:12px 20px;border-top:1px solid var(--border)}.trash-dialog>footer>div{display:flex;align-items:center;gap:7px;margin-right:auto;color:var(--text-3);font-size:9px}.continue{border:1px solid var(--ink);background:var(--ink);color:var(--bg)}.context-menu{position:fixed;z-index:220;width:196px;padding:6px;background:var(--surface);border:1px solid var(--border-strong);border-radius:9px;box-shadow:var(--shadow-lg)}.context-menu button{width:100%;height:34px;display:flex;align-items:center;gap:8px;padding:0 9px;border:0;border-radius:6px;background:transparent;color:var(--text-2);font:600 9px var(--font-ui);text-align:left;cursor:pointer}.context-menu button:hover{background:var(--surface-2)}.context-menu button.danger{color:var(--danger-text)}@media(max-width:650px){.trash-overlay{padding:12px}.trash-list article{grid-template-columns:22px 38px minmax(0,1fr);padding-inline:12px}.restore{display:none}.trash-toolbar>span,.trash-dialog>footer>div,.selection-actions>span{display:none}.selection-actions{justify-content:flex-end;padding-inline:12px}.trash-dialog>footer{padding-inline:12px}}
</style>
