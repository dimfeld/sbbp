import type { SyncPrerecordedResponse } from '@deepgram/sdk';

export type VideoTranscript = SyncPrerecordedResponse & { _provider_format: 'deepgram_v1' };
