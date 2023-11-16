<script lang="ts">
  import { align } from '$lib/align';
  import Viewer from './Viewer.svelte';

  export let data;

  $: processedData = align(data.file);
</script>

{#if !data.file}
  <p>Select a config file from the query string</p>
{:else}
  <Viewer data={processedData} />
{/if}
