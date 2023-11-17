<script lang="ts">
  import { enhance } from '$app/forms';
  import { align } from '$lib/align';
  import Viewer from './Viewer.svelte';

  export let data;

  $: processedData = align(data.file);
</script>

<form use:enhance>
  <label>
    <p>File path</p>
    <input type="text" name="path" />
  </label>
</form>
