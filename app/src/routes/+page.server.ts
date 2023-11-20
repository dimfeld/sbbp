import { redirect, type Actions, fail } from '@sveltejs/kit';
import {
  deleteItem,
  enqueueNewItem,
  listItems,
  loadNewItem,
  reloadItem,
  reprocessItem,
  updateReadState,
} from '$lib/server/data';

export async function load({ depends }) {
  depends('resource://items');
  const items = listItems();

  return {
    items,
  };
}

export const actions = {
  download: async (event) => {
    const formData = await event.request.formData();
    const path = formData.get('path') as string;
    await enqueueNewItem(path);
  },
  add_existing: async (event) => {
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
  },
  reprocess: async (event) => {
    const formData = await event.request.formData();
    const id = formData.get('id') as string;
    if (!id) {
      return fail(400, {
        error: 'No id provided',
      });
    }

    await reprocessItem(+id);
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
  mark_read: async (event) => {
    const formData = await event.request.formData();
    const id = formData.get('id') as string;
    if (!id) {
      return fail(400, {
        error: 'No id provided',
      });
    }

    const read = formData.get('new_read') === 'true';
    await updateReadState(+id, read);
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
