-- Add down migration script here
DROP TRIGGER IF EXISTS users_roles_updated_at_trigger ON users_roles;

-- Drop table
DROP TABLE IF EXISTS users_roles;
