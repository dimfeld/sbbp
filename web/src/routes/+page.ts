import { type VideoListResult } from '$lib/models/video';
import { client } from 'filigree-svelte';

export async function load({ depends, fetch }) {
  depends('resource://items');
  const items = await client({
    url: '/api/videos',
    fetch,
  }).json<VideoListResult[]>();

  return {
    items,
  };
}
