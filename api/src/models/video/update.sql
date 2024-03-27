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
  title = $6,
  duration = $7,
  author = $8,
  date = $9,
  metadata = $10,
  read = $11,
  progress = $12,
  images = $13,
  transcript = $14,
  summary = $15,
  processed_path = $16,
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
