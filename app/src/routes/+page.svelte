<script lang="ts">
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

  export let data;
  export let form;

  let unreadOnly = true;

  $: items = unreadOnly ? data.items.filter((item) => !item.viewerData.read) : data.items;
</script>

<main class="relative p-4 flex flex-col gap-4">
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

  <div class="flex justify-end gap-2">
    <div class="flex items-center gap-2">
      <Switch id="unread-only" bind:checked={unreadOnly} />
      <Label for="unread-only">Unread only</Label>
    </div>
  </div>

  <ul class="flex flex-col gap-4">
    {#each items as item (item.id)}
      {@const read = item.viewerData.read}
      <li class="flex justify-between">
        <p>
          <a href="/docs/{item.id}" class="underline">{item.title}</a> - {item.duration}s
        </p>
        <p></p>
        <form method="POST" action="?/refresh" class="flex gap-2" use:enhance>
          <input type="hidden" name="id" value={item.id} />
          <input type="hidden" name="new_read" value={!read} />
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
          <DropdownMenu.Root>
            <DropdownMenu.Trigger aria-label="Settings" asChild let:builder>
              <Button variant="outline" size="icon" builders={[builder]}>
                <SettingsIcon />
              </Button>
            </DropdownMenu.Trigger>
            <DropdownMenu.Content>
              <DropdownMenu.Item>
                <button type="submit" formaction="?/refresh" class="flex items-center gap-2">
                  <RefreshIcon />
                  Reload from Disk
                </button>
              </DropdownMenu.Item>

              <DropdownMenu.Item>
                <button type="submit" formaction="?/delete" class="flex items-center gap-2">
                  <DeleteIcon /> Delete
                </button>
              </DropdownMenu.Item>
            </DropdownMenu.Content>
          </DropdownMenu.Root>
        </form>
      </li>
    {/each}
  </ul>
</main>
