import type { TranscriptChunk } from '$lib/types';
import { execa } from 'execa';

export async function summarize(title: string, transcript: TranscriptChunk[]) {
  const allText = transcript.map((t) => t.text).join('\n');

  const result = await execa('promptbox', ['run', 'summarize', '--title', title], {
    input: allText,
  });

  return result.stdout;
}
