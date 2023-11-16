export interface ConfigTextChunk {
  text: string;
  timestamp: [number, number];
}

export interface Config {
  title: string;
  /** The number of seconds between each image */
  imageInterval: number;
  numImages: number;
  text: ConfigTextChunk[];
}

export interface ViewerConfig {
  title: string;
  chunks: ViewerChunk[];
}

export interface ViewerChunk {
  timestamp: [number, number];
  text: string;
  images: number[];
}
