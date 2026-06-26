-- Add up migration script here
CREATE TABLE categories (
  id SERIAL PRIMARY KEY,
  pid UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),

  name VARCHAR(255) NOT NULL UNIQUE CHECK (char_length(name) > 0),
  description TEXT,

  image_link TEXT NOT NULL,

  parent_id INTEGER,

  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

  deleted_at TIMESTAMPTZ,

  FOREIGN KEY (parent_id) REFERENCES categories (id)
);


CREATE TRIGGER categories_updated_at_trigger
BEFORE UPDATE ON categories
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();
