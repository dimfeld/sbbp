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
  read = $9,
  progress = $10,
  summary = $11,
  processed_path = $12,
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
