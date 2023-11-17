import { redirect, type Actions, fail } from '@sveltejs/kit';
import { listItems, loadNewItem } from '$lib/server/data';

export async function load() {
  const items = listItems();

  return {
    items,
  };
}

export const actions = {
  default: async (event) => {
    const formData = await event.request.formData();
    const path = formData.get('path') as string;
    if (!path) {
      return fail(400, {
        error: 'No path provided',
      });
    }

    const item = await loadNewItem(path);
    if (item == null) {
      return fail(404, {
        error: 'File not found',
      });
    }

    throw redirect(307, '/docs/' + item.id);
  },
} satisfies Actions;
