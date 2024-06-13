SELECT
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
  _permission AS "_permission!: filigree::auth::ObjectPermission"
FROM
  public.videos tb
  JOIN LATERAL (
    SELECT
      CASE WHEN bool_or(permission IN ('org_admin', 'Video::owner')) THEN
        'owner'
      WHEN bool_or(permission = 'Video::write') THEN
        'write'
      WHEN bool_or(permission = 'Video::read') THEN
        'read'
      ELSE
        NULL
      END _permission
    FROM
      public.permissions
    WHERE
      organization_id = $2
      AND actor_id = ANY ($3)
      AND permission IN ('org_admin', 'Video::owner', 'Video::write', 'Video::read'))
	_permission ON _permission IS NOT NULL
WHERE
  tb.id = $1
  AND tb.organization_id = $2
