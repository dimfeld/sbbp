export interface ViewerConfig {
  title: string;
  chunks: ViewerChunk[];
}

export interface ViewerChunk {
  timestamp: [number, number];
  text: string;
  images: number[];
}

export interface TranscriptChunk {
  timestamp: [number, number];
  text: string;
}

export interface VideoViewerData {
  read: boolean;
  progress: number;
}

export interface Video {
  id: number;
  viewerData: VideoViewerData;
  title: string;
  originalVideoPath: string;
  processedPath: string;
  summary: string;
  images: {
    maxIndex: number;
    removed: number[];
    interval: number;
  };
  duration: number;
}
