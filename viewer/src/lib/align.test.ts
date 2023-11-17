import { expect, test } from 'vitest';
import { align } from './align';
import type { TranscriptChunk, Video, ViewerConfig } from './types';

test('single chunk', () => {
  const config: Video = {
    title: 'test',
    imageInterval: 10,
    numImages: 20,
    duration: 1000,
    id: 1,
    originalVideoPath: '',
    processedPath: '',
  };

  const textChunks: TranscriptChunk[] = [{ timestamp: [0, 1000], text: 'test' }];

  const expected = [
    {
      timestamp: [0, 1000],
      text: 'test',
      images: [0, 19],
    },
  ];

  expect(align(config, textChunks)).toEqual(expected);
});

test('multiple chunks smaller than image size', () => {
  const chunks: TranscriptChunk[] = [
    { timestamp: [0, 1], text: 'a' },
    { timestamp: [1, 2], text: 'b' },
    { timestamp: [2, 11], text: 'c' },
    { timestamp: [11, 12], text: 'd' },
    { timestamp: [12, 12], text: 'e' },
    { timestamp: [13, 14], text: 'f' },
  ];

  const config: Video = {
    title: 'test',
    imageInterval: 10,
    numImages: 3,
    duration: 1000,
    id: 1,
    originalVideoPath: '',
    processedPath: '',
  };

  const expected = [
    {
      timestamp: [0, 11],
      text: 'a b c',
      images: [0, 1],
    },
    {
      timestamp: [11, 20],
      text: 'd e f',
      images: [2, 2],
    },
  ];

  expect(align(config, chunks)).toEqual(expected);
});

test('text chunks somewhat larger than image interval', () => {
  const chunks: TranscriptChunk[] = [
    { timestamp: [0, 1], text: 'a' },
    { timestamp: [1, 15], text: 'b' },
    { timestamp: [15, 31], text: 'c' },
    { timestamp: [31, 38], text: 'd' },
    { timestamp: [38, 45], text: 'e' },
    { timestamp: [45, 50], text: 'f' },
  ];

  const config: Video = {
    title: 'test',
    imageInterval: 10,
    numImages: 7,
    duration: 1000,
    id: 1,
    originalVideoPath: '',
    processedPath: '',
  };

  const expected = [
    {
      timestamp: [0, 15],
      text: 'a b',
      images: [0, 1],
    },
    {
      timestamp: [15, 31],
      text: 'c',
      images: [2, 3],
    },

    {
      timestamp: [31, 45],
      text: 'd e',
      images: [4, 4],
    },
    {
      timestamp: [45, 60],
      text: 'f',
      images: [5, 6],
    },
  ];

  expect(align(config, chunks)).toEqual(expected);
});

test('text chunks span multiple image intervals', () => {
  const chunks: TranscriptChunk[] = [
    { timestamp: [0, 33], text: 'a' },
    { timestamp: [33, 35], text: 'b' },
    { timestamp: [35, 50], text: 'c' },
  ];

  const config: Video = {
    title: 'test',
    imageInterval: 10,
    numImages: 6,
    duration: 1000,
    id: 1,
    originalVideoPath: '',
    processedPath: '',
  };

  const expected = [
    {
      timestamp: [0, 33],
      text: 'a',
      images: [0, 3],
    },
    {
      timestamp: [33, 50],
      text: 'b c',
      images: [4, 5],
    },
  ];
  expect(align(config, chunks)).toEqual(expected);
});

test('no text at start of video', () => {
  const chunks: TranscriptChunk[] = [
    { timestamp: [15, 33], text: 'a' },
    { timestamp: [33, 35], text: 'b' },
    { timestamp: [35, 50], text: 'c' },
  ];

  const config: Video = {
    title: 'test',
    imageInterval: 10,
    numImages: 6,
    duration: 1000,
    id: 1,
    originalVideoPath: '',
    processedPath: '',
  };

  const expected = [
    {
      timestamp: [15, 33],
      text: 'a',
      images: [2, 3],
    },
    {
      timestamp: [33, 50],
      text: 'b c',
      images: [4, 5],
    },
  ];

  expect(align(config, chunks)).toEqual(expected);
});
