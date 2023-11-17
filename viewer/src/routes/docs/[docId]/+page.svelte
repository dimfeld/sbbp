<script lang="ts">
  import { page } from '$app/stores';
  import { align } from '$lib/align';

  export let data;

  $: aligned = align(data.item, data.text);
</script>

<div class="flex flex-col">
  <h1>{data.item.title}</h1>
</div>
<div class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2">
  {#each aligned as chunk}
    <div class="max-w-[65ch]">{chunk.text}</div>
    <div class="flex flex-col gap-2">
      {#each chunk.images as image}
        <img src="/docs/{$page.params.docId}/images/{image}" alt="Image {image}" loading="lazy" />
      {/each}
    </div>
  {/each}
</div>
