#!/usr/bin/env zx

import { removeSimilarImages } from './calculate_similarity.mjs';
import {summarize} from './summarize.mjs';

const inputUrl = process.argv[3];
if(!inputUrl) {
  throw new Error('No URL provided');
}

const pythonDir = path.join(__dirname, 'python');
const processDir = path.join(__dirname, 'app', 'data', inputUrl.replaceAll(/[^a-zA-Z0-9]+/g, '_'));

await $`mkdir -p ${processDir}`;

function titleFromFilename(filename) {
  filename = path.basename(filename);
  const lastDot = filename.lastIndexOf('.');
  if(lastDot >= 0) {
    filename = filename.slice(0, lastDot);
  }

  return filename;
}

let videoFile;
let title;
if(inputUrl.startsWith('http')) {
  const downloadPath = path.join(processDir, 'video-dl-%(title)s.%(ext)s');
  await $`yt-dlp --output ${downloadPath} ${inputUrl}`;
  videoFile = (await glob(path.join(processDir, 'video-dl-*')))[0];
  if(!videoFile) {
    throw new Error('Failed to download video');
  }

  title = titleFromFilename(videoFile).slice('video-dl-'.length);
} else {
  videoFile = inputUrl;
  title = titleFromFilename(videoFile);
  if(!fs.existsSync(inputUrl)) {
    throw new Error('No such file or directory: ' + inputUrl);
  }
}

async function processAudio() {
  const audioPath = path.join(processDir, 'raw_audio.wav');
  const transcriptPath = path.join(processDir, 'transcript.json');
  await $`ffmpeg -y -i ${videoFile} -vn -acodec pcm_s16le -ar 16000 -ac 1 ${audioPath}`;

  const whisperStart = Date.now();
  await within(async () => {
    cd(pythonDir);
    await $`rye run whisper ${audioPath}`.pipe(fs.createWriteStream(transcriptPath));
  });
  const whisperTime = Date.now() - whisperStart;

  await $`rm ${audioPath}`;

  const transcript = JSON.parse(await fs.readFile(transcriptPath));
  // Sometimes the last item has a null timestamp, so find the last chunk that has a timestamp
  const lastWithTimestamp = transcript.findLast((t) => t.timestamp[1]);
  const finalTimestamp = Math.ceil(lastWithTimestamp.timestamp[1]);

  const summaryStart = Date.now();
  const summary = await summarize(title, transcript)
  const summaryTime = Date.now() - summaryStart;

  return { audioConfig: { summary, duration: finalTimestamp }, timing: { whisper: whisperTime, summary: summaryTime } };
}

async function extractImages() {
  const interval = 10;

  const extractStart = Date.now();
  const imagePath = path.join(processDir, 'image-%05d.webp');
  const fps = `fps=1/${interval}`;
  await $`ffmpeg -y -i ${videoFile} -vf ${fps} -c:v libwebp ${imagePath}`;

  const similarityStart = Date.now();
  const images = await glob(path.join(processDir, 'image-*.webp'));
  const removed = await removeSimilarImages(images);
  const similarityDone = Date.now();

  return {
    imageConfig: {
      maxIndex: images.length - 1,
      removed,
      interval,
    },
    timing: {
      extract: similarityStart - extractStart,
      similarity: similarityDone - similarityStart,
    }
  };
}

const [audioInfo, images] = await Promise.all([
  processAudio(),
  extractImages()
]);

const config = {
  title,
  originalVideoPath: inputUrl,
  processedPath: processDir,
  images: images.imageConfig,
  ...audioInfo.audioConfig,
};

const timing = {
  ...images.timing,
  ...audioInfo.timing,
};

const configPath = path.join(processDir, 'sbbp.json');
await fs.writeFile(configPath, JSON.stringify(config));

echo('Took', timing);
echo('Wrote to ', configPath);

