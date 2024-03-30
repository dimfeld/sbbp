import { client, type ModelDefinition } from 'filigree-web';
import { z } from 'zod';
import { ObjectPermission } from '../model_types.js';
import type { VideoProcessingState } from '../api_types.js';
import type { VideoTranscript } from '../transcript.js';

export type VideoId = string;

export const VideoSchema = z.object({
  id: z.string(),
  organization_id: z.string(),
  updated_at: z.string().datetime(),
  created_at: z.string().datetime(),
  processing_state: z.custom<VideoProcessingState>(),
  url: z.string().optional(),
  title: z.string().optional(),
  duration: z.number().int().optional(),
  author: z.string().optional(),
  date: z.string().optional(),
  metadata: z.any().optional(),
  read: z.boolean(),
  progress: z.number().int(),
  images: z.any().optional(),
  transcript: z.custom<VideoTranscript>().optional(),
  summary: z.string().optional(),
  processed_path: z.string().optional(),
  _permission: ObjectPermission,
});

export type Video = z.infer<typeof VideoSchema>;
export const VideoPopulatedGetResultSchema = VideoSchema;
export type VideoPopulatedGetResult = Video;
export const VideoCreateResultSchema = VideoSchema;
export type VideoCreateResult = Video;

export const VideoCreatePayloadAndUpdatePayloadSchema = z.object({
  id: z.string().optional(),
  title: z.string().optional(),
  read: z.boolean(),
  progress: z.number().int(),
});

export type VideoCreatePayloadAndUpdatePayload = z.infer<
  typeof VideoCreatePayloadAndUpdatePayloadSchema
>;
export const VideoCreatePayloadSchema = VideoCreatePayloadAndUpdatePayloadSchema;
export type VideoCreatePayload = VideoCreatePayloadAndUpdatePayload;
export const VideoUpdatePayloadSchema = VideoCreatePayloadAndUpdatePayloadSchema;
export type VideoUpdatePayload = VideoCreatePayloadAndUpdatePayload;

export const VideoListResultAndPopulatedListResultSchema = z.object({
  id: z.string(),
  organization_id: z.string(),
  updated_at: z.string().datetime(),
  created_at: z.string().datetime(),
  processing_state: z.custom<VideoProcessingState>(),
  url: z.string().optional(),
  title: z.string().optional(),
  duration: z.number().int().optional(),
  author: z.string().optional(),
  date: z.string().optional(),
  metadata: z.any().optional(),
  read: z.boolean(),
  progress: z.number().int(),
  summary: z.string().optional(),
  processed_path: z.string().optional(),
  _permission: ObjectPermission,
});

export type VideoListResultAndPopulatedListResult = z.infer<
  typeof VideoListResultAndPopulatedListResultSchema
>;
export const VideoListResultSchema = VideoListResultAndPopulatedListResultSchema;
export type VideoListResult = VideoListResultAndPopulatedListResult;
export const VideoPopulatedListResultSchema = VideoListResultAndPopulatedListResultSchema;
export type VideoPopulatedListResult = VideoListResultAndPopulatedListResult;

export const baseUrl = 'videos';
export const urlWithId = (id: string) => `${baseUrl}/${id}`;

export const urls = {
  create: baseUrl,
  list: baseUrl,
  get: urlWithId,
  update: urlWithId,
  delete: urlWithId,
};

export const VideoModel: ModelDefinition<typeof VideoSchema> = {
  name: 'Video',
  plural: 'Videos',
  baseUrl,
  urls,
  schema: VideoSchema,
  createSchema: VideoCreatePayloadSchema,
  updateSchema: VideoUpdatePayloadSchema,
  fields: [
    {
      name: 'id',
      type: 'uuid',
      label: 'Id',
      constraints: {
        required: true,
      },
    },
    {
      name: 'organization_id',
      type: 'uuid',
      label: 'Organization Id',
      constraints: {
        required: true,
      },
    },
    {
      name: 'updated_at',
      type: 'date-time',
      label: 'Updated At',
      constraints: {
        required: true,
      },
    },
    {
      name: 'created_at',
      type: 'date-time',
      label: 'Created At',
      constraints: {
        required: true,
      },
    },
    {
      name: 'processing_state',
      type: 'text',
      label: 'Processing State',
      constraints: {
        required: true,
      },
    },
    {
      name: 'url',
      type: 'text',
      label: 'Url',
      constraints: {
        required: false,
      },
    },
    {
      name: 'title',
      type: 'text',
      label: 'Title',
      constraints: {
        required: false,
      },
    },
    {
      name: 'duration',
      type: 'integer',
      label: 'Duration',
      description: 'Duration in seconds',
      constraints: {
        required: false,
      },
    },
    {
      name: 'author',
      type: 'text',
      label: 'Author',
      constraints: {
        required: false,
      },
    },
    {
      name: 'date',
      type: 'date',
      label: 'Date',
      constraints: {
        required: false,
      },
    },
    {
      name: 'metadata',
      type: 'object',
      label: 'Metadata',
      constraints: {
        required: false,
      },
    },
    {
      name: 'read',
      type: 'boolean',
      label: 'Read',
      constraints: {
        required: true,
      },
    },
    {
      name: 'progress',
      type: 'integer',
      label: 'Progress',
      constraints: {
        required: true,
      },
    },
    {
      name: 'images',
      type: 'object',
      label: 'Images',
      constraints: {
        required: false,
      },
    },
    {
      name: 'transcript',
      type: 'object',
      label: 'Transcript',
      constraints: {
        required: false,
      },
    },
    {
      name: 'summary',
      type: 'text',
      label: 'Summary',
      description: 'Generated summary of the video',
      constraints: {
        required: false,
      },
    },
    {
      name: 'processed_path',
      type: 'text',
      label: 'Processed Path',
      constraints: {
        required: false,
      },
    },
  ],
};

export interface CreateViaUrlArgs {
  payload: CreateViaUrlPayload;
  fetch?: typeof fetch;
}

export interface CreateViaUrlPayload {
  url: string;
}

export interface CreateViaUrlResponse {
  id: VideoId;
}

export async function create_via_url({ fetch, payload }: CreateViaUrlArgs) {
  return client({
    url: `/api/add_video`,
    method: 'POST',
    fetch,
    json: payload,
  }).json<CreateViaUrlResponse>();
}

export interface RerunStageArgs {
  id: string;
  stage: string;
  payload?: RerunStagePayload;
  fetch?: typeof fetch;
}

export interface RerunStagePayload {}

export interface RerunStageResponse {
  job_id: string;
}

export async function rerun_stage({ fetch, id, stage, payload }: RerunStageArgs) {
  return client({
    url: `/api/videos/${id}/rerun/${stage}`,
    method: 'POST',
    fetch,
    json: payload,
  }).json<RerunStageResponse>();
}

export interface MarkReadArgs {
  id: string;
  payload: MarkReadPayload;
  fetch?: typeof fetch;
}

export interface MarkReadPayload {
  read: boolean;
}

export interface MarkReadResponse {}

export async function mark_read({ fetch, id, payload }: MarkReadArgs) {
  return client({
    url: `/api/videos/${id}/mark_read`,
    method: 'POST',
    fetch,
    json: payload,
  }).json<MarkReadResponse>();
}
