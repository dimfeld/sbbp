import * as data from '$lib/server/data';
import { error } from '@sveltejs/kit';

export async function load({ params }) {
  const item = data.getItem(params.docId);
  if (!item) {
    throw error(404);
  }

  const text = await data.getItemText(item);

  return {
    item,
    text,
  };
}
