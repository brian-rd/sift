<script lang="ts">
  import { X } from '@lucide/svelte';
  import type {
    DownloadFile,
    FileKind,
    FileSortField,
    SiftQueuePreferences,
    SortDirection,
  } from '../lib/types';
  import { FILE_KINDS } from '../lib/fileQueue';
  import Tooltip from './Tooltip.svelte';

  export let files: DownloadFile[];
  export let matchingCount: number;
  export let preferences: SiftQueuePreferences;
  export let onChange: (preferences: SiftQueuePreferences) => void;
  export let onClose: () => void;

  const fileKindOptions: { kind: FileKind; label: string }[] = [
    { kind: 'image', label: 'Images' },
    { kind: 'pdf', label: 'PDFs' },
    { kind: 'archive', label: 'Archives' },
    { kind: 'video', label: 'Videos' },
    { kind: 'audio', label: 'Audio' },
    { kind: 'text', label: 'Text' },
    { kind: 'other', label: 'Other' },
  ];

  const sortOptions: { value: FileSortField; label: string }[] = [
    { value: 'modifiedAt', label: 'Date modified' },
    { value: 'createdAt', label: 'Date created' },
    { value: 'name', label: 'Name' },
    { value: 'size', label: 'Size' },
    { value: 'type', label: 'File type' },
  ];

  function updateSortBy(sortBy: FileSortField) {
    const dateOrSize = sortBy === 'modifiedAt' || sortBy === 'createdAt' || sortBy === 'size';
    onChange({ ...preferences, sortBy, direction: dateOrSize ? 'desc' : 'asc' });
  }

  function toggleKind(kind: FileKind) {
    if (preferences.includedKinds.length === FILE_KINDS.length) {
      onChange({ ...preferences, includedKinds: [kind] });
      return;
    }
    const included = preferences.includedKinds.includes(kind);
    const includedKinds = included
      ? preferences.includedKinds.filter((item) => item !== kind)
      : [...preferences.includedKinds, kind];
    onChange({
      ...preferences,
      includedKinds: includedKinds.length
        ? FILE_KINDS.filter((item) => includedKinds.includes(item))
        : [...FILE_KINDS],
    });
  }

  function directionLabels(sortBy: FileSortField) {
    if (sortBy === 'name' || sortBy === 'type') return { asc: 'A to Z', desc: 'Z to A' };
    if (sortBy === 'size') return { asc: 'Smallest first', desc: 'Largest first' };
    return { asc: 'Oldest first', desc: 'Newest first' };
  }

  function kindCount(kind: FileKind) {
    return files.filter((file) => file.kind === kind).length;
  }
</script>

<div
  class="overlay"
  role="presentation"
  on:click={(event) => event.currentTarget === event.target && onClose()}
