<script lang="ts">
  import { page } from '$app/stores';
  import { align } from '$lib/align';

  export let data;

  $: removed = new Set(data.item.images.removed);
  $: aligned = align(data.item, data.text);

  function imageRange([start, end]: number[]) {
    if (start == end) {
      if (removed.has(start)) {
        return [];
      } else {
        return [start];
      }
    } else {
      return Array.from({ length: end - start + 1 }, (_, i) => start + i).filter(
        (i) => !removed.has(i)
      );
    }
  }
</script>

<main class="p-4 mx-auto flex flex-col items-center">
  <div class="flex flex-col">
    <h1 class="text-xl">{data.item.title}</h1>
  </div>
  <div class="grid grid-cols-[auto] gap-x-4 gap-y-2 mt-4 font-serif">
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
