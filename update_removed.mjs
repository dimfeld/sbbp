#!/usr/bin/env zx

import { removeSimilarImages } from './calculate_similarity.mjs';

const dir = process.argv[3];

const config = JSON.parse(fs.readFileSync(path.join(dir, 'sbbp.json')).toString());

if(!fs.existsSync(dir)) {
  throw new Error('Directory does not exist');
}

const images = await glob(path.join(dir, 'image-*.webp'));
const removed = await removeSimilarImages(images);
config.images.removed = removed;

await fs.writeFile(path.join(dir, 'sbbp.json'), JSON.stringify(config, null, 2));
