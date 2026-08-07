<script lang="ts">
  import {
    FileText,
    FileArchive,
    FileVideo,
    FileAudio,
    FileImage,
    File,
    FileSpreadsheet,
  } from '@lucide/svelte';
  import type { FileKind } from '../lib/types';
  export let kind: FileKind;
  export let extension = '';
  export let size = 20;
  $: icon =
    kind === 'image'
      ? FileImage
      : kind === 'video'
        ? FileVideo
        : kind === 'audio'
          ? FileAudio
          : kind === 'archive'
            ? FileArchive
            : kind === 'text' && ['csv', 'xls', 'xlsx'].includes(extension)
              ? FileSpreadsheet
              : kind === 'pdf' || kind === 'text'
                ? FileText
                : File;
</script>

<span class="file-icon {kind}" aria-hidden="true"
  ><svelte:component this={icon} {size} strokeWidth={1.8} /></span
>

<style>
  .file-icon {
    display: inline-grid;
    place-items: center;
    width: 40px;
    height: 40px;
    border-radius: 9px;
    background: var(--file-icon-bg);
    color: var(--file-icon-color);
    flex: none;
  }
  .image {
    background: var(--file-image-bg);
    color: var(--file-image-color);
  }
  .pdf {
    background: var(--file-pdf-bg);
    color: var(--file-pdf-color);
  }
  .archive {
    background: var(--file-archive-bg);
    color: var(--file-archive-color);
  }
  .video {
    background: var(--file-video-bg);
    color: var(--file-video-color);
  }
  .audio {
    background: var(--file-audio-bg);
    color: var(--file-audio-color);
  }
  .text {
    background: var(--file-text-bg);
    color: var(--file-text-color);
  }
</style>
