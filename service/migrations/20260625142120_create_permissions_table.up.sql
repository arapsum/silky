-- Add up migration script here

CREATE TABLE permissions (
  id SERIAL PRIMARY KEY,
  pid UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),

  name VARCHAR(255) NOT NULL UNIQUE CHECK (char_length(name) > 0),
  description TEXT,

  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);


CREATE TRIGGER permissions_updated_at_trigger
BEFORE UPDATE ON permissions
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();