>
  <div class="queue-dialog" role="dialog" aria-modal="true" aria-labelledby="queue-title">
    <header>
      <div>
        <p>Queue options</p>
        <h2 id="queue-title">Choose what comes next</h2>
      </div>
      <Tooltip text="Close queue options" placement="bottom"
        ><button on:click={onClose} aria-label="Close queue options"><X size={18} /></button></Tooltip
      >
    </header>
    <div class="queue-summary">
      <strong>{matchingCount}</strong>
      <span>of {files.length} remaining {files.length === 1 ? 'file' : 'files'} in this queue</span>
    </div>
    <div class="sort-controls">
      <label
        >Sort by
        <select
          value={preferences.sortBy}
          on:change={(event) => updateSortBy(event.currentTarget.value as FileSortField)}
        >
          {#each sortOptions as option}<option value={option.value}>{option.label}</option>{/each}
        </select></label
      >
      <label
        >Order
        <select
          value={preferences.direction}
          on:change={(event) =>
            onChange({ ...preferences, direction: event.currentTarget.value as SortDirection })}
        >
          <option value="desc">{directionLabels(preferences.sortBy).desc}</option>
          <option value="asc">{directionLabels(preferences.sortBy).asc}</option>
        </select></label
      >
    </div>
    <fieldset class="file-type-filter">
      <legend>Include file types</legend>
      <button
        class:active={preferences.includedKinds.length === FILE_KINDS.length}
        on:click={() => onChange({ ...preferences, includedKinds: [...FILE_KINDS] })}
        aria-pressed={preferences.includedKinds.length === FILE_KINDS.length}
        ><span>All files</span><strong>{files.length}</strong></button
      >
      {#each fileKindOptions as option}<button
          class:active={preferences.includedKinds.length < FILE_KINDS.length &&
            preferences.includedKinds.includes(option.kind)}
          on:click={() => toggleKind(option.kind)}
          aria-pressed={preferences.includedKinds.length < FILE_KINDS.length &&
            preferences.includedKinds.includes(option.kind)}
          ><span>{option.label}</span><strong>{kindCount(option.kind)}</strong></button
        >{/each}
    </fieldset>
    <button class="done" on:click={onClose}>Done</button>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 100;
    display: grid;
    place-items: center;
    padding: 24px;
    background: rgba(10, 11, 9, 0.58);
    backdrop-filter: blur(3px);
  }
  .queue-dialog {
    width: min(480px, 100%);
    padding: 22px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 15px;
    box-shadow: var(--shadow-lg);
  }
  header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 16px;
  }
  header p {
    margin: 0 0 4px;
    color: var(--accent);
    font-size: 8px;
    text-transform: uppercase;
    letter-spacing: 0.11em;
    font-weight: 750;
  }
  h2 {
    margin: 0;
    font: 650 20px var(--font-display);
  }
  header button {
    width: 34px;
    height: 34px;
    border: 0;
    border-radius: 8px;
    background: var(--surface-2);
    color: var(--text-2);
    display: grid;
    place-items: center;
    cursor: pointer;
  }
  .queue-summary {
    display: flex;
    align-items: baseline;
    gap: 7px;
    padding: 11px 12px;
    margin-bottom: 14px;
    border-radius: 9px;
    background: var(--surface-2);
  }
  .queue-summary strong {
    font: 700 20px var(--font-display);
  }
  .queue-summary span {
    color: var(--text-3);
    font-size: 9px;
  }
  .sort-controls {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 9px;
  }
  .sort-controls label {
    display: flex;
    flex-direction: column;
    gap: 6px;
    color: var(--text-3);
    font-size: 8px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  .sort-controls select {
    width: 100%;
    height: 38px;
    padding: 0 9px;
    border: 1px solid var(--border-strong);
    border-radius: 8px;
    background: var(--surface);
    color: var(--ink);
    font: 600 10px var(--font-ui);
    text-transform: none;
    letter-spacing: 0;
  }
  .file-type-filter {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 7px;
    min-width: 0;
    margin: 18px 0 0;
    padding: 0;
    border: 0;
  }
  .file-type-filter legend {
    grid-column: 1 / -1;
    margin-bottom: 7px;
    color: var(--text-3);
    font-size: 8px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  .file-type-filter button {
    min-height: 35px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 10px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface);
    color: var(--text-2);
    font: 600 10px var(--font-ui);
    cursor: pointer;
  }
  .file-type-filter button:hover {
    border-color: var(--border-strong);
  }
  .file-type-filter button.active {
    border-color: var(--primary-action-bg);
    background: var(--primary-action-bg);
    color: var(--primary-action-text);
  }
  .file-type-filter button strong {
    min-width: 20px;
    padding: 2px 5px;
    border-radius: 5px;
    background: var(--surface-2);
    color: var(--text-3);
    font-size: 8px;
  }
  .file-type-filter button.active strong {
    background: var(--primary-action-key-bg);
    color: var(--primary-action-text);
  }
  .done {
    width: 100%;
    height: 38px;
    margin-top: 14px;
    border: 0;
    border-radius: 8px;
    background: var(--primary-action-bg);
    color: var(--primary-action-text);
    font: 700 10px var(--font-ui);
    cursor: pointer;
  }
  .done:hover {
    background: var(--primary-action-hover);
  }
</style>
