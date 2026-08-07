<script lang="ts">
  import { LayoutDashboard, Inbox, ListChecks, History, Settings, Sparkles, FolderOpen } from '@lucide/svelte';
  import type { Screen } from '../lib/types';
  import Tooltip from './Tooltip.svelte';

  export let active: Screen;
  export let onNavigate: (screen: Screen) => void;
  export let onWatchedFolder: () => void;
  export let reviewCount: number;
  export let watchEnabled: boolean;
  export let compact = false;

  const items: { id: Screen; label: string; icon: typeof LayoutDashboard }[] = [
    { id: 'dashboard', label: 'Overview', icon: LayoutDashboard },
    { id: 'sift', label: 'Sift', icon: Inbox },
    { id: 'rules', label: 'Rules', icon: ListChecks },
    { id: 'history', label: 'History', icon: History }
  ];
</script>

<aside class:compact aria-label="Main navigation">
  <div class="brand">
    <span class="brand-mark" aria-hidden="true"><Sparkles size={18} strokeWidth={2.4} /></span>
    <span class="brand-name">Sift</span>
  </div>

  <nav>
    {#each items as item}
      <Tooltip text={item.label} placement="right" block>
        <button class:active={active === item.id} on:click={() => onNavigate(item.id)} aria-current={active === item.id ? 'page' : undefined}>
          <svelte:component this={item.icon} size={18} strokeWidth={2} />
          <span>{item.label}</span>
          {#if item.id === 'sift' && reviewCount > 0}<span class="count">{reviewCount}</span>{/if}
        </button>
      </Tooltip>
    {/each}
    <button class="mobile-settings" class:active={active === 'settings'} on:click={() => onNavigate('settings')} aria-current={active === 'settings' ? 'page' : undefined}>
      <Settings size={18} strokeWidth={2} /><span>Settings</span>
    </button>
  </nav>

  <div class="sidebar-bottom">
    <Tooltip text={watchEnabled ? 'Change watched folder · Watching for new files' : 'Change watched folder · Automatic watching paused'} placement="right" block>
      <button class="folder-card" on:click={onWatchedFolder}>
        <FolderOpen size={17} />
        <div><strong>Downloads</strong><span>{watchEnabled ? 'Watching now' : 'Manual scans'}</span></div>
        <span class:paused={!watchEnabled} class="status-dot"></span>
      </button>
    </Tooltip>
    <button class:active={active === 'settings'} class="settings-button" on:click={() => onNavigate('settings')} aria-current={active === 'settings' ? 'page' : undefined}>
      <Settings size={17} /><span>Settings</span>
    </button>
  </div>
</aside>

<style>
  aside{width:236px;min-width:236px;height:100vh;padding:24px 14px 18px;background:var(--sidebar);border-right:1px solid var(--border);display:flex;flex-direction:column;position:sticky;top:0}.brand{display:flex;align-items:center;gap:11px;padding:0 8px 28px}.brand-mark{width:34px;height:34px;display:grid;place-items:center;border-radius:10px;color:var(--accent-contrast);background:var(--accent)}.brand-name{font-family:var(--font-display);font-size:22px;font-weight:650;letter-spacing:-.02em}nav{display:flex;flex-direction:column;gap:4px}nav :global(.tooltip-anchor){width:100%}nav button,.settings-button{width:100%;height:42px;display:flex;align-items:center;gap:11px;padding:0 11px;border:0;border-radius:9px;color:var(--text-2);background:transparent;font:inherit;font-weight:550;cursor:pointer;transition:background .16s,color .16s}nav button:hover,.settings-button:hover{background:var(--surface-2);color:var(--ink)}nav button.active,.settings-button.active{background:var(--surface);color:var(--ink);box-shadow:0 1px 2px rgba(13,20,17,.08)}nav button .count{margin-left:auto;min-width:22px;padding:2px 7px;border-radius:999px;background:var(--accent);color:var(--accent-contrast);font-size:11px;text-align:center}.mobile-settings{display:none}.sidebar-bottom{margin-top:auto;display:flex;flex-direction:column;gap:5px}.folder-card{display:flex;align-items:center;gap:10px;padding:12px;border-radius:10px;background:var(--surface);border:1px solid var(--border)}.folder-card div{display:flex;flex-direction:column;min-width:0}.folder-card strong{font-size:12px}.folder-card span{font-size:10px;color:var(--text-3)}.folder-card .status-dot{width:7px;height:7px;border-radius:50%;background:var(--green);margin-left:auto}.folder-card .status-dot.paused{background:var(--text-3)}.settings-button{margin-top:2px}@media(max-width:820px){aside{width:72px;min-width:72px;padding-inline:10px}.brand-name,nav button>span,.folder-card div,.settings-button span{display:none}.brand{padding-inline:9px}.brand-mark{width:34px}.folder-card{justify-content:center;padding:12px 8px}.folder-card .status-dot{position:absolute;margin:22px 0 0 22px}nav button,.settings-button{justify-content:center;padding:0}.compact{width:72px;min-width:72px}}@media(max-width:560px){aside{position:fixed;bottom:0;top:auto;left:0;width:100%;height:64px;z-index:50;padding:7px 10px;border-right:0;border-top:1px solid var(--border)}.brand,.sidebar-bottom{display:none}.mobile-settings{display:flex}nav{flex-direction:row;justify-content:space-around}nav :global(.tooltip-anchor){width:54px}nav button{width:54px;height:48px;flex-direction:column;gap:2px;font-size:9px}nav button>span{display:block}nav button .count{position:absolute;margin:-22px 0 0 27px;padding:1px 5px;min-width:17px}}
  .folder-card{width:100%;color:inherit;text-align:left;cursor:pointer}.folder-card:hover{border-color:var(--border-strong)}
  nav button:focus-visible,.settings-button:focus-visible,.folder-card:focus-visible{outline:none;box-shadow:inset 0 0 0 2px var(--accent-pressed)}
</style>
