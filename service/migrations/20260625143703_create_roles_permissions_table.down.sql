-- Add down migration script here
DROP TRIGGER IF EXISTS roles_permissions_updated_at_trigger ON roles_permissions;

-- Drop table
DROP TABLE IF EXISTS roles_permissions;
