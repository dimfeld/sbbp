import { finishProcessing, flagProcessingError, updateProcessStatus } from './data';
import { processInput } from './process';

interface QueueItem {
  id: number;
  url: string;
}
const queue: QueueItem[] = [];
let isProcessing = false;

export function queueItem(id: number, url: string) {
  const startImmediately = queue.length === 0 && !isProcessing;

  if (startImmediately) {
    processItem(id, url);
  } else {
    queue.push({ id, url });
  }

  return startImmediately;
}

async function processItem(id: number, url: string) {
  isProcessing = true;

  try {
    const finalConfig = await processInput(url, (status) => updateProcessStatus(id, status));
    await finishProcessing(id, finalConfig);
  } catch (e) {
    console.error(e);
    updateProcessStatus(id, 'error');
    await flagProcessingError(id, (e as Error).stack || (e as Error).message);
  } finally {
    const nextItem = queue.shift();
    if (nextItem) {
      setImmediate(() => {
        processItem(nextItem.id, nextItem.url);
      });
    } else {
      isProcessing = false;
    }
  }
}
