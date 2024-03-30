<script lang="ts">
  import { browser } from '$app/environment';
  import { enhance } from '$app/forms';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Switch } from '$lib/components/ui/switch';
  import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
  import {
    XIcon as DeleteIcon,
    RefreshCcwIcon as RefreshIcon,
    SettingsIcon,
    MailOpenIcon as ReadIcon,
    MailIcon as UnreadIcon,
  } from 'lucide-svelte';
  import { formatDuration } from '$lib/format.js';
  import { invalidate } from '$app/navigation';

  const { data, form } = $props();

  let unreadOnly = true;

  let timer: number | null = null;
  $effect(() => {
    if (
      browser &&
      !timer &&
      data.items.some((item) => (item.viewerData.processStatus ?? 'complete') !== 'complete')
    ) {
      timer = window.setTimeout(() => {
        timer = null;
        invalidate('resource://items');
      }, 5000);
    }
  });

  let items = $derived(
    unreadOnly ? data.items.filter((item) => !item.viewerData.read) : data.items
  );
</script>

<main class="relative p-4 flex flex-col gap-4">
  <form
    method="POST"
    action="?/download"
    use:enhance
    class="flex flex-col gap-2 rounded-lg border border-border p-4"
  >
    <Label class="flex gap-2 flex-1 max-w-[100ch] text-base" for="path">Add new video</Label>
    <div class="flex gap-2">
      <Input type="text" name="url" class="flex-1" autocomplete="off" />
      <Button type="submit">Add</Button>
    </div>
  </form>

  {#if form?.error}
    <p class="text-red-500">{form.error}</p>
  {/if}

  <div class="flex justify-end gap-2">
    <div class="flex items-center gap-2">
      <Switch id="unread-only" bind:checked={unreadOnly} />
      <Label for="unread-only">Unread only</Label>
    </div>
  </div>

  <ul class="flex flex-col gap-4">
    {#each items as item (item.id)}
      {@const read = item.viewerData.read}
      {@const ready = (item.viewerData.processStatus ?? 'complete') === 'complete'}
      <li class="flex justify-between">
        <p class="flex flex-col">
          {#if ready}
            <a href="/docs/{item.id}" class="underline">{item.title}</a>
            <span>
              {formatDuration(item.duration)}
            </span>
          {:else}
            <p>{item.originalVideoPath}</p>
            {#if item.viewerData.processStatus === 'error' && item.process?.error}
              <p class="text-red-500">{item.process.error}</p>
            {:else}
              <p>{item.viewerData.processStatus || ''}</p>
            {/if}
          {/if}
        </p>

        <div class="flex ga-2">
          <form method="POST" action="?/mark_read" use:enhance>
            <input type="hidden" name="id" value={item.id} />
            <input type="hidden" name="new_read" value={!read} />
            {#if ready}
              <Button
                variant="outline"
                size="icon"
                type="submit"
                formaction="?/mark_read"
                aria-label="Mark {read ? 'Unread' : 'Read'}"
              >
                {#if read}
                  <ReadIcon />
                {:else}
                  <UnreadIcon />
                {/if}
              </Button>
            {/if}
          </form>

          <DropdownMenu.Root>
            <DropdownMenu.Trigger aria-label="Settings" asChild let:builder>
              <Button variant="outline" size="icon" builders={[builder]}>
                <SettingsIcon />
              </Button>
            </DropdownMenu.Trigger>
            <DropdownMenu.Content>
              <DropdownMenu.Item>
                <form method="POST" action="?/reprocess" use:enhance>
                  <input type="hidden" name="id" value={item.id} />
                  <input type="hidden" name="stage" value="summarize" />
                  <button type="submit" class="flex items-center gap-2">
                    <RefreshIcon class="h-4 w-4" />
                    Reprocess
                  </button>
                </form>
              </DropdownMenu.Item>

              <DropdownMenu.Item>
                <form method="POST" action="?/delete" use:enhance>
                  <input type="hidden" name="id" value={item.id} />
                  <button type="submit" class="flex items-center gap-2">
                    <DeleteIcon class="w-4 h-4" /> Delete
                  </button>
                </form>
              </DropdownMenu.Item>
            </DropdownMenu.Content>
          </DropdownMenu.Root>
        </div>
      </li>
    {/each}
  </ul>

  <form
    method="POST"
    action="?/add_existing"
    use:enhance
    class="flex flex-col gap-2 rounded-lg border border-border p-4"
  >
    <Label class="flex gap-2 flex-1 max-w-[100ch] text-base" for="path">Add existing path</Label>
    <div class="flex gap-2">
      <Input type="text" name="path" class="flex-1" autocomplete="off" />
      <Button type="submit">Add</Button>
    </div>
  </form>
</main>
