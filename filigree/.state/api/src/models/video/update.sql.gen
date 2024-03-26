WITH permissions AS (
  SELECT
    COALESCE(bool_or(permission IN ('org_admin', 'Video::owner')), FALSE) AS is_owner,
    COALESCE(bool_or(permission IN ('org_admin', 'Video::owner', 'Video::write')), FALSE) AS is_user
  FROM
    permissions
  WHERE
    organization_id = $2
    AND actor_id = ANY ($3)
    AND permission IN ('org_admin', 'Video::owner', 'Video::write'))
UPDATE
  videos
SET
  processing_state = $4,
  url = $5,
  images = $6,
  title = $7,
  duration = $8,
  author = $9,
  date = $10,
  metadata = $11,
  read = $12,
  progress = $13,
  summary = $14,
  processed_path = $15,
  updated_at = now()
FROM
  permissions
WHERE
  id = $1
  AND organization_id = $2
  AND (permissions.is_owner
    OR permissions.is_user)
RETURNING
  permissions.is_owner AS "is_owner!"
