INSERT INTO videos (
  id,
  organization_id,
  processing_state,
  url,
  title,
  duration,
  author,
  date,
  metadata,
  read,
  progress,
  images,
  transcript,
  summary,
  processed_path)
VALUES (
  $1,
  $2,
  $3,
  $4,
  $5,
  $6,
  $7,
  $8,
  $9,
  $10,
  $11,
  $12,
  $13,
  $14,
  $15)
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
  metadata,
  read,
  progress,
  images AS "images: crate::models::video::VideoImages",
  transcript,
  summary,
  processed_path,
  'owner' AS "_permission!: filigree::auth::ObjectPermission"
