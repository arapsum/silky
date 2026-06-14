-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS citext;

CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = now();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TABLE users (
  id SERIAL PRIMARY KEY,

  pid UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),

  name VARCHAR(255) NOT NULL CHECK (char_length(name) > 0),
  email CITEXT NOT NULL UNIQUE,

  password_hash TEXT NOT NULL CHECK (char_length(password_hash) > 0),

  verified_at TIMESTAMPTZ,

  verification_token_hash TEXT,
  verification_token_expires_at TIMESTAMPTZ,

  reset_token_hash TEXT,
  reset_token_expires_at TIMESTAMPTZ,

  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

  deleted_at TIMESTAMPTZ
);

ALTER TABLE users
ADD CONSTRAINT verification_token_consistency
CHECK (
  (verification_token_hash IS NULL AND verification_token_expires_at IS NULL)
  OR
  (verification_token_hash IS NOT NULL AND verification_token_expires_at IS NOT NULL)
);

ALTER TABLE users
ADD CONSTRAINT reset_token_consistency
CHECK (
  (reset_token_hash IS NULL AND reset_token_expires_at IS NULL)
  OR
  (reset_token_hash IS NOT NULL AND reset_token_expires_at IS NOT NULL)
);


CREATE TRIGGER users_updated_at_trigger
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();
