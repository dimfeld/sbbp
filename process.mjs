#!/usr/bin/env zx

import 'zx/globals';

const inputUrl = process.argv[3];

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

  title = titleFromFilename(videoFile);
} else {
  videoFile = inputUrl;
  title = titleFromFilename(title).slice('video-dl-'.length);
  if(!fs.existsSync(inputUrl)) {
    throw new Error('No such file or directory: ' + inputUrl);
  }
}

async function processAudio() {
  const audioPath = path.join(processDir, 'raw_audio.wav');
  const transcriptPath = path.join(processDir, 'transcript.json');
  await $`ffmpeg -y -i ${videoFile} -vn -acodec pcm_s16le -ar 16000 -ac 1 ${audioPath}`;

  await within(async () => {
    cd(pythonDir);
    await $`rye run whisper ${audioPath}`.pipe(fs.createWriteStream(transcriptPath));
  });

  await $`rm ${audioPath}`;

  const transcript = JSON.parse(await fs.readFile(transcriptPath));
  const finalTimestamp = Math.ceil(transcript[transcript.length - 1].timestamp[1]);
  return finalTimestamp;
}

async function calculateSimilarity(image1, image2) {
  return within(async () => {
    cd(pythonDir);
    const sim = $`rye run compare-images ${image1} ${image2}`;
    return parseFloat(sim);
  });
}

async function removeSimilarImages(images) {
  let keep = [0];
  for(let i = 1; i < iamges.length; i++) {
    const lastImageIndex = keep[keep.length - 1];
    const lastImage = images[lastImageIndex];
    const currentImage = images[i];
    const similarity = await calculateSimilarity(lastImage, currentImage);
    if(similarity < 0.95) {
      // Don't actually delete the image for now, to make it easier to tweak the removal algorithm without redownloading.
      keep.push(i);
    }
  }

  const kept = new Set(keep);
  const removed = Array.from({ length: images.length }, (_, i) => i).filter((i) => !kept.has(i));

  return removed;
}

async function extractImages() {
  const interval = 10;

  const imagePath = path.join(processDir, 'image-%05d.webp');
  const fps = `fps=1/${interval}`;
  await $`ffmpeg -y -i ${videoFile} -vf ${fps} -c:v libwebp ${imagePath}`;

  const images = await glob(path.join(processDir, 'image-*.webp'));
  const removed = await removeSimilarImages(images);

  return {
    maxIndex: images.length - 1,
    removed,
    interval,
  };
}

const [duration, images] = await Promise.all([
  processAudio(),
  extractImages()
]);

const config = {
  title,
  originalVideoPath: inputUrl,
  processedPath: processDir,
  images,
  duration,
};

const configPath = path.join(processDir, 'sbbp.json');
await fs.writeFile(configPath, JSON.stringify(config));

echo('Wrote to ', configPath);

