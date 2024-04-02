import type { Video } from '$lib/models/video';
import { error } from '@sveltejs/kit';
import { client } from 'filigree-web';

export async function load({ params, fetch }) {
  const item = await client({
    url: `/api/videos/${params.docId}`,
    fetch,
  }).json<Video>();

  if (!item) {
    error(404);
  }

  return {
    item,
  };
}
