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
  return fs.writeFile(configPath, JSON.stringify(config, null, 2));
}

export function listItems() {
  return config.items;
}

// Currently this only takes local paths to already-processed data.
export async function loadNewItem(file: string): Promise<Video | null> {
  const existing = config.items.find((item) => item.processedPath === file);
  if (existing) {
    return existing;
  }

  let contentDir = file;
  if (await fs.stat(contentDir).then((s) => s.isFile())) {
    contentDir = path.dirname(contentDir);
  }

  if (!(await fs.stat(contentDir).then((s) => s.isDirectory()))) {
    return null;
  }

  const itemConfigPath = path.join(contentDir, 'sbbp.json');
  const itemConfigData = await fs.readFile(itemConfigPath);
  const itemConfig: Omit<Video, 'id'> = JSON.parse(itemConfigData.toString());

  const id = config.items.reduce((acc, item) => Math.max(acc, item.id), 0) + 1;

  const newItem = {
    id,
    title: itemConfig.title,
    originalVideoPath: itemConfig.originalVideoPath,
    processedPath: contentDir,
    images: itemConfig.images,
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

export function createPath(item: Video, filename: string) {
  const fullPath = path.join(item.processedPath, filename);
  return path.resolve(dataDir, fullPath);
}

export async function getItemText(item: Video): Promise<TranscriptChunk[]> {
  const transcriptPath = createPath(item, 'transcript.json');
  const data = await fs.readFile(transcriptPath);
  return JSON.parse(data.toString());
}

export function loadImage(docId: number, id: number) {
  const item = getItem(docId);
  if (!item) {
    return null;
  }

  if (id > item.images.maxIndex) {
    return null;
  }

  const imagePath = createPath(item, 'image-' + (id + 1).toString().padStart(5, '0') + '.webp');
  let stream = createReadStream(imagePath);

  return Readable.toWeb(stream);
}
