<script lang="ts">
  export let source: string;

  type InlineToken = { type: 'text' | 'strong' | 'em' | 'code' | 'link'; text: string; href?: string };
  type Block = {
    type: 'heading' | 'paragraph' | 'quote' | 'code' | 'list' | 'ordered' | 'rule';
    level?: number;
    text?: string;
    items?: string[];
  };

  function inline(text: string): InlineToken[] {
    const tokens: InlineToken[] = [];
    const pattern = /(\*\*[^*]+\*\*|`[^`]+`|\*[^*]+\*|\[[^\]]+\]\([^)]+\))/g;
    let cursor = 0;
    for (const match of text.matchAll(pattern)) {
      const index = match.index ?? 0;
      if (index > cursor) tokens.push({ type: 'text', text: text.slice(cursor, index) });
      const value = match[0];
      if (value.startsWith('**')) tokens.push({ type: 'strong', text: value.slice(2, -2) });
      else if (value.startsWith('`')) tokens.push({ type: 'code', text: value.slice(1, -1) });
      else if (value.startsWith('*')) tokens.push({ type: 'em', text: value.slice(1, -1) });
      else {
        const link = /^\[([^\]]+)\]\(([^)]+)\)$/.exec(value);
        const href = link?.[2] ?? '';
        tokens.push(
          /^https?:\/\//i.test(href)
            ? { type: 'link', text: link?.[1] ?? value, href }
            : { type: 'text', text: link?.[1] ?? value },
        );
      }
      cursor = index + value.length;
    }
    if (cursor < text.length) tokens.push({ type: 'text', text: text.slice(cursor) });
    return tokens;
  }

  function parseMarkdown(markdown: string): Block[] {
    const lines = markdown.replace(/\r\n?/g, '\n').split('\n');
    const blocks: Block[] = [];
    for (let index = 0; index < lines.length;) {
      const line = lines[index];
      if (!line.trim()) {
        index += 1;
        continue;
      }
      if (/^```/.test(line.trim())) {
        const code: string[] = [];
        index += 1;
        while (index < lines.length && !/^```/.test(lines[index].trim())) code.push(lines[index++]);
        index += index < lines.length ? 1 : 0;
        blocks.push({ type: 'code', text: code.join('\n') });
        continue;
      }
      const heading = /^(#{1,6})\s+(.+)$/.exec(line);
      if (heading) {
        blocks.push({ type: 'heading', level: heading[1].length, text: heading[2] });
        index += 1;
        continue;
      }
      if (/^\s*[-*_]{3,}\s*$/.test(line)) {
        blocks.push({ type: 'rule' });
        index += 1;
        continue;
      }
      if (/^\s*[-*+]\s+/.test(line)) {
        const items: string[] = [];
        while (index < lines.length && /^\s*[-*+]\s+/.test(lines[index]))
          items.push(lines[index++].replace(/^\s*[-*+]\s+/, ''));
        blocks.push({ type: 'list', items });
        continue;
      }
      if (/^\s*\d+\.\s+/.test(line)) {
        const items: string[] = [];
        while (index < lines.length && /^\s*\d+\.\s+/.test(lines[index]))
          items.push(lines[index++].replace(/^\s*\d+\.\s+/, ''));
        blocks.push({ type: 'ordered', items });
        continue;
      }
      if (/^>\s?/.test(line)) {
        const quote: string[] = [];
        while (index < lines.length && /^>\s?/.test(lines[index]))
          quote.push(lines[index++].replace(/^>\s?/, ''));
        blocks.push({ type: 'quote', text: quote.join(' ') });
        continue;
      }
      const paragraph = [line.trim()];
      index += 1;
      while (
        index < lines.length &&
        lines[index].trim() &&
        !/^(#{1,6})\s+|^```|^\s*[-*+]\s+|^\s*\d+\.\s+|^>\s?/.test(lines[index])
      )
        paragraph.push(lines[index++].trim());
      blocks.push({ type: 'paragraph', text: paragraph.join(' ') });
    }
    return blocks;
  }

  $: blocks = parseMarkdown(source);
</script>

{#each blocks as block}
  {#if block.type === 'heading'}
    {#if block.level === 1}<h1>{block.text}</h1>{:else if block.level === 2}<h2>{block.text}</h2>{:else}<h3>
        {block.text}
      </h3>{/if}
  {:else if block.type === 'paragraph' || block.type === 'quote'}
    <svelte:element this={block.type === 'quote' ? 'blockquote' : 'p'}
      >{#each inline(block.text ?? '') as token}{#if token.type === 'strong'}<strong>{token.text}</strong
          >{:else if token.type === 'em'}<em>{token.text}</em>{:else if token.type === 'code'}<code
            >{token.text}</code
          >{:else if token.type === 'link'}<a href={token.href} target="_blank" rel="noreferrer"
            >{token.text}</a
          >{:else}{token.text}{/if}{/each}</svelte:element
    >
  {:else if block.type === 'code'}
    <pre><code>{block.text}</code></pre>
  {:else if block.type === 'list' || block.type === 'ordered'}
    <svelte:element this={block.type === 'ordered' ? 'ol' : 'ul'}
      >{#each block.items ?? [] as item}<li>
          {#each inline(item) as token}{#if token.type === 'strong'}<strong>{token.text}</strong
              >{:else if token.type === 'em'}<em>{token.text}</em>{:else if token.type === 'code'}<code
                >{token.text}</code
              >{:else if token.type === 'link'}<a href={token.href} target="_blank" rel="noreferrer"
                >{token.text}</a
              >{:else}{token.text}{/if}{/each}
        </li>{/each}</svelte:element
    >
  {:else}<hr />{/if}
{/each}

<style>
  :global(.markdown-preview h1),
  :global(.markdown-preview h2),
  :global(.markdown-preview h3) {
    font-family: var(--font-display);
    line-height: 1.25;
    color: var(--ink);
  }
  :global(.markdown-preview h1) {
    font-size: 24px;
    margin: 0 0 18px;
  }
  :global(.markdown-preview h2) {
    font-size: 19px;
    margin: 24px 0 10px;
  }
  :global(.markdown-preview h3) {
    font-size: 15px;
    margin: 20px 0 8px;
  }
  :global(.markdown-preview p),
  :global(.markdown-preview li),
  :global(.markdown-preview blockquote) {
    font-size: 12px;
    line-height: 1.72;
    color: var(--text-2);
  }
  :global(.markdown-preview p) {
    margin: 0 0 13px;
  }
  :global(.markdown-preview ul),
  :global(.markdown-preview ol) {
    margin: 8px 0 16px;
    padding-left: 24px;
  }
  :global(.markdown-preview li) {
    margin: 4px 0;
  }
  :global(.markdown-preview blockquote) {
    margin: 14px 0;
    padding: 9px 13px;
    border-left: 3px solid var(--accent);
    background: var(--surface-2);
  }
  :global(.markdown-preview code) {
    padding: 2px 4px;
    border-radius: 4px;
    background: var(--surface-3);
    font: 11px var(--font-mono);
  }
  :global(.markdown-preview pre) {
    overflow: auto;
    padding: 14px;
    border-radius: 8px;
    background: var(--surface-2);
  }
  :global(.markdown-preview pre code) {
    padding: 0;
    background: transparent;
    line-height: 1.6;
  }
  :global(.markdown-preview a) {
    color: var(--accent);
  }
  :global(.markdown-preview hr) {
    border: 0;
    border-top: 1px solid var(--border);
    margin: 20px 0;
  }
</style>
