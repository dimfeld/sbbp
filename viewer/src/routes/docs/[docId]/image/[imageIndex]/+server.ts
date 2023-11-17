import { loadImage } from '$lib/server/data';
import { error } from '@sveltejs/kit';

export async function GET({ params }) {
  const { docId, imageIndex } = params;
  const image = loadImage(docId, imageIndex);
  if (!image) {
    throw error(404, { message: 'not found' });
  }

  const res = new Response(image);
  res.headers.set('Content-Type', 'image/webp');
  return res;
}
