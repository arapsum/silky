-- Add up migration script here
CREATE TABLE roles_permissions (
  id SERIAL PRIMARY KEY,
  pid UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),

  role_id INTEGER NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
  permission_id INTEGER NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,

  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

  CONSTRAINT roles_permissions_role_id_permission_id_unique UNIQUE (role_id, permission_id)
);


CREATE TRIGGER roles_permissions_updated_at_trigger
BEFORE UPDATE ON roles_permissions
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();
