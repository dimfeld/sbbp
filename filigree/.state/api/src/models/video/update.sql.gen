WITH permissions AS (
  SELECT
    COALESCE(bool_or(permission IN ('org_admin', 'Video::owner')), FALSE) AS is_owner,
    COALESCE(bool_or(permission IN ('org_admin', 'Video::owner', 'Video::write')), FALSE) AS is_user
  FROM
    public.permissions
  WHERE
    organization_id = $2
    AND actor_id = ANY ($3)
    AND permission IN ('org_admin', 'Video::owner', 'Video::write'))
UPDATE
  public.videos
SET
  title = CASE WHEN permissions.is_owner THEN
    $4
  ELSE
    videos.title
  END,
  read = CASE WHEN permissions.is_owner THEN
    $5
  ELSE
    videos.read
  END,
  progress = CASE WHEN permissions.is_owner THEN
    $6
  ELSE
    videos.progress
  END,
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
