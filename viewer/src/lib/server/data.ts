import { env } from '$env/dynamic/private';
import type { TranscriptChunk, Video } from '$lib/types';
import { readFileSync, createReadStream } from 'fs';
import * as fs from 'fs/promises';
import * as path from 'path';
import { Readable } from 'stream';

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

init();

function saveConfig() {
  return fs.writeFile(configPath, JSON.stringify(config));
}

export function listItems() {
  return config.items;
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

  const itemConfigPath = path.join(contentDir, 'sbbp.json');
  const itemConfigData = await fs.readFile(itemConfigPath);
  const itemConfig = JSON.parse(itemConfigData.toString());

  const id = config.items.reduce((acc, item) => Math.max(acc, item.id), 0) + 1;

  const newItem = {
    id,
    title: itemConfig.title,
    originalVideoPath: itemConfig.originalVideoPath,
    processedPath: contentDir,
    imageInterval: itemConfig.imageInterval,
    numImages: itemConfig.numImages,
    duration: itemConfig.duration,
  };

  config.items.push(newItem);
  await saveConfig();
  return newItem;
}

export async function deleteItem(id: number) {
  config.items = config.items.filter((item) => item.id !== id);
  await saveConfig();
}

export function getItem(id: number) {
  return config.items.find((item) => item.id === id);
}

export async function getItemText(item: Video): Promise<TranscriptChunk[]> {
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
