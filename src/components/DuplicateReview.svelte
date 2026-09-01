<script lang="ts">
  import { CopyCheck, HardDrive, ShieldCheck, Trash2, X } from '@lucide/svelte';
  import type { DownloadFile, DuplicateGroup } from '../lib/types';
  import { formatBytes, formatDate, formatWindowsPath } from '../lib/format';
  import FileIcon from './FileIcon.svelte';
  import Tooltip from './Tooltip.svelte';

  export let groups: DuplicateGroup[];
  export let busy: boolean;
  export let onClose: () => void;
  export let onRemove: (files: DownloadFile[]) => Promise<void>;

  let keepers = groups.map((group) => group.files[0]?.path ?? '');

  $: removalFiles = groups.flatMap((group, index) =>
    group.files.filter((file) => file.path !== keepers[index]),
  );
  $: reclaimedBytes = removalFiles.reduce((total, file) => total + file.size, 0);

  function chooseKeeper(groupIndex: number, path: string) {
    keepers = keepers.map((keeper, index) => (index === groupIndex ? path : keeper));
  }

  function closeOnEscape(event: KeyboardEvent) {
    if (event.key === 'Escape' && !busy) onClose();
  }
</script>

<svelte:window on:keydown={closeOnEscape} />

<div class="duplicate-overlay" role="presentation">
  <div class="duplicate-dialog" role="dialog" aria-modal="true" aria-labelledby="duplicates-title">
    <header>
      <div class="title-block">
        <span class="title-icon"><CopyCheck size={20} /></span>
        <div>
          <p>Duplicate cleanup</p>
          <h2 id="duplicates-title">Choose which copies to keep</h2>
          <span>Only files with identical content are grouped together.</span>
        </div>
      </div>
      <Tooltip text="Close duplicate review" placement="bottom"
        ><button on:click={onClose} disabled={busy} aria-label="Close duplicate review"
          ><X size={18} /></button
        ></Tooltip
      >
    </header>

    <div class="summary" aria-live="polite">
      <div><strong>{groups.length}</strong><span>duplicate {groups.length === 1 ? 'set' : 'sets'}</span></div>
      <div><strong>{removalFiles.length}</strong><span>copies to remove</span></div>
      <div><strong>{formatBytes(reclaimedBytes)}</strong><span>space recovered</span></div>
    </div>

    <div class="group-list">
      {#each groups as group, groupIndex}
        <section class="duplicate-group" aria-labelledby={`duplicate-group-${groupIndex}`}>
          <div class="group-heading">
            <div>
              <span>Set {groupIndex + 1}</span>
              <strong id={`duplicate-group-${groupIndex}`}
                >{group.files.length} identical files · {formatBytes(group.size)} each</strong
              >
            </div>
            <span>{formatBytes(group.size * (group.files.length - 1))} recoverable</span>
          </div>
          <div class="copy-list">
            {#each group.files as file, fileIndex}
              <label class:keeper={keepers[groupIndex] === file.path}>
                <input
                  type="radio"
                  name={`duplicate-group-${groupIndex}`}
                  checked={keepers[groupIndex] === file.path}
                  on:change={() => chooseKeeper(groupIndex, file.path)}
                />
                <FileIcon kind={file.kind} extension={file.extension} />
                <span class="file-copy">
                  <Tooltip text={file.name}><strong>{file.name}</strong></Tooltip>
                  <span>Created {formatDate(file.createdAt)} · Modified {formatDate(file.modifiedAt)}</span>
                  <Tooltip text={formatWindowsPath(file.path)}
                    ><small>{formatWindowsPath(file.path)}</small></Tooltip
                  >
                </span>
                {#if keepers[groupIndex] === file.path}<span class="keep-badge"
                    ><ShieldCheck size={12} /> Keep</span
                  >{:else}<span class="remove-badge">Remove</span>{/if}
                {#if fileIndex === 0}<span class="recommended">Recommended keeper</span>{/if}
              </label>
            {/each}
          </div>
        </section>
      {/each}
    </div>

    <footer>
      <div><HardDrive size={15} /><span>Copies move to Sift Trash first, so you can undo.</span></div>
      <button class="cancel" on:click={onClose} disabled={busy}>Cancel</button>
      <button
        class="remove"
        on:click={() => onRemove(removalFiles)}
        disabled={busy || removalFiles.length === 0}
        ><Trash2 size={15} />
        {busy
          ? 'Moving copies…'
          : `Move ${removalFiles.length} ${removalFiles.length === 1 ? 'copy' : 'copies'} to Sift Trash`}</button
      >
    </footer>
  </div>
</div>

<style>
  .duplicate-overlay {
    position: fixed;
    inset: 0;
    z-index: 180;
    display: grid;
    place-items: center;
    padding: 28px;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(4px);
  }
  .duplicate-dialog {
    width: min(820px, 100%);
    max-height: min(780px, 92vh);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border: 1px solid var(--border);
    border-radius: 16px;
    background: var(--surface);
    box-shadow: var(--shadow-lg);
  }
  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    padding: 22px 24px 18px;
    border-bottom: 1px solid var(--border);
  }
  .title-block {
    display: flex;
    gap: 13px;
  }
  .title-icon {
    width: 42px;
    height: 42px;
    display: grid;
    place-items: center;
    flex: 0 0 auto;
    border-radius: 10px;
    background: var(--surface-2);
    color: var(--accent-pressed);
  }
  .title-block p {
    margin: 0 0 3px;
    color: var(--accent-pressed);
    font-size: 9px;
    font-weight: 750;
    letter-spacing: 0.11em;
    text-transform: uppercase;
  }
  h2 {
    margin: 0;
    font: 650 22px var(--font-display);
  }
  .title-block div > span {
    display: block;
    margin-top: 4px;
    color: var(--text-3);
    font-size: 10px;
  }
  header button {
    width: 36px;
    height: 36px;
    display: grid;
    place-items: center;
    border: 0;
    border-radius: 8px;
    background: var(--surface-2);
    color: var(--text-2);
    cursor: pointer;
  }
  header button:disabled {
    opacity: 0.45;
  }
  .summary {
    min-height: 66px;
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    background: var(--surface-2);
    border-bottom: 1px solid var(--border);
  }
  .summary > div {
    display: flex;
    flex-direction: column;
    justify-content: center;
    padding: 10px 20px;
    border-right: 1px solid var(--border);
  }
  .summary > div:last-child {
    border-right: 0;
  }
  .summary strong {
    font: 650 17px var(--font-display);
  }
  .summary span {
    margin-top: 1px;
    color: var(--text-3);
    font-size: 8px;
  }
  .group-list {
    min-height: 180px;
    overflow: auto;
    padding: 14px;
  }
  .duplicate-group {
    overflow: hidden;
    margin-bottom: 12px;
    border: 1px solid var(--border);
    border-radius: 11px;
  }
  .duplicate-group:last-child {
    margin-bottom: 0;
  }
  .group-heading {
    min-height: 48px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 13px;
    background: var(--surface-2);
    border-bottom: 1px solid var(--border);
  }
  .group-heading > div {
    display: flex;
    flex-direction: column;
  }
  .group-heading div > span,
  .group-heading > span {
    color: var(--text-3);
    font-size: 8px;
  }
  .group-heading strong {
    margin-top: 2px;
    font-size: 10px;
  }
  .copy-list label {
    min-height: 68px;
    display: grid;
    grid-template-columns: 20px 42px minmax(0, 1fr) auto;
    gap: 10px;
    align-items: center;
    position: relative;
    padding: 8px 13px;
    border-bottom: 1px solid var(--border);
    cursor: pointer;
  }
  .copy-list label:last-child {
    border-bottom: 0;
  }
  .copy-list label:hover {
    background: var(--surface-2);
  }
  .copy-list label.keeper {
    background: var(--success-bg);
  }
  input {
    accent-color: var(--accent-pressed);
  }
  .file-copy {
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .file-copy strong {
    display: block;
    overflow: hidden;
    color: var(--ink);
    font-size: 11px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .file-copy > span {
    margin-top: 2px;
    color: var(--text-2);
    font-size: 8px;
  }
  .file-copy small {
    display: block;
    max-width: 480px;
    overflow: hidden;
    margin-top: 2px;
    color: var(--text-3);
    font-size: 8px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .keep-badge,
  .remove-badge {
    min-width: 64px;
    height: 25px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    border-radius: 6px;
    font-size: 8px;
    font-weight: 700;
  }
  .keep-badge {
    background: var(--success-strong);
    color: var(--success-text);
  }
  .remove-badge {
    background: var(--danger-bg);
    color: var(--danger-text);
  }
  .recommended {
    position: absolute;
    top: 5px;
    right: 8px;
    color: var(--text-3);
    font-size: 7px;
    text-transform: uppercase;
  }
  footer {
    min-height: 66px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 20px;
    border-top: 1px solid var(--border);
    background: var(--surface);
  }
  footer > div {
    display: flex;
    align-items: center;
    gap: 7px;
    margin-right: auto;
    color: var(--text-3);
    font-size: 9px;
  }
  footer button {
    height: 38px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 0 12px;
    border-radius: 8px;
    font: 650 10px var(--font-ui);
    cursor: pointer;
  }
  .cancel {
    border: 1px solid var(--border-strong);
    background: var(--surface);
    color: var(--text-2);
  }
  .remove {
    min-width: 196px;
    border: 1px solid var(--danger-border);
    background: var(--danger-bg);
    color: var(--danger-text);
  }
  footer button:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  @media (max-width: 650px) {
    .duplicate-overlay {
      padding: 12px;
    }
    .copy-list label {
      grid-template-columns: 20px 38px minmax(0, 1fr);
      padding-inline: 10px;
    }
    .keep-badge,
    .remove-badge,
    .recommended,
    footer > div {
      display: none;
    }
    footer {
      padding-inline: 12px;
    }
    .remove {
      min-width: 0;
      flex: 1;
    }
  }
</style>
