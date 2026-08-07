<script lang="ts">
  import {
    Database,
    Folder,
    Keyboard,
    MonitorCog,
    Pin,
    Play,
    Plus,
    RotateCcw,
    ShieldCheck,
    Trash2,
    X,
  } from '@lucide/svelte';
  import PageHeader from './PageHeader.svelte';
  import type { PinnedDestination, ShortcutAction, ShortcutBindings, ThemePreference } from '../lib/types';
  import { DEFAULT_SHORTCUTS, isBindableCode, shortcutLabel } from '../lib/shortcuts';
  import Tooltip from './Tooltip.svelte';

  export let watchedFolder: string;
  export let watchEnabled: boolean;
  export let trashImmediately: boolean;
  export let autoplayMedia: boolean;
  export let theme: ThemePreference;
  export let shortcuts: ShortcutBindings;
  export let pinnedDestinations: PinnedDestination[];
  export let onFolderChange: (folder: string) => void;
  export let onPickFolder: () => void;
  export let onWatchEnabledChange: (enabled: boolean) => void;
  export let onTrashImmediatelyChange: (enabled: boolean) => void;
  export let onAutoplayMediaChange: (enabled: boolean) => void;
  export let onThemeChange: (theme: ThemePreference) => void;
  export let onShortcutsChange: (shortcuts: ShortcutBindings) => void;
  export let onAddPinned: () => void;
  export let onRemovePinned: (destination: PinnedDestination) => void;

  let notify = true;
  let cacheLimit = '500 MB';
  let capturing: ShortcutAction | null = null;
  let shortcutError = '';

  $: displayWatchedFolder = watchedFolder.startsWith('\\\\?\\UNC\\')
    ? `\\\\${watchedFolder.slice(8)}`
    : watchedFolder.startsWith('\\\\?\\')
      ? watchedFolder.slice(4)
      : watchedFolder;

  const shortcutRows: { action: ShortcutAction; name: string; description: string }[] = [
    { action: 'keep', name: 'Keep here', description: 'Leave the file in Downloads' },
    { action: 'trash', name: 'Trash', description: 'Stage the file for Trash review' },
    { action: 'undo', name: 'Undo', description: 'Reverse the previous action' },
    { action: 'fileAway', name: 'File Away', description: 'Move to a destination' },
  ];

  function captureShortcut(event: KeyboardEvent) {
    if (!capturing) return;
    event.preventDefault();
    if (event.code === 'Escape') {
      capturing = null;
      shortcutError = '';
      return;
    }
    if (event.ctrlKey || event.altKey || event.metaKey || !isBindableCode(event.code)) {
      shortcutError = 'Choose a single non-modifier key.';
      return;
    }
    const conflict = shortcutRows.find(
      (row) => row.action !== capturing && shortcuts[row.action] === event.code,
    );
    if (conflict) {
      shortcutError = `${shortcutLabel(event.code)} is already assigned to ${conflict.name}.`;
      return;
    }
    onShortcutsChange({ ...shortcuts, [capturing]: event.code });
    capturing = null;
    shortcutError = '';
  }
</script>

<svelte:window on:keydown={captureShortcut} />

<PageHeader
  eyebrow="Make it yours"
  title="Settings"
  description="Choose where Sift watches and how it behaves."
/>

