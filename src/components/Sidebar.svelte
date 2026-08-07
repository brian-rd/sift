<script lang="ts">
  import { LayoutDashboard, Inbox, ListChecks, History, Settings, Sparkles, FolderOpen, ShieldCheck } from '@lucide/svelte';
  import type { Screen } from '../lib/types';

  export let active: Screen;
  export let onNavigate: (screen: Screen) => void;
  export let reviewCount: number;
  export let compact = false;

  const items: { id: Screen; label: string; icon: typeof LayoutDashboard }[] = [
    { id: 'dashboard', label: 'Overview', icon: LayoutDashboard },
    { id: 'sift', label: 'Sift', icon: Inbox },
    { id: 'rules', label: 'Rules', icon: ListChecks },
    { id: 'history', label: 'History', icon: History },
    { id: 'settings', label: 'Settings', icon: Settings }
  ];
</script>

<aside class:compact aria-label="Main navigation">
  <div class="brand">
    <span class="brand-mark" aria-hidden="true"><Sparkles size={18} strokeWidth={2.4} /></span>
    <span class="brand-name">Sift</span>
  </div>

  <nav>
    {#each items as item}
      <button class:active={active === item.id} on:click={() => onNavigate(item.id)} aria-current={active === item.id ? 'page' : undefined} title={compact ? item.label : undefined}>
        <svelte:component this={item.icon} size={18} strokeWidth={2} />
        <span>{item.label}</span>
        {#if item.id === 'sift' && reviewCount > 0}<span class="count">{reviewCount}</span>{/if}
      </button>
    {/each}
  </nav>

  <div class="sidebar-bottom">
    <div class="folder-card">
      <FolderOpen size={17} />
      <div><strong>Downloads</strong><span>Watching now</span></div>
      <span class="status-dot" title="Active"></span>
    </div>
    <div class="trust-note"><ShieldCheck size={15} /><span>Nothing moves without you</span></div>
  </div>
</aside>

<style>
  aside{width:236px;min-width:236px;height:100vh;padding:24px 14px 18px;background:#ecebe6;border-right:1px solid var(--border);display:flex;flex-direction:column;position:sticky;top:0}.brand{display:flex;align-items:center;gap:11px;padding:0 8px 28px}.brand-mark{width:34px;height:34px;display:grid;place-items:center;border-radius:10px;color:#fff;background:var(--ink)}.brand-name{font-family:var(--font-display);font-size:22px;font-weight:650;letter-spacing:-.02em}nav{display:flex;flex-direction:column;gap:4px}nav button{width:100%;height:42px;display:flex;align-items:center;gap:11px;padding:0 11px;border:0;border-radius:9px;color:var(--text-2);background:transparent;font:inherit;font-weight:550;cursor:pointer;transition:background .16s,color .16s}nav button:hover{background:rgba(255,255,255,.65);color:var(--ink)}nav button.active{background:#fff;color:var(--ink);box-shadow:0 1px 2px rgba(28,28,25,.06)}nav button .count{margin-left:auto;min-width:22px;padding:2px 7px;border-radius:999px;background:var(--ink);color:white;font-size:11px;text-align:center}.sidebar-bottom{margin-top:auto}.folder-card{display:flex;align-items:center;gap:10px;padding:12px;border-radius:10px;background:rgba(255,255,255,.55);border:1px solid rgba(34,34,30,.07)}.folder-card div{display:flex;flex-direction:column;min-width:0}.folder-card strong{font-size:12px}.folder-card span{font-size:10px;color:var(--text-3)}.folder-card .status-dot{width:7px;height:7px;border-radius:50%;background:var(--green);margin-left:auto}.trust-note{display:flex;align-items:center;gap:7px;color:var(--text-3);font-size:10px;padding:14px 8px 0}@media(max-width:820px){aside{width:72px;min-width:72px;padding-inline:10px}.brand-name,nav button>span,.folder-card div,.trust-note span{display:none}.brand{padding-inline:9px}.brand-mark{width:34px}.folder-card{justify-content:center;padding:12px 8px}.folder-card .status-dot{position:absolute;margin:22px 0 0 22px}.trust-note{justify-content:center}nav button{justify-content:center;padding:0}.compact{width:72px;min-width:72px}}@media(max-width:560px){aside{position:fixed;bottom:0;top:auto;left:0;width:100%;height:64px;z-index:50;padding:7px 10px;border-right:0;border-top:1px solid var(--border)}.brand,.sidebar-bottom{display:none}nav{flex-direction:row;justify-content:space-around}nav button{width:54px;height:48px;flex-direction:column;gap:2px;font-size:9px}nav button>span{display:block}nav button .count{position:absolute;margin:-22px 0 0 27px;padding:1px 5px;min-width:17px}}
</style>
