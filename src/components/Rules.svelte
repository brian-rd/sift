<script lang="ts">
  import { ArrowDown, ArrowUp, Check, ChevronRight, Eye, FileSearch, GripVertical, MoreHorizontal, Plus, ShieldCheck, ToggleLeft, ToggleRight, X } from '@lucide/svelte';
  import type { DownloadFile, Rule, RuleActionType, RuleConditionType } from '../lib/types';
  import type { RuleMatch } from '../lib/rules';
  import { evaluateRules } from '../lib/rules';
  import FileIcon from './FileIcon.svelte';
  import { formatBytes } from '../lib/format';
  import PageHeader from './PageHeader.svelte';
  import Tooltip from './Tooltip.svelte';

  export let rules: Rule[];
  export let files: DownloadFile[];
  export let onUpdate: (rules: Rule[]) => void;
  export let onRun: (matching: RuleMatch[]) => void | Promise<void>;

  let showEditor = false;
  let showPreview = false;
  let name = '';
  let conditionType: RuleConditionType = 'extension';
  let conditionValue = 'pdf';
  let actionType: RuleActionType = 'move';
  let destination = 'Documents';

  $: activeRules = rules.filter((rule) => rule.enabled);
  $: matches = evaluateRules(files, rules);
  $: matchCounts = matches.reduce((counts, match) => counts.set(match.rule.id, (counts.get(match.rule.id) ?? 0) + 1), new Map<string, number>());
  $: matchedFiles = matches.map(({ file, rule }) => ({ ...file, suggestedFolder: rule.destination, matchedRule: rule.name }));

  function toggleRule(id: string) { onUpdate(rules.map((rule) => rule.id === id ? { ...rule, enabled: !rule.enabled } : rule)); }
  function moveRule(index: number, delta: number) {
    const target = index + delta;
    if (target < 0 || target >= rules.length) return;
    const next = [...rules]; [next[index], next[target]] = [next[target], next[index]]; onUpdate(next);
  }
  function addRule() {
    if (!name.trim() || !conditionValue.trim()) return;
    onUpdate([...rules, { id: crypto.randomUUID(), name: name.trim(), conditionType, conditionValue: conditionValue.trim(), actionType, destination: actionType === 'move' ? destination.trim() : undefined, enabled: true, matches: 0 }]);
    showEditor = false; name = '';
  }
  const conditionLabel = (rule: Rule) => ({ extension: 'Extension is', contains: 'Name contains', startsWith: 'Name starts with', endsWith: 'Name ends with', glob: 'Glob matches', regex: 'Regex matches', size: 'Size over', age: 'Older than' })[rule.conditionType];
</script>

<PageHeader eyebrow="Automatic sorting" title="Rules" description="Handle the predictable files. You will always see a preview before anything moves.">
  <button class="primary" on:click={() => showEditor = true}><Plus size={15} /> New rule</button>
</PageHeader>

<section class="summary-bar">
  <div><span class="summary-icon"><ShieldCheck size={19} /></span><div><strong>{activeRules.length} active rules</strong><p>{matchedFiles.length} files currently match</p></div></div>
  <button on:click={() => showPreview = true} disabled={matchedFiles.length === 0}><Eye size={15} /> Preview changes <ChevronRight size={14} /></button>
</section>

