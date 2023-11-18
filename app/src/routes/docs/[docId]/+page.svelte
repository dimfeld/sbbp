<script lang="ts">
  import { page } from '$app/stores';
  import { align } from '$lib/align';

  export let data;

  let showRemoved = false;

  $: removed = new Set(data.item.images.removed);
  $: aligned = align(data.item, data.text);

  function imageRange([start, end]: number[], showRemoved: boolean) {
    if (start == end) {
      if (showRemoved || !removed.has(start)) {
        return [start];
      } else {
        return [];
      }
    } else {
      let range = Array.from({ length: end - start + 1 }, (_, i) => start + i);

      if (!showRemoved) {
        range = range.filter((i) => !removed.has(i));
      }

      return range;
    }
  }

  let largeImage: number | null = null;
</script>

<label class="fixed right-2 top-2 z-10">
  <input type="checkbox" bind:checked={showRemoved} />
  Show removed images
</label>

<main class="relative p-4 mx-auto flex flex-col items-center">
  <header class="flex flex-col">
    <h1 class="text-xl">{data.item.title}</h1>
  </header>

  {#if data.item.summary}
    <section>
      <p class="text-lg">Video Summary</p>
      <p class="whitespace-pre-wrap font-serif leading-relaxed max-w-[90ch]">{data.item.summary}</p>
    </section>
  {/if}

  <div class="grid grid-cols-[auto_auto] gap-x-4 gap-y-2 mt-8 font-serif leading-relaxed">
    {#each aligned as chunk}
      <div class="max-w-[65ch]">{chunk.text}</div>
      <div class="flex flex-col gap-2 max-w-lg">
        {#each imageRange(chunk.images, showRemoved) as image}
          <button type="button" on:click={() => (largeImage = image)}>
            <img
              class="object-cover"
              src="/docs/{$page.params.docId}/image/{image}"
              alt="Image {image}"
              loading="lazy"
            />
          </button>
        {/each}
      </div>
    {/each}
  </div>
</main>

{#if largeImage}
  <button class="fixed inset-0 z-50" on:click={() => (largeImage = null)}>
    <img src="/docs/{$page.params.docId}/image/{largeImage}" alt="Image {largeImage}" />
  </button>
{/if}
