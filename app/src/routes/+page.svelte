<script lang="ts">
  import { enhance } from '$app/forms';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { XIcon as DeleteIcon, RefreshCcwIcon as RefreshIcon } from 'lucide-svelte';

  export let data;
  export let form;
</script>

<main class="relative p-4">
  <form
    method="POST"
    action="?/add"
    use:enhance
    class="flex flex-col gap-2 rounded-lg border border-border p-4"
  >
    <Label class="flex gap-2 flex-1 max-w-[100ch] text-base" for="path">Add new path</Label>
    <div class="flex gap-2">
      <Input type="text" name="path" class="flex-1" autocomplete="off" />
      <Button type="submit">Add</Button>
    </div>
  </form>

  {#if form?.error}
    <p class="text-red-500">{form.error}</p>
  {/if}

  <ul class="flex flex-col gap-4 mt-4">
    {#each data.items as item (item.id)}
      <li class="flex justify-between">
        <p>
          <a href="/docs/{item.id}" class="underline">{item.title}</a> - {item.duration}s
        </p>
        <p></p>
        <form method="POST" action="?/refresh" class="flex gap-2" use:enhance>
          <input type="hidden" name="id" value={item.id} />
          <Button variant="outline" size="icon" type="submit" formaction="?/refresh">
            <RefreshIcon />
          </Button>
          <Button variant="outline" size="icon" type="submit" formaction="?/delete">
            <DeleteIcon />
          </Button>
        </form>
      </li>
    {/each}
  </ul>
</main>
