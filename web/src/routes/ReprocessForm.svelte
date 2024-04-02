<script lang="ts">
  import { enhance } from '$app/forms';
  import { Icon, MenuItem } from 'svelte-ux';
  import { mdiRefresh as refreshIcon } from '@mdi/js';

  interface Props {
    id: string;
    label: string;
    stage: string;
    close: () => void;
  }

  const { id, label, stage, close }: Props = $props();
</script>

<MenuItem>
  <form method="POST" action="?/reprocess" use:enhance>
    <input type="hidden" name="id" value={id} />
    <input type="hidden" name="stage" value={stage} />
    <button type="submit" on:click={close} class="flex items-center gap-2">
      <Icon data={refreshIcon} />
      {label}
    </button>
  </form>
</MenuItem>
