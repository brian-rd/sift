<script lang="ts">
  import { ArrowRight, ArchiveRestore, Clock3, FileCheck2, HardDrive, Inbox, ListChecks, ShieldCheck, Sparkles } from '@lucide/svelte';
  import type { DownloadFile } from '../lib/types';
  import { formatBytes, timeAgo } from '../lib/format';
  import FileIcon from './FileIcon.svelte';
  import PageHeader from './PageHeader.svelte';

  export let files: DownloadFile[];
  export let scanning: boolean;
  export let isDemo: boolean;
  export let onScan: () => void;
  export let onSift: () => void;
  export let onRules: () => void;
  export let onPreviewRules: () => void;

  $: totalBytes = files.reduce((sum, file) => sum + file.size, 0);
  $: matched = files.filter((file) => file.matchedRule);
  $: largeOrOld = files.filter((file) => file.size > 75_000_000 || Date.now() - file.modifiedAt > 30 * 86_400_000);
  $: recent = [...files].sort((a, b) => b.modifiedAt - a.modifiedAt).slice(0, 5);
</script>

<PageHeader eyebrow="Downloads, under control" title="Good morning" description="A quick look at what is waiting in your Downloads folder." {scanning} {onScan} />

{#if isDemo}
  <div class="demo-banner"><Sparkles size={15} /><span><strong>Preview mode.</strong> Connect the desktop app to scan your real Downloads folder.</span></div>
{/if}

<section class="hero-card">
  <div class="hero-copy">
    <span class="hero-kicker"><Inbox size={14} /> Ready to sort</span>
    <h2>{files.length} files are waiting</h2>
    <p>Work through them one at a time. Nothing moves until you choose.</p>
    <button class="primary" on:click={onSift}>Start sifting <ArrowRight size={16} /></button>
  </div>
  <div class="queue-visual" aria-hidden="true">
    <div class="sheet back"><span></span><span></span><span></span></div>
    <div class="sheet middle"><span></span><span></span><span></span></div>
    <div class="sheet front"><div class="mini-icon"><ArchiveRestore size={24} /></div><strong>{files.length}</strong><small>items</small></div>
  </div>
</section>

<section class="stats" aria-label="Downloads summary">
  <article><span class="stat-icon"><Inbox size={17} /></span><div><strong>{files.length}</strong><span>Unsorted files</span></div></article>
  <article><span class="stat-icon"><HardDrive size={17} /></span><div><strong>{formatBytes(totalBytes)}</strong><span>Storage used</span></div></article>
  <article><span class="stat-icon success"><FileCheck2 size={17} /></span><div><strong>{matched.length}</strong><span>Matched by rules</span></div></article>
  <article><span class="stat-icon amber"><Clock3 size={17} /></span><div><strong>{largeOrOld.length}</strong><span>Large or old</span></div></article>
</section>

<div class="dashboard-grid">
  <section class="panel recent-panel">
    <div class="section-heading"><div><p class="eyebrow">Incoming</p><h2>Recent files</h2></div><button class="text-button" on:click={onSift}>Review all <ArrowRight size={14} /></button></div>
    <div class="file-list">
      {#each recent as file}
        <div class="file-row">
          <FileIcon kind={file.kind} extension={file.extension} />
          <div class="file-name"><strong title={file.name}>{file.name}</strong><span>{formatBytes(file.size)} · {timeAgo(file.modifiedAt)}</span></div>
          {#if file.matchedRule}<span class="matched"><ShieldCheck size={12} /> {file.matchedRule}</span>{:else}<span class="needs-review">Needs review</span>{/if}
        </div>
      {/each}
    </div>
  </section>

  <section class="panel rules-panel">
    <div class="section-heading"><div><p class="eyebrow">Automation</p><h2>Rules ready</h2></div><button class="icon-button" on:click={onRules} aria-label="Open rules"><ListChecks size={17} /></button></div>
    <div class="rule-score"><div class="ring"><strong>{matched.length}</strong><span>files</span></div><div><strong>Safe to organise</strong><p>{matched.length === 1 ? 'One file matches' : `${matched.length} files match`} your active rules.</p></div></div>
    <button class="secondary full" on:click={onPreviewRules} disabled={matched.length === 0}>Preview rule run <ArrowRight size={15} /></button>
    <p class="safety"><ShieldCheck size={13} /> You will confirm every change first</p>
  </section>
</div>

<style>
  .demo-banner{display:flex;align-items:center;gap:8px;padding:10px 12px;margin:-10px 0 18px;border:1px solid #d9d5c7;border-radius:9px;background:#fffdf5;color:#70664a;font-size:12px}.hero-card{min-height:228px;background:var(--ink);border-radius:16px;color:#fff;display:flex;justify-content:space-between;overflow:hidden;position:relative;padding:34px 38px;margin-bottom:18px}.hero-card:after{content:"";position:absolute;width:290px;height:290px;border-radius:50%;right:-76px;top:-118px;background:rgba(255,255,255,.035)}.hero-copy{position:relative;z-index:2}.hero-kicker{display:inline-flex;align-items:center;gap:7px;color:#bfc0b8;font-size:11px;text-transform:uppercase;letter-spacing:.1em;font-weight:700}.hero-card h2{font:600 31px/1.15 var(--font-display);letter-spacing:-.035em;margin:16px 0 9px}.hero-card p{color:#b7b7b0;font-size:13px;margin:0 0 24px}.primary{height:39px;padding:0 15px;display:inline-flex;align-items:center;gap:18px;border:0;border-radius:8px;background:var(--accent);color:#fff;font:650 12px var(--font-ui);cursor:pointer}.primary:hover{background:var(--accent-hover)}.queue-visual{position:relative;width:240px;margin-right:25px;align-self:stretch}.sheet{position:absolute;width:166px;height:138px;border-radius:12px;display:flex;flex-direction:column;padding:22px;box-shadow:0 16px 28px rgba(0,0,0,.18)}.sheet span{width:80%;height:7px;border-radius:4px;background:rgba(18,20,18,.13);margin-bottom:9px}.back{right:-6px;top:3px;transform:rotate(11deg);background:#6f7369}.middle{right:22px;top:17px;transform:rotate(5deg);background:#aaa99f}.front{right:52px;top:32px;background:#f4f3ef;color:var(--ink);align-items:center;justify-content:center;padding:0}.front .mini-icon{color:var(--accent);margin-bottom:4px}.front strong{font:650 30px/1 var(--font-display)}.front small{color:var(--text-3);font-weight:600;margin-top:4px}.stats{display:grid;grid-template-columns:repeat(4,1fr);gap:10px;margin-bottom:18px}.stats article{height:84px;display:flex;align-items:center;gap:12px;padding:14px;background:#fff;border:1px solid var(--border);border-radius:11px}.stat-icon{width:36px;height:36px;border-radius:9px;display:grid;place-items:center;background:#e8edf0;color:#4b6570}.stat-icon.success{background:#e7f0e9;color:#467454}.stat-icon.amber{background:#f4ecde;color:#976a2e}.stats article div{display:flex;flex-direction:column}.stats strong{font:650 19px var(--font-display)}.stats article div span{font-size:10px;color:var(--text-3);margin-top:2px}.dashboard-grid{display:grid;grid-template-columns:minmax(0,1.8fr) minmax(250px,1fr);gap:18px}.panel{background:#fff;border:1px solid var(--border);border-radius:13px;padding:20px}.section-heading{display:flex;align-items:flex-start;justify-content:space-between;margin-bottom:14px}.section-heading .eyebrow{font-size:9px;letter-spacing:.12em;text-transform:uppercase;font-weight:750;color:var(--text-3);margin:0 0 4px}.section-heading h2{font:650 17px var(--font-display);margin:0}.text-button{border:0;background:none;display:flex;align-items:center;gap:5px;color:var(--text-2);font:600 11px var(--font-ui);cursor:pointer;padding:4px}.text-button:hover{color:var(--accent)}.icon-button{width:34px;height:34px;border:1px solid var(--border);border-radius:8px;background:#fafaf8;display:grid;place-items:center;cursor:pointer;color:var(--text-2)}.file-list{display:flex;flex-direction:column}.file-row{display:flex;align-items:center;gap:11px;padding:10px 0;border-top:1px solid var(--border)}.file-row:first-child{border-top:0}.file-name{display:flex;min-width:0;flex-direction:column}.file-name strong{font-size:12px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}.file-name span{font-size:10px;color:var(--text-3);margin-top:3px}.matched,.needs-review{margin-left:auto;display:flex;align-items:center;gap:4px;padding:4px 7px;border-radius:6px;font-size:9px;font-weight:650;white-space:nowrap}.matched{background:#edf4ee;color:#477354}.needs-review{background:#f0f0ed;color:#73736d}.rule-score{display:flex;gap:15px;align-items:center;padding:14px 0 20px}.ring{width:72px;height:72px;border:7px solid #dde9df;border-top-color:#5f8969;border-radius:50%;display:flex;flex-direction:column;align-items:center;justify-content:center;flex:none}.ring strong{font:650 20px/1 var(--font-display)}.ring span{font-size:9px;color:var(--text-3)}.rule-score>div:last-child strong{font-size:12px}.rule-score p{font-size:10px;color:var(--text-3);line-height:1.5;margin:4px 0 0}.secondary{height:38px;padding:0 13px;border:1px solid var(--border-strong);border-radius:8px;background:#f8f8f5;font:650 11px var(--font-ui);cursor:pointer;display:flex;align-items:center;justify-content:center;gap:8px}.secondary:hover{background:#efefeb}.secondary:disabled{opacity:.5;cursor:not-allowed}.full{width:100%}.safety{display:flex;align-items:center;justify-content:center;gap:5px;color:var(--text-3);font-size:9px;margin:12px 0 0}@media(max-width:900px){.stats{grid-template-columns:repeat(2,1fr)}.dashboard-grid{grid-template-columns:1fr}.queue-visual{display:none}}@media(max-width:560px){.hero-card{padding:26px 24px;min-height:210px}.hero-card h2{font-size:26px}.stats{grid-template-columns:1fr 1fr}.stats article{height:76px;padding:11px}.stat-icon{display:none}.dashboard-grid{display:block}.rules-panel{margin-top:14px}.matched,.needs-review{display:none}}
</style>
