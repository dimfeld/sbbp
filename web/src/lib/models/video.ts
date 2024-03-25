import { client, type ModelDefinition } from "filigree-web";
import { z } from "zod";
import { ObjectPermission } from "../model_types.js";

export const VideoSchema = z.object({
	id: z.string(),
	organization_id: z.string(),
	updated_at: z.string().datetime(),
	created_at: z.string().datetime(),
	processing_state: z.string(),
	url: z.string().optional(),
	images: z.any().optional(),
	title: z.string().optional(),
	duration: z.number().int().optional(),
	read: z.boolean(),
	progress: z.number().int(),
	summary: z.string().optional(),
	processed_path: z.string().optional(),
	_permission: ObjectPermission,
});

export type Video = z.infer<typeof VideoSchema>;
export const VideoPopulatedGetSchema = VideoSchema;
export type VideoPopulatedGet = Video;
export const VideoPopulatedListSchema = VideoSchema;
export type VideoPopulatedList = Video;
export const VideoCreateResultSchema = VideoSchema;
export type VideoCreateResult = Video;

export const VideoCreatePayloadAndUpdatePayloadSchema = z.object({
	id: z.string().optional(),
	processing_state: z.string(),
	url: z.string().optional(),
	images: z.any().optional(),
	title: z.string().optional(),
	duration: z.number().int().optional(),
	read: z.boolean(),
	progress: z.number().int(),
	summary: z.string().optional(),
	processed_path: z.string().optional(),
});

export type VideoCreatePayloadAndUpdatePayload = z.infer<
	typeof VideoCreatePayloadAndUpdatePayloadSchema
>;
export const VideoCreatePayloadSchema =
	VideoCreatePayloadAndUpdatePayloadSchema;
export type VideoCreatePayload = VideoCreatePayloadAndUpdatePayload;
export const VideoUpdatePayloadSchema =
	VideoCreatePayloadAndUpdatePayloadSchema;
export type VideoUpdatePayload = VideoCreatePayloadAndUpdatePayload;

export const baseUrl = "videos";
export const urlWithId = (id: string) => `${baseUrl}/${id}`;

export const urls = {
	create: baseUrl,
	list: baseUrl,
	get: urlWithId,
	update: urlWithId,
	delete: urlWithId,
};

export const VideoModel: ModelDefinition<typeof VideoSchema> = {
	name: "Video",
	plural: "Videos",
	baseUrl,
	urls,
	schema: VideoSchema,
	createSchema: VideoCreatePayloadSchema,
	updateSchema: VideoUpdatePayloadSchema,
	fields: [
		{
			name: "id",
			type: "uuid",
			label: "Id",
			constraints: {
				required: true,
			},
		},
		{
			name: "organization_id",
			type: "uuid",
			label: "Organization Id",
			constraints: {
				required: true,
			},
		},
		{
			name: "updated_at",
			type: "date-time",
			label: "Updated At",
			constraints: {
				required: true,
			},
		},
		{
			name: "created_at",
			type: "date-time",
			label: "Created At",
			constraints: {
				required: true,
			},
		},
		{
			name: "processing_state",
			type: "text",
			label: "Processing State",
			constraints: {
				required: true,
			},
		},
		{
			name: "url",
			type: "text",
			label: "Url",
			constraints: {
				required: false,
			},
		},
		{
			name: "images",
			type: "object",
			label: "Images",
			constraints: {
				required: false,
			},
		},
		{
			name: "title",
			type: "text",
			label: "Title",
			constraints: {
				required: false,
			},
		},
		{
			name: "duration",
			type: "integer",
			label: "Duration",
			description: "Duration in seconds",
			constraints: {
				required: false,
			},
		},
		{
			name: "read",
			type: "boolean",
			label: "Read",
			constraints: {
				required: true,
			},
		},
		{
			name: "progress",
			type: "integer",
			label: "Progress",
			constraints: {
				required: true,
			},
		},
		{
			name: "summary",
			type: "text",
			label: "Summary",
			description: "Generated summary of the video",
			constraints: {
				required: false,
			},
		},
		{
			name: "processed_path",
			type: "text",
			label: "Processed Path",
			constraints: {
				required: false,
			},
		},
	],
};
