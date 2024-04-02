import { create_via_url, mark_read, rerun_stage, type VideoListResult } from '$lib/models/video';
import { type Actions, fail } from '@sveltejs/kit';
import { client } from 'filigree-web';

export const actions = {
  download: async (event) => {
    const formData = await event.request.formData();
    const url = formData.get('url') as string;
    await create_via_url({
      fetch: event.fetch,
      payload: {
        url,
      },
    });
  },
  reprocess: async (event) => {
    const formData = await event.request.formData();
    const id = formData.get('id') as string;
    const stage = formData.get('stage') as string;
    if (!id) {
      return fail(400, {
        error: 'No id provided',
      });
    }

    await rerun_stage({
      id,
      stage,
      fetch: event.fetch,
    });
  },
  mark_read: async (event) => {
    const formData = await event.request.formData();
    const id = formData.get('id') as string;
    if (!id) {
      return fail(400, {
        error: 'No id provided',
      });
    }

    await mark_read({
      id,
      payload: {
        read: formData.get('new_read') === 'true',
      },
      fetch: event.fetch,
    });
  },
  delete: async (event) => {
    const formData = await event.request.formData();
    const id = formData.get('id') as string;
    if (!id) {
      return fail(400, {
        error: 'No id provided',
      });
    }

    await client({
      url: `/api/video/${id}`,
      method: 'DELETE',
    });

    return {};
  },
} satisfies Actions;
