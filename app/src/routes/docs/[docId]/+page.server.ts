import * as data from '$lib/server/data';
import { updateReadState } from '$lib/server/data';
import { error } from '@sveltejs/kit';

export async function load({ params }) {
  const item = data.getItem(+params.docId);
  if (!item) {
    throw error(404);
  }

  const text = await data.getItemText(item);

  return {
    item,
    text,
  };
}

export const actions = {
  mark_read: async (event) => {
    const formData = await event.request.formData();
    const id = +event.params.docId;
    const read = formData.get('new_read') === 'true';
    await updateReadState(+id, read);
  },
};
