<script lang="ts">
  export let text: string;
  export let placement: 'top' | 'bottom' | 'right' = 'top';
  export let disabled = false;
  export let block = false;

  let anchor: HTMLSpanElement;
  let visible = false;
  let x = 0;
  let y = 0;
  let timer: ReturnType<typeof setTimeout>;

  function position() {
    if (!anchor) return;
    const rect = anchor.getBoundingClientRect();
    if (placement === 'right') {
      x = rect.right + 9;
      y = rect.top + rect.height / 2;
    } else {
      x = rect.left + rect.width / 2;
      y = placement === 'bottom' ? rect.bottom + 9 : rect.top - 9;
    }
  }

  function show(event: FocusEvent | MouseEvent) {
    if (disabled || !text) return;
    clearTimeout(timer);
    position();
    timer = setTimeout(() => (visible = true), event.type === 'focusin' ? 0 : 260);
  }

  function hide() {
    clearTimeout(timer);
    visible = false;
  }
</script>

<svelte:window on:resize={hide} on:scroll={hide} />

<span
  class:block
  class="tooltip-anchor"
  bind:this={anchor}
  on:mouseenter={show}
  on:mouseleave={hide}
  on:focusin={show}
  on:focusout={hide}
  on:click={hide}
  role="presentation"
>
  <slot />
</span>

{#if visible}
  <span
    class:top={placement === 'top'}
    class:bottom={placement === 'bottom'}
    class:right={placement === 'right'}
    class="sift-tooltip"
    style={`left:${x}px;top:${y}px`}
    role="tooltip">{text}</span
  >
{/if}

<style>
  .tooltip-anchor {
    display: inline-flex;
    min-width: 0;
    max-width: 100%;
  }
  .tooltip-anchor.block {
    display: flex;
    width: 100%;
  }
  .sift-tooltip {
    position: fixed;
    z-index: 500;
    max-width: 260px;
    padding: 7px 9px;
    border: 1px solid var(--border-strong);
    border-radius: 7px;
    background: var(--tooltip-bg);
    color: var(--tooltip-text);
    box-shadow: var(--shadow-lg);
    font: 600 10px/1.35 var(--font-ui);
    white-space: normal;
    overflow-wrap: anywhere;
    pointer-events: none;
    animation: tooltip-in 0.14s ease-out;
  }
  .sift-tooltip.top {
    transform: translate(-50%, -100%);
  }
  .sift-tooltip.bottom {
    transform: translateX(-50%);
  }
  .sift-tooltip.right {
    transform: translateY(-50%);
  }
  @keyframes tooltip-in {
    from {
      opacity: 0;
      transform: translate(-50%, calc(-100% + 3px));
    }
  }
  .sift-tooltip.bottom {
    animation-name: tooltip-in-bottom;
  }
  @keyframes tooltip-in-bottom {
    from {
      opacity: 0;
      transform: translate(-50%, -3px);
    }
  }
  .sift-tooltip.right {
    animation-name: tooltip-in-right;
  }
  @keyframes tooltip-in-right {
    from {
      opacity: 0;
      transform: translate(-3px, -50%);
    }
  }
</style>
