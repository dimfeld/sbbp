<script lang="ts">
  import { enhance } from '$app/forms';

  export let data;
</script>

<form method="POST" use:enhance class="flex gap-2">
  <label class="flex gap-2 flex-1 max-w-[100ch]">
    <p>Add new path</p>
    <input type="text" name="path" class="flex-1" />
  </label>
  <button type="submit">Add</button>
</form>

<ul>
  {#each data.items as item (item.id)}
    <li><a href="/docs/{item.id}">{item.title}</a> - {item.duration}</li>
  {/each}
</ul>
