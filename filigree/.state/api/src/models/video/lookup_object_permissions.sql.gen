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
  AND permission IN ('org_admin', 'Video::owner', 'Video::write', 'Video::read')
