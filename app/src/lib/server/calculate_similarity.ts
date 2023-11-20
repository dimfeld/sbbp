import * as path from 'path';
import { execa } from 'execa';

export async function removeSimilarImages(pythonDir: string, imageGlob: string) {
  imageGlob = path.resolve(imageGlob);
  const result = await execa('rye', ['run', 'compare-images', imageGlob], {
    cwd: pythonDir,
  });

  return JSON.parse(result.stdout);
}
