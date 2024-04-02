import { mark_read } from '$lib/models/video.js';

export const actions = {
  mark_read: async (event) => {
    const formData = await event.request.formData();
    const id = event.params.docId;
    const read = formData.get('new_read') === 'true';
    await mark_read({ fetch: event.fetch, id, payload: { read } });
  },
};
