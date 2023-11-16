import type { Config, ConfigTextChunk, ViewerChunk } from './types';

export function align(config: Config | null) {
  if (!config) {
    return null;
  }

  const { text: inputText, imageInterval, numImages, title } = config;

  const imageName = (index: number) => {
    return 'image-' + (index + 1).toString().padStart(5, '0') + '.webp';
  };
  const imageTimestamp = (index: number) => {
    return index * imageInterval;
  };

  let imageIndex = 0;

  // Sometimes the video doesn't actually start right away, such as when the video is from a livestream.
  // In this case we skip ahead to where the talking starts.
  let firstTextTimestamp = config.text[0].timestamp[0];
  if (firstTextTimestamp > config.imageInterval) {
    imageIndex = Math.floor(firstTextTimestamp / config.imageInterval);
  }

  let output: ViewerChunk[] = [
    {
      timestamp: [firstTextTimestamp, firstTextTimestamp],
      text: '',
      images: [],
    },
  ];

  function addTextToCurrentChunk() {
    const chunk = output[output.length - 1];
  }

  let textIndex = 0;
  while (textIndex < config.text.length) {
    const {
      timestamp: [textStart, textEnd],
    } = config.text[textIndex];

    const currentChunk = output[output.length - 1];
    const chunkBoundary = currentChunk.timestamp[0] + imageInterval;

    // const info = {
    //   textStart,
    //   textEnd,
    //   textIndex,
    // };

    if (textStart > chunkBoundary) {
      // Set the images for this chunk, and create a new chunk
      // console.log({ ...info, action: 'newChunk' });
      currentChunk.images = [
        Math.ceil(currentChunk.timestamp[0] / imageInterval),
        Math.floor(currentChunk.timestamp[1] / imageInterval),
      ];

      output.push({
        timestamp: [textStart, textStart],
        text: '',
        images: [],
      });
    } else {
      // console.log({ ...info, action: 'addTextToCurrentChunk' });
      const addText = inputText[textIndex].text;
      if (currentChunk.text && addText[0] !== ' ' && !currentChunk.text.endsWith(' ')) {
        currentChunk.text += ' ';
      }
      currentChunk.text += inputText[textIndex].text;
      currentChunk.timestamp[1] = Math.max(
        inputText[textIndex].timestamp[1],
        currentChunk.timestamp[1]
      );
      textIndex++;
    }
  }

  const lastChunk = output[output.length - 1];
  lastChunk.timestamp[1] = Math.max(imageTimestamp(numImages - 1), lastChunk.timestamp[1]);
  lastChunk.images = [
    Math.min(Math.ceil(lastChunk.timestamp[0] / imageInterval), numImages - 1),
    Math.min(Math.floor(lastChunk.timestamp[1] / imageInterval), numImages - 1),
  ];

  return {
    title,
    chunks: output,
  };
}
