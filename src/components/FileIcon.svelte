<script lang="ts">
  import { FileText, FileArchive, FileVideo, FileAudio, FileImage, File, FileSpreadsheet } from '@lucide/svelte';
  import type { FileKind } from '../lib/types';
  export let kind: FileKind;
  export let extension = '';
  export let size = 20;
  $: icon = kind === 'image' ? FileImage : kind === 'video' ? FileVideo : kind === 'audio' ? FileAudio : kind === 'archive' ? FileArchive : kind === 'text' && ['csv','xls','xlsx'].includes(extension) ? FileSpreadsheet : kind === 'pdf' || kind === 'text' ? FileText : File;
</script>
<span class="file-icon {kind}" aria-hidden="true"><svelte:component this={icon} {size} strokeWidth={1.8} /></span>

<style>
  .file-icon{display:inline-grid;place-items:center;width:40px;height:40px;border-radius:9px;background:#ececea;color:#575752;flex:none}.image{background:#e8f1ea;color:#3c7450}.pdf{background:#f4e8e4;color:#a44f3e}.archive{background:#eee8f3;color:#795998}.video{background:#e8edf5;color:#506d94}.audio{background:#f1e9dc;color:#9a6b2b}.text{background:#e5efee;color:#33746d}
</style>
