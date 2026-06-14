-- Add down migration script here
-- Remove trigger
DROP TRIGGER IF EXISTS users_updated_at_trigger ON users;

-- Remove table (this also removes its constraints)
DROP TABLE IF EXISTS users;

-- Remove function
DROP FUNCTION IF EXISTS set_updated_at();

-- Remove extension
DROP EXTENSION IF EXISTS citext;
