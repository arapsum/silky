-- Add up migration script here
CREATE TABLE users_roles (
  id SERIAL PRIMARY KEY,
  pid UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),

  user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  role_id INTEGER NOT NULL REFERENCES roles(id) ON DELETE CASCADE,

  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

  CONSTRAINT users_roles_user_id_role_id_unique UNIQUE (user_id, role_id)
);


CREATE TRIGGER users_roles_updated_at_trigger
BEFORE UPDATE ON users_roles
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();
