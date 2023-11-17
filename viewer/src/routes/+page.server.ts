import { redirect, type Actions, fail } from '@sveltejs/kit';
import { loadNewItem } from '$lib/server/data';

export async function load({ url }) {
  const path = url.searchParams.get('path');
  if (!path) {
    return {
      file: null,
    };
  }

  return {
    file: null,
  };
}

export const actions = {
  default: async (event) => {
    const formData = await event.request.formData();
    const path = formData.get('path');
    if (!path) {
      return fail(400, {
        error: 'No path provided',
      });
    }

    const id = await loadNewItem(path);
    if (id == null) {
      return fail(404, {
        error: 'File not found',
      });
    }

    throw redirect(307, '/docs/' + id);
  },
} satisfies Actions;
