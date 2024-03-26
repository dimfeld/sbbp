SELECT
  id,
  organization_id,
  updated_at,
  created_at,
  processing_state,
  url,
  images,
  title,
  duration,
  author,
  date,
  metadata,
  read,
  progress,
  summary,
  processed_path,
  perm._permission
FROM
  videos tb
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
      permissions
    WHERE
      organization_id = $1
      AND actor_id = ANY ($2)
      AND permission IN ('org_admin', 'Video::owner', 'Video::write', 'Video::read')) perm ON
	perm._permission IS NOT NULL
WHERE
  organization_id = $1
  AND __insertion_point_filters
ORDER BY
  __insertion_point_order_by
LIMIT $3 OFFSET $4
