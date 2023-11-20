import { env } from '$env/dynamic/private';
import type { ProcessResult, ProcessStatus, TranscriptChunk, Video } from '$lib/types';
import { readFileSync, createReadStream } from 'fs';
import * as fs from 'fs/promises';
import * as path from 'path';
import { Readable } from 'stream';
import { queueItem } from './process_queue';
import { pkgUpSync } from 'pkg-up';

interface Config {
  version: number;
  items: Video[];
}

const dataDir = env.SBBP_DATA_DIR || path.join(path.dirname(pkgUpSync()!), 'data');
const configPath = path.join(dataDir, 'config.json');

let config: Config;

const CONFIG_VERSION = 2;

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
    console.log('Error loading config', e);
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
  if (!path.isAbsolute(file)) {
    file = path.resolve(dataDir, file);
  } else {
    file = path.relative(dataDir, file);
  }

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
  console.log('delete', id);
  config.items = config.items.filter((item) => item.id !== id);
  await saveConfig();
}

export async function reloadItem(id: number) {
  console.log('reload', id);
  config.items = config.items.filter((item) => item.id !== id);
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

export async function updateProcessStatus(docId: number, status: ProcessStatus) {
  return await updateItem(docId, (item) => {
    item.viewerData.processStatus = status;
  });
}

export async function flagProcessingError(docId: number, error: string) {
  return await updateItem(docId, (item) => {
    item.viewerData.processStatus = 'error';
    item.process = {
      timing: {},
      error,
    };
  });
}

export async function finishProcessing(docId: number, newConfig: ProcessResult) {
  return await updateItem(docId, (item) => {
    return {
      ...item,
      ...newConfig,
      viewerData: {
        ...item.viewerData,
        processStatus: 'complete',
      },
    };
  });
}

export async function loadItem(file: string) {
  let contentDir = path.resolve(dataDir, file);
  if (await fs.stat(contentDir).then((s) => s.isFile())) {
    contentDir = path.dirname(contentDir);
  }

  if (!(await fs.stat(contentDir).then((s) => s.isDirectory()))) {
    return null;
  }

  const itemConfigPath = path.join(contentDir, 'sbbp.json');
  const itemConfigData = await fs.readFile(itemConfigPath);
  const itemConfig: Omit<Video, 'id' | 'viewerData'> = JSON.parse(itemConfigData.toString());
  itemConfig.processedPath = path.basename(contentDir);

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

export async function enqueueNewItem(inputUrl: string) {
  console.log('enqueue', inputUrl);
  const newId = config.items.reduce((acc, item) => Math.max(acc, item.id), 0) + 1;
  const newItem: Video = {
    duration: 0,
    id: newId,
    images: {
      maxIndex: 0,
      interval: 0,
      removed: [],
    },
    originalVideoPath: inputUrl,
    processedPath: '',
    summary: '',
    title: '',
    viewerData: {
      progress: 0,
      read: false,
      processStatus: 'queued',
    },
  };

  config.items.push(newItem);

  queueItem(newId, inputUrl);

  await saveConfig();
}

export async function reprocessItem(id: number) {
  console.log('reprocess', id);
  const item = getItem(id);
  if (!item) {
    return null;
  }

  queueItem(id, item.originalVideoPath);
}