<div class="settings-layout">
  <nav aria-label="Settings sections">
    <a href="#folders">Folders</a><a href="#trash">Trash</a><a href="#appearance">Appearance</a><a
      href="#media">Media</a
    ><a href="#shortcuts">Shortcuts</a><a href="#storage">Storage & privacy</a>
  </nav>
  <div class="settings-content">
    <section id="folders">
      <header>
        <span><Folder size={18} /></span>
        <div>
          <h2>Watched folder</h2>
          <p>Sift only scans the folder you choose.</p>
        </div>
      </header>
      <label class="path-field"
        >Folder path
        <div>
          <input
            value={displayWatchedFolder}
            on:input={(event) => onFolderChange(event.currentTarget.value)}
          /><button on:click={onPickFolder}>Choose folder</button>
        </div></label
      >
      <div class="setting-row">
        <div>
          <strong>Watch for new files</strong><span>Refresh Sift when a completed download appears.</span>
        </div>
        <button
          class="switch"
          class:on={watchEnabled}
          on:click={() => onWatchEnabledChange(!watchEnabled)}
          role="switch"
          aria-label="Watch for new files"
          aria-checked={watchEnabled}><span></span></button
        >
      </div>
    </section>

    <section id="trash">
      <header>
        <span><Trash2 size={18} /></span>
        <div>
          <h2>Trash behavior</h2>
          <p>Choose whether deleted files wait for review.</p>
        </div>
      </header>
      <div class="setting-row direct-trash">
        <div>
          <strong>Send directly to Recycle Bin</strong><span
            >Skip Sift Trash review and restore files later from History if needed.</span
          >
        </div>
        <button
          class="switch"
          class:on={trashImmediately}
          on:click={() => onTrashImmediatelyChange(!trashImmediately)}
          role="switch"
          aria-label="Send directly to Recycle Bin"
          aria-checked={trashImmediately}><span></span></button
        >
      </div>
      {#if trashImmediately}<div class="warning-note">
          <Trash2 size={16} /><span
            >Recycle Bin files remain undoable from History until Windows removes them.</span
          >
        </div>{/if}
    </section>

    <section>
      <header>
        <span><Pin size={18} /></span>
        <div>
          <h2>Pinned destinations</h2>
          <p>Visible while sifting and available with number keys 1–9.</p>
        </div>
      </header>
      <div class="pinned-list">
        {#each pinnedDestinations as destination, index}<div>
            <kbd>{index + 1}</kbd><Folder size={15} /><span
              ><strong>{destination.name}</strong><small>{destination.path}</small></span
            ><Tooltip text={`Remove ${destination.name}`}
              ><button on:click={() => onRemovePinned(destination)} aria-label={`Remove ${destination.name}`}
                ><X size={14} /></button
              ></Tooltip
            >
          </div>{/each}
        <button class="add-folder" on:click={onAddPinned} disabled={pinnedDestinations.length >= 9}
          ><Plus size={14} /> Add folder</button
        >
      </div>
    </section>

    <section id="appearance">
      <header>
        <span><MonitorCog size={18} /></span>
        <div>
          <h2>Appearance</h2>
          <p>Follow Windows by default or choose a fixed theme.</p>
        </div>
      </header>
      <div class="theme-options" aria-label="Theme preference">
        {#each [['system', 'System'], ['light', 'Light'], ['dark', 'Dark']] as option}
          <button
            class:active={theme === option[0]}
            on:click={() => onThemeChange(option[0] as ThemePreference)}
            aria-pressed={theme === option[0]}>{option[1]}</button
          >
        {/each}
      </div>
    </section>

    <section id="media">
      <header>
        <span><Play size={18} /></span>
        <div>
          <h2>Media previews</h2>
          <p>Choose whether audio and video start on their own.</p>
        </div>
      </header>
      <div class="setting-row media-playback">
        <div>
          <strong>Autoplay audio and video</strong><span
            >Start playback when a media file becomes the current item.</span
          >
        </div>
        <button
          class="switch"
          class:on={autoplayMedia}
          on:click={() => onAutoplayMediaChange(!autoplayMedia)}
          role="switch"
          aria-label="Autoplay audio and video previews"
          aria-checked={autoplayMedia}><span></span></button
        >
      </div>
    </section>

    <section id="shortcuts">
      <header>
        <span><Keyboard size={18} /></span>
        <div>
          <h2>Keyboard shortcuts</h2>
          <p>Select an action, then press the key you want to use.</p>
        </div>
      </header>
      <div class="shortcut-list">
        {#each shortcutRows as row}
          <div>
            <span><strong>{row.name}</strong><small>{row.description}</small></span><button
              class:capturing={capturing === row.action}
              on:click={() => {
                capturing = row.action;
                shortcutError = '';
              }}>{capturing === row.action ? 'Press a key…' : shortcutLabel(shortcuts[row.action])}</button
            >
          </div>
        {/each}
      </div>
      {#if shortcutError}<p class="shortcut-error" role="alert">{shortcutError}</p>{/if}
      <button
        class="reset"
        on:click={() => {
          onShortcutsChange({ ...DEFAULT_SHORTCUTS });
          capturing = null;
          shortcutError = '';
        }}><RotateCcw size={14} /> Reset shortcuts</button
      >
    </section>

    <section id="storage">
      <header>
        <span><Database size={18} /></span>
        <div>
          <h2>Storage & privacy</h2>
          <p>Previews stay on this device and are never uploaded.</p>
        </div>
      </header>
      <div class="setting-row">
        <div><strong>Preview cache limit</strong><span>Oldest previews are cleared automatically.</span></div>
        <select bind:value={cacheLimit}
          ><option>250 MB</option><option>500 MB</option><option>1 GB</option></select
        >
      </div>
      <div class="setting-row">
        <div>
          <strong>Completion notifications</strong><span>Show a notification after a rule run finishes.</span>
        </div>
        <button
          class="switch"
          class:on={notify}
          on:click={() => (notify = !notify)}
          role="switch"
          aria-label="Completion notifications"
          aria-checked={notify}><span></span></button
        >
      </div>
      <div class="privacy-note">
        <ShieldCheck size={16} /><span
          ><strong>Local by design.</strong> Sift has no account, cloud storage, or analytics in this MVP.</span
        >
      </div>
      <button class="clear"><Trash2 size={14} /> Clear preview cache</button>
    </section>
  </div>
</div>

<style>
  .settings-layout {
    display: grid;
    grid-template-columns: 160px minmax(0, 700px);
    gap: 30px;
  }
  .settings-layout > nav {
    position: sticky;
    top: 24px;
    height: max-content;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .settings-layout > nav a {
    padding: 8px 10px;
    border-radius: 7px;
    color: var(--text-3);
    font-size: 10px;
    font-weight: 600;
    text-decoration: none;
  }
  .settings-layout > nav a:hover,
  .settings-layout > nav a:first-child {
    background: var(--surface-2);
    color: var(--ink);
  }
  .settings-content {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding-bottom: 30px;
  }
  .settings-content > section {
    padding: 20px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    scroll-margin-top: 20px;
  }
  .settings-content section > header {
    display: flex;
    gap: 11px;
    align-items: center;
    padding-bottom: 16px;
    border-bottom: 1px solid var(--border);
    margin-bottom: 16px;
  }
  .settings-content section > header > span {
    width: 37px;
    height: 37px;
    border-radius: 9px;
    display: grid;
    place-items: center;
    background: var(--surface-2);
    color: var(--text-2);
  }
  h2 {
    font: 650 15px var(--font-display);
    margin: 0;
  }
  header p {
    font-size: 9px;
    color: var(--text-3);
    margin: 3px 0 0;
  }
  .path-field {
    font-size: 9px;
    color: var(--text-3);
    font-weight: 650;
  }
  .path-field > div {
    display: flex;
    margin-top: 7px;
  }
  .path-field input {
    height: 39px;
    flex: 1;
    min-width: 0;
    border: 1px solid var(--border-strong);
    border-radius: 8px 0 0 8px;
    padding: 0 10px;
    background: var(--surface);
    font: 11px var(--font-mono);
    color: var(--text-2);
  }
  .path-field button {
    padding: 0 13px;
    border: 1px solid var(--ink);
    border-radius: 0 8px 8px 0;
    background: var(--ink);
    color: var(--bg);
    font: 600 10px var(--font-ui);
    cursor: pointer;
  }
  .setting-row {
    min-height: 55px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-top: 1px solid var(--border);
    margin-top: 15px;
    padding-top: 15px;
    gap: 20px;
  }
  .setting-row div {
    display: flex;
    flex-direction: column;
  }
  .setting-row strong {
    font-size: 10px;
  }
  .setting-row div span {
    font-size: 9px;
    color: var(--text-3);
    margin-top: 3px;
    line-height: 1.45;
  }
  .switch {
    width: 38px;
    height: 22px;
    min-width: 38px;
    border: 0;
    border-radius: 12px;
    padding: 2px;
    background: var(--surface-3);
    cursor: pointer;
    transition: background 0.16s;
  }
  .switch span {
    display: block;
    width: 18px;
    height: 18px;
    background: var(--surface);
    border-radius: 50%;
    transition: transform 0.16s;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.16);
  }
  .switch.on {
    background: var(--accent);
  }
  .switch.on span {
    transform: translateX(16px);
  }
  .direct-trash {
    border-top: 0;
    margin-top: 0;
    padding-top: 0;
  }
  .media-playback {
    border-top: 0;
    margin-top: 0;
    padding-top: 0;
  }
  .warning-note {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 11px;
    margin-top: 14px;
    border-radius: 8px;
    border: 1px solid var(--danger-border);
    background: var(--danger-bg);
    color: var(--danger-text);
    font-size: 9px;
  }
  .pinned-list {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 7px;
  }
  .pinned-list > div,
  .add-folder {
    min-height: 48px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 9px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface-2);
  }
  .pinned-list kbd {
    width: 21px;
    height: 21px;
    display: grid;
    place-items: center;
    background: var(--surface-3);
    border-radius: 5px;
    color: var(--text-2);
    font-size: 9px;
  }
  .pinned-list > div > span {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .pinned-list strong {
    font-size: 10px;
  }
  .pinned-list small {
    max-width: 185px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-3);
    font-size: 8px;
  }
  .pinned-list > div button {
    margin-left: auto;
    border: 0;
    background: transparent;
    color: var(--text-3);
    cursor: pointer;
  }
  .add-folder {
    justify-content: center;
    border-style: dashed;
    color: var(--text-3);
    font: 600 9px var(--font-ui);
    cursor: pointer;
  }
  .theme-options {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 6px;
    padding: 5px;
    background: var(--surface-2);
    border-radius: 9px;
  }
  .theme-options button {
    height: 34px;
    border: 0;
    border-radius: 7px;
    background: transparent;
    color: var(--text-2);
    font: 650 10px var(--font-ui);
    cursor: pointer;
  }
  .theme-options button.active {
    background: var(--surface);
    color: var(--ink);
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.1);
  }
  .shortcut-list > div {
    min-height: 48px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid var(--border);
  }
  .shortcut-list > div > span {
    display: flex;
    flex-direction: column;
  }
  .shortcut-list strong {
    font-size: 10px;
  }
  .shortcut-list small {
    font-size: 8px;
    color: var(--text-3);
    margin-top: 2px;
  }
  .shortcut-list button {
    min-width: 92px;
    height: 31px;
    border: 1px solid var(--border-strong);
    border-bottom-width: 2px;
    border-radius: 7px;
    background: var(--surface-2);
    color: var(--text-2);
    font: 600 9px var(--font-mono);
    cursor: pointer;
  }
  .shortcut-list button.capturing {
    border-color: var(--accent);
    color: var(--accent);
    background: var(--surface);
  }
  .shortcut-error {
    margin: 10px 0 0;
    color: var(--danger-text);
    font-size: 9px;
  }
  .reset,
  .clear {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 14px;
    padding: 8px 10px;
    border: 1px solid var(--border-strong);
    border-radius: 7px;
    background: var(--surface);
    color: var(--text-2);
    font: 600 9px var(--font-ui);
    cursor: pointer;
  }
  .clear {
    border-color: var(--danger-border);
    color: var(--danger-text);
  }
  select {
    height: 34px;
    border: 1px solid var(--border-strong);
    border-radius: 7px;
    padding: 0 8px;
    background: var(--surface);
    color: var(--ink);
    font: 10px var(--font-ui);
  }
  .privacy-note {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 11px;
    margin-top: 15px;
    border-radius: 8px;
    background: var(--success-bg);
    color: var(--success-text);
    font-size: 9px;
  }
  @media (max-width: 700px) {
    .settings-layout {
      grid-template-columns: 1fr;
    }
    .settings-layout > nav {
      display: none;
    }
    .pinned-list {
      grid-template-columns: 1fr;
    }
  }
  .pinned-list > div :global(.tooltip-anchor) {
    margin-left: auto;
  }
</style>
