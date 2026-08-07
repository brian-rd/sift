<script lang="ts">
  import { RefreshCw } from '@lucide/svelte';
  export let eyebrow = '';
  export let title: string;
  export let description = '';
  export let scanning = false;
  export let onScan: (() => void) | undefined = undefined;
</script>

<header class="page-header">
  <div>
    {#if eyebrow}<p class="eyebrow">{eyebrow}</p>{/if}
    <h1>{title}</h1>
    {#if description}<p class="description">{description}</p>{/if}
  </div>
  {#if onScan}
    <button class="scan-button" on:click={onScan} disabled={scanning}>
      <span class:spinning={scanning}><RefreshCw size={15} /></span> {scanning ? 'Scanning…' : 'Scan again'}
    </button>
  {/if}
  <slot />
</header>

<style>
  .page-header{display:flex;align-items:flex-start;justify-content:space-between;gap:24px;margin-bottom:28px}.eyebrow{margin:0 0 6px;text-transform:uppercase;letter-spacing:.13em;font-size:10px;font-weight:750;color:var(--text-3)}h1{margin:0;font:650 30px/1.15 var(--font-display);letter-spacing:-.035em;color:var(--ink)}.description{margin:8px 0 0;color:var(--text-2);font-size:13px}.scan-button{flex:none;display:flex;align-items:center;gap:7px;padding:9px 12px;background:#fff;border:1px solid var(--border-strong);border-radius:8px;color:var(--text-2);font:600 12px var(--font-ui);cursor:pointer}.scan-button:hover{border-color:#b4b2ab;color:var(--ink)}.scan-button:disabled{opacity:.6}.scan-button>span{display:grid;place-items:center}.spinning{animation:spin .8s linear infinite}@keyframes spin{to{transform:rotate(360deg)}}@media(max-width:560px){.page-header{margin-bottom:20px}h1{font-size:26px}.description{max-width:280px}}
</style>
