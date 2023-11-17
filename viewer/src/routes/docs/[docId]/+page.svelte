<script lang="ts">
  import { page } from '$app/stores';
  import { align } from '$lib/align';

  export let data;

  $: aligned = align(data.item, data.text);
  $: console.log(aligned);

  function imageRange([start, end]: number[]) {
    if (start == end) {
      return [start];
    } else {
      return Array.from({ length: end - start + 1 }, (_, i) => start + i);
    }
  }
</script>

<main class="p-4">
  <div class="flex flex-col">
    <h1>{data.item.title}</h1>
  </div>
  <div class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2">
    {#each aligned as chunk}
      <div class="max-w-[65ch]">{chunk.text}</div>
      <div class="flex flex-col gap-2 max-w-lg">
        {#each imageRange(chunk.images) as image}
          <img
            class="object-cover"
            src="/docs/{$page.params.docId}/image/{image}"
            alt="Image {image}"
            loading="lazy"
          />
        {/each}
      </div>
    {/each}
  </div>
</main>
