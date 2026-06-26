-- Add down migration script here

DROP TRIGGER IF EXISTS categories_updated_at_trigger ON roles;

-- Drop table
DROP TABLE IF EXISTS categories;
