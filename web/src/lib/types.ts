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

export type ProcessStatus = 'queued' | 'downloading' | 'processing' | 'error' | 'complete';

export interface VideoViewerData {
  read: boolean;
  progress: number;
  processStatus?: ProcessStatus;
}

export interface Video {
  id: number;
  viewerData: VideoViewerData;
  title: string;
  originalVideoPath: string;
  processedPath: string;
  process?: {
    error?: string;
    timing: Record<string, number>;
  };
  summary: string;
  images: {
    maxIndex: number;
    removed: number[];
    interval: number;
  };
  duration: number;
}

export type ProcessResult = Omit<Video, 'id' | 'viewerData'>;