<section class="rules-card">
  <div class="rules-header"><span>Priority</span><span>Rule</span><span>When</span><span>Then</span><span>Status</span><span></span></div>
  {#each rules as rule, index}
    {@const liveMatches = matchCounts.get(rule.id) ?? 0}
    <article class:disabled={!rule.enabled}>
      <div class="priority"><GripVertical size={15} /><strong>{index + 1}</strong><div><Tooltip text={`Move ${rule.name} up`} placement="right"><button on:click={() => moveRule(index,-1)} disabled={index === 0} aria-label={`Move ${rule.name} up`}><ArrowUp size={11} /></button></Tooltip><Tooltip text={`Move ${rule.name} down`} placement="right"><button on:click={() => moveRule(index,1)} disabled={index === rules.length - 1} aria-label={`Move ${rule.name} down`}><ArrowDown size={11} /></button></Tooltip></div></div>
      <div class="rule-name"><strong>{rule.name}</strong><span>{liveMatches} {liveMatches === 1 ? 'match' : 'matches'}</span></div>
      <div class="condition"><span>{conditionLabel(rule)}</span><code>{rule.conditionValue}{rule.conditionType === 'age' ? ' days' : ''}</code></div>
      <div class="action"><span>{rule.actionType === 'move' ? 'Move to' : rule.actionType}</span><strong>{rule.destination ?? (rule.actionType === 'review' ? 'Review queue' : '—')}</strong></div>
      <Tooltip text={`${rule.enabled ? 'Disable' : 'Enable'} ${rule.name}`}><button class="toggle" on:click={() => toggleRule(rule.id)} aria-label={`${rule.enabled ? 'Disable' : 'Enable'} ${rule.name}`} aria-pressed={rule.enabled}>{#if rule.enabled}<ToggleRight size={28} />{:else}<ToggleLeft size={28} />{/if}</button></Tooltip>
      <Tooltip text={`More options for ${rule.name}`}><button class="more" aria-label={`More options for ${rule.name}`}><MoreHorizontal size={17} /></button></Tooltip>
    </article>
  {/each}
</section>

<div class="rules-note"><FileSearch size={18} /><div><strong>Rules stop at the first match</strong><p>Drag or use the arrows to set priority. Incomplete downloads and hidden files are ignored automatically.</p></div></div>

{#if showEditor}
  <div class="modal-backdrop" role="presentation" on:click={(e) => e.currentTarget === e.target && (showEditor = false)}>
    <div class="modal" role="dialog" aria-modal="true" aria-labelledby="new-rule-title">
      <header><div><p class="eyebrow">Rule builder</p><h2 id="new-rule-title">Create a sorting rule</h2></div><Tooltip text="Close rule builder" placement="bottom"><button on:click={() => showEditor = false} aria-label="Close"><X size={18} /></button></Tooltip></header>
      <label>Rule name<input bind:value={name} placeholder="e.g. Monthly reports" /></label>
      <div class="field-grid">
        <label>When<select bind:value={conditionType}><option value="extension">Extension is</option><option value="contains">Name contains</option><option value="startsWith">Name starts with</option><option value="endsWith">Name ends with</option><option value="glob">Glob matches</option><option value="regex">Regex matches</option><option value="size">Size over (MB)</option><option value="age">Older than (days)</option></select></label>
        <label>Value<input bind:value={conditionValue} placeholder="pdf" /></label>
      </div>
      <div class="field-grid">
        <label>Then<select bind:value={actionType}><option value="move">Move</option><option value="rename">Rename</option><option value="trash">Move to Trash</option><option value="ignore">Ignore</option><option value="review">Add to review queue</option></select></label>
        {#if actionType === 'move'}<label>Destination<input bind:value={destination} placeholder="Documents / Reports" /></label>{/if}
      </div>
      <div class="rule-sentence"><span>IF</span> {conditionLabel({conditionType} as Rule).toLowerCase()} <code>{conditionValue || '…'}</code> <span>THEN</span> {actionType === 'move' ? `move to ${destination || '…'}` : actionType}</div>
      <footer><button class="secondary" on:click={() => showEditor = false}>Cancel</button><button class="primary" on:click={addRule} disabled={!name.trim() || !conditionValue.trim()}><Check size={15} /> Create rule</button></footer>
    </div>
  </div>
{/if}

{#if showPreview}
  <div class="modal-backdrop" role="presentation" on:click={(e) => e.currentTarget === e.target && (showPreview = false)}>
    <div class="modal preview-modal" role="dialog" aria-modal="true" aria-labelledby="preview-title">
      <header><div><p class="eyebrow">Dry run</p><h2 id="preview-title">Preview {matchedFiles.length} changes</h2></div><Tooltip text="Close preview" placement="bottom"><button on:click={() => showPreview = false} aria-label="Close"><X size={18} /></button></Tooltip></header>
      <p class="modal-intro">Review exactly what will happen. No files have moved yet.</p>
      <div class="preview-list">
        {#each matchedFiles as file}
          <div><FileIcon kind={file.kind} extension={file.extension} /><span><strong>{file.name}</strong><small>{formatBytes(file.size)}</small></span><ChevronRight size={15} /><span class="destination"><small>Move to</small><strong>{file.suggestedFolder}</strong></span></div>
        {/each}
      </div>
      <footer><span class="safe-copy"><ShieldCheck size={14} /> Moves are undoable from History</span><button class="secondary" on:click={() => showPreview = false}>Cancel</button><button class="primary" on:click={async () => { await onRun(matches); showPreview = false; }}><Check size={15} /> Apply {matches.length} changes</button></footer>
    </div>
  </div>
{/if}

<style>
  .primary,.secondary{height:38px;padding:0 14px;border-radius:8px;display:inline-flex;align-items:center;justify-content:center;gap:7px;font:650 11px var(--font-ui);cursor:pointer}.primary{border:1px solid var(--ink);background:var(--ink);color:#fff}.primary:hover{background:#363831}.primary:disabled{opacity:.5}.secondary{border:1px solid var(--border-strong);background:#fff;color:var(--text-2)}.summary-bar{min-height:76px;display:flex;align-items:center;justify-content:space-between;padding:14px 17px;margin-bottom:15px;background:#edf3ed;border:1px solid #d7e1d6;border-radius:11px}.summary-bar>div{display:flex;align-items:center;gap:12px}.summary-icon{width:40px;height:40px;display:grid;place-items:center;border-radius:9px;background:#dae8dc;color:#4e7958}.summary-bar strong{font-size:12px}.summary-bar p{font-size:10px;color:#657968;margin:3px 0 0}.summary-bar button{height:36px;padding:0 12px;display:flex;align-items:center;gap:7px;border:1px solid #c8d6c7;background:#fff;border-radius:8px;color:#526c56;font:650 10px var(--font-ui);cursor:pointer}.summary-bar button:disabled{opacity:.5}.rules-card{background:#fff;border:1px solid var(--border);border-radius:12px;overflow:hidden}.rules-header,.rules-card article{display:grid;grid-template-columns:100px minmax(140px,1.2fr) minmax(160px,1fr) minmax(150px,1fr) 60px 38px;align-items:center}.rules-header{height:38px;padding:0 15px;background:#f0f0ed;color:var(--text-3);text-transform:uppercase;letter-spacing:.08em;font-size:8px;font-weight:750}.rules-card article{min-height:72px;padding:8px 15px;border-top:1px solid var(--border);transition:opacity .16s}.rules-card article.disabled{opacity:.5}.priority{display:flex;align-items:center;gap:8px;color:#b1b0a9}.priority strong{color:var(--text-2);font-size:11px}.priority div{display:flex;flex-direction:column}.priority button{width:18px;height:16px;padding:0;border:0;background:transparent;color:var(--text-3);cursor:pointer}.priority button:disabled{opacity:.25}.rule-name,.condition,.action{display:flex;flex-direction:column;min-width:0}.rule-name strong,.action strong{font-size:11px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}.rule-name span,.condition span,.action span{font-size:9px;color:var(--text-3);margin-bottom:3px}.condition code{width:max-content;max-width:90%;padding:3px 6px;border-radius:5px;background:#f1f1ee;color:#5c5b55;font:10px var(--font-mono);overflow:hidden;text-overflow:ellipsis}.toggle,.more{border:0;background:transparent;cursor:pointer;color:#527c5b;padding:5px}.more{color:var(--text-3)}.rules-note{display:flex;gap:11px;padding:18px 5px;color:var(--text-3)}.rules-note strong{font-size:10px;color:var(--text-2)}.rules-note p{font-size:9px;margin:3px 0 0}.modal-backdrop{position:fixed;inset:0;z-index:100;background:rgba(19,20,18,.44);display:grid;place-items:center;padding:24px;backdrop-filter:blur(3px)}.modal{width:min(540px,100%);max-height:90vh;overflow:auto;background:#f8f8f5;border-radius:15px;padding:23px;box-shadow:0 28px 70px rgba(0,0,0,.2)}.modal header{display:flex;justify-content:space-between;align-items:flex-start;margin-bottom:21px}.modal header .eyebrow{font-size:8px;text-transform:uppercase;letter-spacing:.12em;color:var(--accent);font-weight:750;margin:0 0 4px}.modal h2{font:650 21px var(--font-display);margin:0}.modal header button{width:34px;height:34px;border:0;background:#ecece8;border-radius:8px;display:grid;place-items:center;cursor:pointer}.modal label{display:flex;flex-direction:column;gap:6px;font-size:10px;font-weight:650;color:var(--text-2);margin-bottom:14px}.field-grid{display:grid;grid-template-columns:1fr 1fr;gap:12px}.modal input,.modal select{height:40px;padding:0 11px;border:1px solid var(--border-strong);border-radius:8px;background:#fff;color:var(--ink);font:12px var(--font-ui);outline:none}.modal input:focus,.modal select:focus{border-color:var(--accent);box-shadow:0 0 0 3px rgba(199,92,55,.13)}.rule-sentence{padding:13px;border:1px dashed #d3d1c9;border-radius:8px;background:#f1f1ed;color:var(--text-2);font-size:10px;line-height:1.7}.rule-sentence span{color:var(--accent);font-size:8px;font-weight:800;margin-right:4px}.rule-sentence code{font:10px var(--font-mono)}.modal footer{display:flex;justify-content:flex-end;align-items:center;gap:8px;margin-top:20px}.modal-intro{font-size:11px;color:var(--text-2);margin:-10px 0 14px}.preview-modal{width:min(680px,100%)}.preview-list{max-height:390px;overflow:auto;background:#fff;border:1px solid var(--border);border-radius:10px}.preview-list>div{display:grid;grid-template-columns:40px minmax(0,1fr) auto minmax(150px,.8fr);gap:11px;align-items:center;padding:11px;border-top:1px solid var(--border)}.preview-list>div:first-child{border:0}.preview-list span{display:flex;flex-direction:column;min-width:0}.preview-list strong{font-size:10px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}.preview-list small{font-size:8px;color:var(--text-3);margin-top:3px}.safe-copy{display:flex;align-items:center;gap:5px;margin-right:auto;color:#58705b;font-size:9px}@media(max-width:800px){.rules-header{display:none}.rules-card article{grid-template-columns:54px 1fr 1fr 44px 30px}.rules-card .action{display:none}}@media(max-width:560px){.summary-bar{align-items:flex-start;gap:10px}.summary-bar button{font-size:0}.summary-bar button svg{font-size:initial}.rules-card article{grid-template-columns:42px 1fr 42px 28px}.rules-card .condition{display:none}.field-grid{grid-template-columns:1fr}.preview-list>div{grid-template-columns:40px 1fr}.preview-list>div>svg,.preview-list .destination{display:none}.safe-copy{display:none}}
  .primary{border-color:var(--primary-action-bg);background:var(--primary-action-bg);color:var(--primary-action-text)}.primary:hover{background:var(--primary-action-hover)}
  .primary:active{background:var(--primary-action-pressed)}.secondary{background:var(--surface)}.summary-bar{background:var(--success-bg);border-color:var(--success-border)}.summary-icon{background:var(--success-strong);color:var(--success-text)}.summary-bar p{color:var(--success-text)}.summary-bar button{border-color:var(--success-border);background:var(--surface);color:var(--success-text)}.rules-card{background:var(--surface)}.toggle{color:var(--accent-pressed)}.modal input:focus,.modal select:focus{box-shadow:0 0 0 3px var(--focus-ring)}
</style>
