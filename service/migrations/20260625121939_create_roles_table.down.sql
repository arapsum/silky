-- Add down migration script here
DROP TRIGGER IF EXISTS roles_updated_at_trigger ON roles;

-- Drop table
DROP TABLE IF EXISTS roles;
