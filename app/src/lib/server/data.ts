import { env } from '$env/dynamic/private';
import type { TranscriptChunk, Video } from '$lib/types';
import { readFileSync, createReadStream } from 'fs';
import * as fs from 'fs/promises';
import * as path from 'path';
import { Readable } from 'stream';

interface Config {
  version: number;
  items: Video[];
}

const dataDir = env.SBBP_DATA_DIR || './data';
const configPath = path.join(dataDir, 'config.json');

let config: Config;

const CONFIG_VERSION = 1;

function migrateConfig(config: Config) {
  const version = config.version ?? 0;

  if (version < 1) {
    for (let item of config.items) {
      item.viewerData = {
        read: false,
        progress: 0,
      };
    }
  }

  // Future migrations here

  config.version = CONFIG_VERSION;

  return config;
}

function init() {
  try {
    config = JSON.parse(readFileSync(configPath).toString());

    if (config.version ?? 0 < CONFIG_VERSION) {
      config = migrateConfig(config);
      saveConfig();
    }

    for (let item of config.items) {
      if (!item.viewerData) {
        item.viewerData = {
          read: false,
          progress: 0,
        };
      }
    }
  } catch (e) {
    config = {
      version: CONFIG_VERSION,
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

async function updateItem(
  docId: number,
  updateFn: (item: Video) => Video | void
): Promise<Video | null> {
  let item = config.items.findIndex((item) => item.id === docId);
  if (item < 0) {
    return null;
  }

  const result = updateFn(config.items[item]);
  if (result) {
    config.items[item] = result;
  }
  await saveConfig();
  return config.items[item];
}

// Currently this only takes local paths to already-processed data.
export async function loadNewItem(file: string): Promise<Video | null> {
  const existing = config.items.find((item) => item.processedPath === file);
  if (existing) {
    return existing;
  }

  const itemConfig = await loadItem(file);
  if (!itemConfig) {
    return null;
  }

  const id = config.items.reduce((acc, item) => Math.max(acc, item.id), 0) + 1;

  const newItem: Video = {
    id,
    viewerData: {
      read: false,
      progress: 0,
    },
    ...itemConfig,
  };

  config.items.push(newItem);
  await saveConfig();
  return newItem;
}

export async function deleteItem(id: number) {
  config.items = config.items.filter((item) => item.id !== id);
  await saveConfig();
}

export async function reloadItem(id: number) {
  const itemIndex = config.items.findIndex((item) => item.id === id);
  if (itemIndex < 0) {
    return null;
  }

  const existingItem = config.items[itemIndex];

  const newConfig = await loadItem(existingItem.processedPath);
  if (!newConfig) {
    return null;
  }

  config.items[itemIndex] = {
    id: existingItem.id,
    viewerData: existingItem.viewerData,
    ...newConfig,
  };

  await saveConfig();
  return config.items[itemIndex];
}

export async function updateReadState(docId: number, read: boolean): Promise<Video | null> {
  return await updateItem(docId, (item) => {
    item.viewerData.read = read;
  });
}

export async function updateReadProgress(docId: number, progress: number): Promise<Video | null> {
  return await updateItem(docId, (item) => {
    item.viewerData.progress = progress;
  });
}

export async function loadItem(file: string) {
  let contentDir = file;
  if (await fs.stat(contentDir).then((s) => s.isFile())) {
    contentDir = path.dirname(contentDir);
  }

  if (!(await fs.stat(contentDir).then((s) => s.isDirectory()))) {
    return null;
  }

  const itemConfigPath = path.join(contentDir, 'sbbp.json');
  const itemConfigData = await fs.readFile(itemConfigPath);
  const itemConfig: Omit<Video, 'id' | 'viewerData'> = JSON.parse(itemConfigData.toString());
  itemConfig.processedPath = contentDir;

  return itemConfig;
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
