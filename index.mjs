#!/usr/bin/env zx

import 'zx/globals';

const inputUrl = process.argv[2];

const savePath = 'video-dl';

// await $`yt-dlp --output ${savePath} ${inputUrl}`;

let [ videoFile ]= await glob(`${savePath}.*`);

await $`mkdir -p processed`;

async function processAudio() {
  const audioPath = 'processed/raw_audio.wav';
  await $`ffmpeg -y -i ${videoFile} -vn -acodec pcm_s16le -ar 16000 -ac 1 ${audioPath}`;
  await $`rye run whisper ${audioPath}`.pipe(fs.createWriteStream('processed/transcript.json'));
}

async function extractImages() {
  await $`ffmpeg -y -i ${videoFile} -vf 'fps=1/10' -c:v libwebp processed/image-%05d.webp`;
}

await Promise.all([
  processAudio(),
  extractImages()
]);
