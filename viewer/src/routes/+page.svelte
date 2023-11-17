<script lang="ts">
  import { enhance } from '$app/forms';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';

  export let data;
</script>

<main class="p-4">
  <form method="POST" use:enhance class="flex flex-col gap-2 rounded-lg border border-border p-4">
    <Label class="flex gap-2 flex-1 max-w-[100ch] text-base" for="path">Add new path</Label>
    <div class="flex gap-2">
      <Input type="text" name="path" class="flex-1" />
      <Button type="submit">Add</Button>
    </div>
  </form>

  <ul class="flex flex-col gap-4">
    {#each data.items as item (item.id)}
      <li><a href="/docs/{item.id}" class="underline">{item.title}</a> - {item.duration}s</li>
    {/each}
  </ul>
</main>
