<script lang="ts">
  import { enhance } from '$app/forms';
  import { Button } from '$lib/components/ui/button';

  export let read: boolean;
</script>

<div class="flex gap-4">
  <form method="POST" action="?/mark_read" use:enhance>
    <input type="hidden" name="new_read" value={read ? 'false' : 'true'} />
    <Button variant="outline" type="submit" formaction="?/mark_read">
      Mark {read ? 'Unread' : 'Read'}
    </Button>
  </form>
  <a href="/"><Button variant="outline">Back to Index</Button></a>
</div>
