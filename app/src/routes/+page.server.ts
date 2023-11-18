import { redirect, type Actions, fail } from '@sveltejs/kit';
import { deleteItem, listItems, loadNewItem, reloadItem } from '$lib/server/data';

export async function load() {
  const items = listItems();

  return {
    items,
  };
}

export const actions = {
  add: async (event) => {
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
  refresh: async (event) => {
    const formData = await event.request.formData();
    const id = formData.get('id') as string;
    if (!id) {
      return fail(400, {
        error: 'No id provided',
      });
    }

    const item = await reloadItem(+id);
    if (!item) {
      return fail(404, {
        error: 'File not found',
      });
    }

    return {};
  },
  delete: async (event) => {
    const formData = await event.request.formData();
    const id = formData.get('id') as string;
    if (!id) {
      return fail(400, {
        error: 'No id provided',
      });
    }

    await deleteItem(+id);

    return {};
  },
} satisfies Actions;
