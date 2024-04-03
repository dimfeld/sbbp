<script lang="ts">
  import { browser } from '$app/environment';
  import { enhance } from '$app/forms';
  import { Button, Icon, Menu, MenuItem, Switch, TextField, Toggle } from 'svelte-ux';
  import {
    mdiFileDocumentRemove as deleteIcon,
    mdiRefresh as refreshIcon,
    mdiCog as settingsIcon,
    mdiEmailOutline as unreadIcon,
    mdiEmailOpenOutline as readIcon,
  } from '@mdi/js';
  import { formatDuration } from '$lib/format.js';
  import { invalidate } from '$app/navigation';
  import ReprocessForm from './ReprocessForm.svelte';
  import { onDestroy, tick } from 'svelte';

  const { data, form } = $props();

  let unreadOnly = $state(true);

  let timer: number | null = null;
  $effect(() => {
    if (
      browser &&
      !timer &&
      data.items.some((item) => (item.processing_state ?? 'ready') !== 'ready')
    ) {
      timer = window.setTimeout(() => {
        timer = null;
        invalidate('resource://items');
      }, 5000);
    }
  });

  onDestroy(() => {
    if (timer) {
      clearTimeout(timer);
    }
  });

  let items = $derived(unreadOnly ? data.items.filter((item) => !item.read) : data.items);
</script>

<main class="relative p-4 flex flex-col gap-4">
  <form
    method="POST"
    action="?/download"
    use:enhance
    class="flex flex-col gap-2 rounded-lg border border-border p-4"
  >
    <label class="flex gap-2 flex-1 max-w-[100ch] text-base" for="path">Add new video</label>
    <div class="flex gap-2">
      <TextField name="url" class="flex-1" autocomplete="off" />
      <Button color="primary" type="submit">Add</Button>
    </div>
  </form>

  {#if form?.error}
    <p class="text-red-500">{form.error}</p>
  {/if}

  <div class="flex justify-end gap-2">
    <div class="flex items-center gap-2">
      <Switch id="unread-only" bind:checked={unreadOnly} />
      <label for="unread-only">Unread only</label>
    </div>
  </div>

  <ul class="flex flex-col gap-4">
    {#each items as item (item.id)}
      {@const read = item.read}
      {@const ready = item.processing_state === 'ready'}
      <li class="flex justify-between">
        <div class="flex flex-col">
          {#if ready}
            <a href="/docs/{item.id}" class="underline">{item.title}</a>
            <span>
              {formatDuration(item.duration)}
            </span>
          {:else}
            <p>{item.url}</p>
            {#if item.processing_state === 'error' && item.process?.error}
              <p class="text-red-500">{item.process.error}</p>
            {:else}
              <p>{item.title || item.processing_state || ''}</p>
            {/if}
          {/if}
        </div>

        <div class="flex gap-2 items-center">
          <form method="POST" action="?/mark_read" use:enhance>
            <input type="hidden" name="id" value={item.id} />
            <input type="hidden" name="new_read" value={!read} />
            {#if ready}
              <Button
                variant="outline"
                icon={read ? readIcon : unreadIcon}
                type="submit"
                formaction="?/mark_read"
                aria-label="Mark {read ? 'Unread' : 'Read'}"
              />
            {/if}
          </form>

          <Toggle let:on={open} let:toggleOff let:toggle>
            <Button variant="outline" icon={settingsIcon} on:click={toggle} />
            <Menu {open} explicitClose on:close={toggleOff} let:close>
              <p class="px-2 text-sm font-medium pl-6 py-2">Reprocess</p>
              <ReprocessForm id={item.id} stage="download" label="Download" {close} />
              <ReprocessForm id={item.id} stage="extract" label="Extract" {close} />
              <ReprocessForm id={item.id} stage="analyze" label="Analyze" {close} />
              <ReprocessForm id={item.id} stage="transcribe" label="Transcribe" {close} />
              <ReprocessForm id={item.id} stage="summarize" label="Summarize" {close} />

              <MenuItem>
                <form method="POST" action="?/delete" use:enhance>
                  <input type="hidden" name="id" value={item.id} />
                  <button type="submit" on:click={close} class="flex items-center gap-2">
                    <Icon data={deleteIcon} /> Delete
                  </button>
                </form>
              </MenuItem>
            </Menu>
          </Toggle>
        </div>
      </li>
    {/each}
  </ul>
</main>
