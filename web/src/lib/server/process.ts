import { $, execa } from 'execa';
import { pkgUpSync } from 'pkg-up';
import * as path from 'path';
import * as fs from 'fs';
import { globby } from 'globby';
import { removeSimilarImages } from './calculate_similarity.js';
import { summarize } from './summarize.js';
import type { ProcessResult, ProcessStatus, TranscriptChunk } from '$lib/types.js';

const pkgDir = path.dirname(pkgUpSync()!);
const pythonDir = process.env.PYTHON_DIR || path.join(pkgDir, '..', 'python');
const dataDir = process.env.DATA_DIR || path.join(pkgDir, 'data');

export async function processInput(inputUrl: string, statusCb: (status: ProcessStatus) => void) {
  console.log('processing', inputUrl);
  const processDir = path.join(dataDir, inputUrl.replaceAll(/[^a-zA-Z0-9]+/g, '_'));
  await fs.promises.mkdir(processDir, { recursive: true });

  statusCb('downloading');
  const { videoFile, title } = await downloadVideo(inputUrl, processDir);

  statusCb('processing');
  const [audioInfo, images] = await Promise.all([
    processAudio(processDir, videoFile, title),
    extractImages(processDir, videoFile),
  ]);
  statusCb('complete');

  // console.log('tasks done')

  const timing = {
    ...images.timing,
    ...audioInfo.timing,
  };

  const config: ProcessResult = {
    title,
    originalVideoPath: inputUrl,
    processedPath: processDir,
    process: {
      timing,
    },
    images: images.imageConfig,
    ...audioInfo.audioConfig,
  };

  const configPath = path.join(processDir, 'sbbp.json');
  await fs.promises.writeFile(configPath, JSON.stringify(config));

  console.log('Took', JSON.stringify(timing, null, 2));
  console.log('Wrote to ', configPath);

  return config;
}

function titleFromFilename(filename: string) {
  filename = path.basename(filename);
  const lastDot = filename.lastIndexOf('.');
  if (lastDot >= 0) {
    filename = filename.slice(0, lastDot);
  }

  return filename;
}

async function downloadVideo(inputUrl: string, processDir: string) {
  let videoFile;
  let title;
  if (inputUrl.startsWith('http')) {
    const downloadPath = path.join(processDir, 'video-dl-%(title)s.%(ext)s');
    await $`yt-dlp --output ${downloadPath} ${inputUrl}`;
    videoFile = (await globby(path.join(processDir, 'video-dl-*')))[0];
    if (!videoFile) {
      throw new Error('Failed to download video');
    }

    title = titleFromFilename(videoFile).slice('video-dl-'.length);
  } else {
    videoFile = inputUrl;
    title = titleFromFilename(videoFile);
    if (!fs.existsSync(inputUrl)) {
      throw new Error('No such file or directory: ' + inputUrl);
    }
  }

  return {
    videoFile,
    title,
  };
}

async function processAudio(processDir: string, videoFile: string, title: string) {
  const audioPath = path.join(processDir, 'raw_audio.wav');
  const transcriptPath = path.join(processDir, 'transcript.json');
  await $`ffmpeg -y -i ${videoFile} -vn -acodec pcm_s16le -ar 16000 -ac 1 ${audioPath}`;

  const whisperStart = Date.now();
  await execa('rye', ['run', 'whisper', audioPath], {
    cwd: pythonDir,
  }).pipeStdout?.(transcriptPath);
  const whisperTime = Date.now() - whisperStart;

  await fs.promises.unlink(audioPath);

  const transcriptRaw = await fs.promises.readFile(transcriptPath);
  const transcript: TranscriptChunk[] = JSON.parse(transcriptRaw.toString());
  // Sometimes the last item has a null timestamp, so find the last chunk that has a timestamp
  const lastWithTimestamp = transcript.findLast((t) => t.timestamp[1]);
  const finalTimestamp = Math.ceil(lastWithTimestamp?.timestamp[1] ?? 0);

  const summaryStart = Date.now();
  let summary = '';
  try {
    summary = await summarize(title, transcript);
  } catch (e) {
    console.error(e);
  }
  const summaryTime = Date.now() - summaryStart;

  // console.log('processAudio done');

  return {
    audioConfig: { summary, duration: finalTimestamp },
    timing: { whisper: whisperTime, summary: summaryTime },
  };
}

async function extractImages(processDir: string, videoFile: string) {
  const interval = 10;

  const extractStart = Date.now();
  const imagePath = path.join(processDir, 'image-%05d.webp');
  const fps = `fps=1/${interval}`;
  await $`ffmpeg -y -i ${videoFile} -vf ${fps} -c:v libwebp ${imagePath}`;

  const similarityStart = Date.now();
  const imageGlob = path.join(processDir, 'image-*.webp');
  const { removed, numImages } = await removeSimilarImages(pythonDir, imageGlob);
  const similarityDone = Date.now();

  // console.log('extractImages done');

  return {
    imageConfig: {
      maxIndex: numImages - 1,
      removed,
      interval,
    },
    timing: {
      extract: similarityStart - extractStart,
      similarity: similarityDone - similarityStart,
    },
  };
}
