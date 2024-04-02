import type { ViewerChunk } from './types';
import type { Video } from '$lib/models/video';

export function align(video: Video) {
  if (!video.images || !video.transcript) {
    return [];
  }

  const { interval: imageInterval, max_index: maxImageIndex } = video.images;

  const transcript = video.transcript;

  const numImages = maxImageIndex + 1;

  const imageTimestamp = (index: number) => {
    return index * imageInterval;
  };

  const output = transcript.results.channels[0].alternatives[0].paragraphs?.paragraphs.map((p) => {
    let text = p.sentences.map((s) => s.text).join(' ');
    let chunk: ViewerChunk = {
      text,
      timestamp: [p.start, p.end],
      images: [
        Math.max(1, Math.min(Math.ceil(p.start / imageInterval), numImages - 1)),
        Math.max(1, Math.min(Math.floor(p.end / imageInterval), numImages - 1)),
      ],
    };

    return chunk;
  });

  return output ?? [];
  /*
  if (!output) {
    return [];
  }

  let firstTextTimestamp = inputText[0].timestamp[0];
  let output: ViewerChunk[] = [
    {
      timestamp: [firstTextTimestamp, firstTextTimestamp],
      text: '',
      images: [],
    },
  ];

  let textIndex = 0;
  while (textIndex < inputText.length) {
    const { start: textStart } = inputText[textIndex];

    const currentChunk = output[output.length - 1];
    const chunkBoundary = currentChunk.timestamp[0] + imageInterval;

    // const info = {
    //   textStart,
    //   textEnd,
    //   textIndex,
    // };

    if (textStart > chunkBoundary) {
      // Create a new chunk
      output.push({
        timestamp: [textStart, textStart],
        text: '',
        images: [],
      });
    } else {
      // console.log({ ...info, action: 'addTextToCurrentChunk' });
      const addText = inputText[textIndex].text;

      // Add a space if the transcript doesn't have it (Whisper usually does actually, but just in case)
      if (currentChunk.text && addText[0] !== ' ' && !currentChunk.text.endsWith(' ')) {
        currentChunk.text += ' ';
      }
      currentChunk.text += inputText[textIndex].text;

      currentChunk.timestamp[1] = Math.max(inputText[textIndex].end, currentChunk.timestamp[1]);

      textIndex++;
    }
  }

  const lastChunk = output[output.length - 1];
  lastChunk.timestamp[1] = Math.max(imageTimestamp(numImages - 1), lastChunk.timestamp[1]);

  for (let chunk of output) {
    chunk.images = [
      Math.min(Math.ceil(chunk.timestamp[0] / imageInterval), numImages - 1),
      Math.min(Math.floor(chunk.timestamp[1] / imageInterval), numImages - 1),
    ];
  }

  return output;
  */
}
