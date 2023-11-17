import { env } from '$env/dynamic/private';
import { readFileSync, createReadStream } from 'fs';
import * as fs from 'fs/promises';
import * as path from 'path';
import { Readable } from 'stream';

export interface Video {
  id: number;
  title: string;
  originalVideoPath: string;
  processedPath: string;
  numImages: number;
  duration: number;
}

interface Config {
  items: Video[];
}

const dataDir = env.SBBP_DATA_DIR || './data';
const configPath = path.join(dataDir, 'config.json');

let config: Config;

function init() {
  try {
    config = JSON.parse(readFileSync(configPath).toString());
  } catch (e) {
    config = {
      items: [],
    };

    saveConfig();
  }
}

function saveConfig() {
  return fs.writeFile(configPath, JSON.stringify(config));
}

// Currently this only takes local paths to already-processed data.
export async function loadNewItem(file: string) {
  const existing = config.items.find((item) => item.processedPath === file);
  if (existing) {
    return existing.id;
  }

  let contentDir = file;
  if (contentDir.endsWith('.json')) {
    contentDir = path.dirname(contentDir);
  }
}

export function getItem(id: number) {
  return config.items.find((item) => item.id === id);
}

export async function getItemText(item: Video) {
  const data = await fs.readFile(path.join(dataDir, item.processedPath, 'transcript.json'));
  return JSON.parse(data.toString());
}

export function loadImage(docId: number, id: number) {
  const item = getItem(docId);
  if (!item) {
    return null;
  }

  if (id >= item.numImages) {
    return null;
  }

  let stream = createReadStream(
    path.join(
      dataDir,
      item.processedPath,
      'image-' + (id + 1).toString().padStart(5, '0') + '.webp'
    )
  );

  return Readable.toWeb(stream);
}
