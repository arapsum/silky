-- Add down migration script here
DROP TRIGGER IF EXISTS permissions_updated_at_trigger ON permissions;

-- Drop table
DROP TABLE IF EXISTS permissions;
