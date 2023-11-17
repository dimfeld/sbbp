#!/usr/bin/env zx

import 'zx/globals';

const inputUrl = process.argv[3];

const processDir = path.join(__dirname, 'viewer', 'data', inputUrl.replaceAll(/[^a-zA-Z0-9]+/g, '_'));
const downloadPath = path.join(processDir, 'video-dl')

const imageInterval = 10;

await $`mkdir -p ${processDir}`;

let videoFile;
if(inputUrl.startsWith('http')) {
  await $`yt-dlp --output ${downloadPath} ${inputUrl}`;
  videoFile = (await glob(`${downloadPath}.*`))[0];
  if(!videoFile) {
    throw new Error('Failed to download video');
  }
} else {
  videoFile = inputUrl;
  if(!fs.existsSync(inputUrl)) {
    throw new Error('No such file or directory: ' + inputUrl);
  }
}

async function processAudio() {
  const audioPath = path.join(processDir, 'raw_audio.wav');
  const transcriptPath = path.join(processDir, 'transcript.json');
  await $`ffmpeg -y -i ${videoFile} -vn -acodec pcm_s16le -ar 16000 -ac 1 ${audioPath}`;
  await spinner(() => $`rye run whisper ${audioPath}`.pipe(fs.createWriteStream(transcriptPath)));
  await $`rm ${audioPath}`;

  const transcript = JSON.parse(await fs.readFile(transcriptPath));
  const finalTimestamp = Math.ceil(transcript[transcript.length - 1].timestamp[1]);
  return finalTimestamp;
}

async function extractImages() {
  const imagePath = path.join(processDir, 'image-%05d.webp');
  const fps = `fps=1/${imageInterval}`;
  await $`ffmpeg -y -i ${videoFile} -vf ${fps} -c:v libwebp ${imagePath}`;

  const images = await glob(`${imagePath}.*`);
  return images.length;
}

const [duration, numImages] = await Promise.all([
  processAudio(),
  extractImages()
]);

const config = {
  title: path.basename(videoFile).split('.')[0],
  originalVideoPath: inputUrl,
  processedPath: processDir,
  numImages,
  imageInterval,
  duration,
};

const configPath = path.join(processDir, 'sbbp.json');
await fs.writeFile(configPath, JSON.stringify(config));

echo('Wrote to ', configPath);

