<script lang="ts">
  import { Check, Database, Folder, Keyboard, Pin, Plus, ShieldCheck, Trash2, X } from '@lucide/svelte';
  import PageHeader from './PageHeader.svelte';

  export let watchedFolder: string;
  export let onFolderChange: (folder: string) => void;
  export let onPickFolder: () => void;
  let watchEnabled = true;
  let notify = true;
  let cacheLimit = '500 MB';
  let pinned = ['Documents', 'Pictures', 'Work', 'Receipts'];
  let saved = false;

  function save() { saved = true; setTimeout(() => saved = false, 1800); }
</script>

<PageHeader eyebrow="Make it yours" title="Settings" description="Choose where Sift watches and how it behaves." />

<div class="settings-layout">
  <nav aria-label="Settings sections"><a href="#folders">Folders</a><a href="#shortcuts">Shortcuts</a><a href="#storage">Storage & privacy</a></nav>
  <div class="settings-content">
    <section id="folders">
      <header><span><Folder size={18} /></span><div><h2>Watched folder</h2><p>Sift only scans the folder you choose.</p></div></header>
      <label class="path-field">Folder path<div><input value={watchedFolder} on:input={(e) => onFolderChange(e.currentTarget.value)} /><button on:click={onPickFolder}>Choose folder</button></div></label>
      <div class="setting-row"><div><strong>Watch for new files</strong><span>Refresh the inbox when a completed download appears.</span></div><button class="switch" class:on={watchEnabled} on:click={() => watchEnabled = !watchEnabled} role="switch" aria-label="Watch for new files" aria-checked={watchEnabled}><span></span></button></div>
    </section>

    <section>
      <header><span><Pin size={18} /></span><div><h2>Pinned destinations</h2><p>Use number keys 1–9 to move files here while sifting.</p></div></header>
      <div class="pinned-list">
        {#each pinned as folder, index}<div><kbd>{index + 1}</kbd><Folder size={15} /><strong>{folder}</strong><button on:click={() => pinned = pinned.filter((item) => item !== folder)} aria-label={`Remove ${folder}`}><X size={14} /></button></div>{/each}
        <button class="add-folder" on:click={() => pinned = [...pinned, `Folder ${pinned.length + 1}`]} disabled={pinned.length >= 9}><Plus size={14} /> Add folder</button>
      </div>
    </section>

    <section id="shortcuts">
      <header><span><Keyboard size={18} /></span><div><h2>Keyboard shortcuts</h2><p>Designed to keep your hands on the keyboard.</p></div></header>
      <div class="shortcut-grid"><span>Move to Trash <kbd>←</kbd></span><span>Keep in Downloads <kbd>↓</kbd></span><span>Move to suggestion <kbd>→</kbd></span><span>Review later <kbd>↑</kbd></span><span>Choose destination <kbd>M</kbd></span><span>Undo last action <kbd>Ctrl Z</kbd></span></div>
    </section>

    <section id="storage">
      <header><span><Database size={18} /></span><div><h2>Storage & privacy</h2><p>Previews stay on this device and are never uploaded.</p></div></header>
      <div class="setting-row"><div><strong>Preview cache limit</strong><span>Oldest previews are cleared automatically.</span></div><select bind:value={cacheLimit}><option>250 MB</option><option>500 MB</option><option>1 GB</option></select></div>
      <div class="setting-row"><div><strong>Completion notifications</strong><span>Show a notification after a rule run finishes.</span></div><button class="switch" class:on={notify} on:click={() => notify = !notify} role="switch" aria-label="Completion notifications" aria-checked={notify}><span></span></button></div>
      <div class="privacy-note"><ShieldCheck size={16} /><span><strong>Local by design.</strong> Sift has no account, cloud storage, or analytics in this MVP.</span></div>
      <button class="clear"><Trash2 size={14} /> Clear preview cache</button>
    </section>
    <div class="save-row"><button class="save" on:click={save}>{#if saved}<Check size={15} /> Saved{:else}Save changes{/if}</button></div>
  </div>
</div>

<style>
  .settings-layout{display:grid;grid-template-columns:160px minmax(0,680px);gap:30px}.settings-layout>nav{position:sticky;top:24px;height:max-content;display:flex;flex-direction:column;gap:3px}.settings-layout>nav a{padding:8px 10px;border-radius:7px;color:var(--text-3);font-size:10px;font-weight:600;text-decoration:none}.settings-layout>nav a:hover,.settings-layout>nav a:first-child{background:#e9e8e3;color:var(--ink)}.settings-content{display:flex;flex-direction:column;gap:14px}.settings-content>section{padding:20px;background:#fff;border:1px solid var(--border);border-radius:12px;scroll-margin-top:20px}.settings-content section>header{display:flex;gap:11px;align-items:center;padding-bottom:16px;border-bottom:1px solid var(--border);margin-bottom:16px}.settings-content section>header>span{width:37px;height:37px;border-radius:9px;display:grid;place-items:center;background:#ecece8;color:var(--text-2)}h2{font:650 15px var(--font-display);margin:0}header p{font-size:9px;color:var(--text-3);margin:3px 0 0}.path-field{font-size:9px;color:var(--text-3);font-weight:650}.path-field>div{display:flex;margin-top:7px}.path-field input{height:39px;flex:1;min-width:0;border:1px solid var(--border-strong);border-radius:8px 0 0 8px;padding:0 10px;font:11px var(--font-mono);color:var(--text-2)}.path-field button{padding:0 13px;border:1px solid var(--ink);border-radius:0 8px 8px 0;background:var(--ink);color:#fff;font:600 10px var(--font-ui);cursor:pointer}.setting-row{min-height:55px;display:flex;align-items:center;justify-content:space-between;border-top:1px solid var(--border);margin-top:15px;padding-top:15px}.setting-row div{display:flex;flex-direction:column}.setting-row strong{font-size:10px}.setting-row div span{font-size:9px;color:var(--text-3);margin-top:3px}.switch{width:38px;height:22px;border:0;border-radius:12px;padding:2px;background:#cecec8;cursor:pointer;transition:background .16s}.switch span{display:block;width:18px;height:18px;background:#fff;border-radius:50%;transition:transform .16s;box-shadow:0 1px 3px rgba(0,0,0,.16)}.switch.on{background:#5d8065}.switch.on span{transform:translateX(16px)}.pinned-list{display:grid;grid-template-columns:1fr 1fr;gap:7px}.pinned-list>div,.add-folder{height:42px;display:flex;align-items:center;gap:8px;padding:0 9px;border:1px solid var(--border);border-radius:8px;background:#fafaf8}.pinned-list kbd{width:21px;height:21px;display:grid;place-items:center;background:#e7e7e2;border-radius:5px;color:var(--text-2);font-size:9px}.pinned-list strong{font-size:10px}.pinned-list>div button{margin-left:auto;border:0;background:transparent;color:var(--text-3);cursor:pointer}.add-folder{justify-content:center;border-style:dashed;color:var(--text-3);font:600 9px var(--font-ui);cursor:pointer}.shortcut-grid{display:grid;grid-template-columns:1fr 1fr;gap:0 24px}.shortcut-grid span{height:38px;display:flex;align-items:center;justify-content:space-between;border-bottom:1px solid var(--border);font-size:9px;color:var(--text-2)}kbd{padding:3px 6px;border:1px solid #d3d2cb;border-bottom-width:2px;border-radius:5px;background:#f5f5f2;font:8px var(--font-mono);color:var(--text-2)}select{height:34px;border:1px solid var(--border-strong);border-radius:7px;padding:0 8px;background:#fff;font:10px var(--font-ui)}.privacy-note{display:flex;align-items:center;gap:8px;padding:11px;margin-top:15px;border-radius:8px;background:#edf3ee;color:#607362;font-size:9px}.clear{display:flex;align-items:center;gap:6px;margin-top:14px;padding:8px 10px;border:1px solid #e0cbc5;border-radius:7px;background:#fff;color:#915143;font:600 9px var(--font-ui);cursor:pointer}.save-row{display:flex;justify-content:flex-end;padding-bottom:30px}.save{height:39px;min-width:112px;display:flex;align-items:center;justify-content:center;gap:6px;border:0;border-radius:8px;background:var(--ink);color:#fff;font:650 10px var(--font-ui);cursor:pointer}@media(max-width:700px){.settings-layout{grid-template-columns:1fr}.settings-layout>nav{display:none}.pinned-list,.shortcut-grid{grid-template-columns:1fr}}
</style>
