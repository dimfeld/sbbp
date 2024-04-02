<script lang="ts">
  import { page } from '$app/stores';
  import { align } from '$lib/align';
  import DocSettings from './DocSettings.svelte';

  const { data } = $props();

  let showRemoved = $state(false);

  let removed = $derived(new Set(data.item.images.removed));
  let aligned = $derived(align(data.item));

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

  let largeImage: number | null = $state(null);

  function onkeydown(event: KeyboardEvent) {
    if (!largeImage) {
      return;
    }
    if (event.key === 'Escape') {
      largeImage = null;
    } else if (event.key === 'ArrowLeft') {
      largeImage = Math.max(1, largeImage - 1);
    } else if (event.key === 'ArrowRight') {
      largeImage = Math.min(data.item.images.max_index, largeImage + 1);
    }
  }
</script>

<svelte:window {onkeydown} />

<main class="relative p-4 w-full overflow-y-auto flex flex-col items-center">
  <label class="sticky w-full top-0 left-2 z-10">
    <input type="checkbox" bind:checked={showRemoved} />
    Show removed images
  </label>

  <header
    class="flex items-start md:items-center justify-start md:justify-between gap-4 w-full flex-col md:flex-row"
  >
    <h1 class="text-3xl">{data.item.title}</h1>
    <DocSettings read={data.item.read} />
  </header>

  {#if data.item.summary}
    <section>
      <p class="text-2xl">Video Summary</p>
      <p class="whitespace-pre-wrap font-serif text-xl leading-relaxed max-w-[90ch]">
        {data.item.summary}
      </p>
    </section>
  {/if}

  <div
    class="grid lg:grid-cols-[auto_auto] grid-cols-1 gap-x-4 gap-y-2 mt-8 font-serif text-xl leading-relaxed"
  >
    {#each aligned as chunk}
      <div class="max-w-[65ch]">{chunk.text}</div>
      <div class="flex flex-col gap-2 max-w-lg">
        {#each imageRange(chunk.images, showRemoved) as image}
          <button type="button" on:click={() => (largeImage = image)}>
            <img
              class="object-cover aspect-video border"
              class:border-red-500={removed.has(image)}
              src="/api/videos/{$page.params.docId}/image/{image}"
              alt="Image {image}"
              loading="lazy"
            />
          </button>
        {/each}
      </div>
    {/each}
  </div>

  <div class="flex self-start mt-8">
    <DocSettings read={data.item.read} />
  </div>
</main>

{#if largeImage}
  <button class="fixed inset-0 z-50" on:click={() => (largeImage = null)}>
    <img src="/api/videos/{$page.params.docId}/image/{largeImage}" alt="Image {largeImage}" />
  </button>
{/if}
