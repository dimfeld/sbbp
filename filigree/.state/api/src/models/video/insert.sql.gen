INSERT INTO public.videos (
  id,
  organization_id,
  title,
  read,
  progress)
VALUES (
  $1,
  $2,
  $3,
  $4,
  $5)
RETURNING
  id AS "id: VideoId",
  organization_id AS "organization_id: crate::models::organization::OrganizationId",
  updated_at,
  created_at,
  processing_state AS "processing_state: crate::models::video::VideoProcessingState",
  url,
  title,
  duration,
  author,
  date,
  metadata AS "metadata: crate::models::video::VideoMetadata",
  read,
  progress,
  images AS "images: crate::models::video::VideoImages",
  transcript,
  summary,
  processed_path,
  'owner' AS "_permission!: filigree::auth::ObjectPermission"
