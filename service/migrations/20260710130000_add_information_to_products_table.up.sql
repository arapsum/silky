ALTER TABLE products
ADD COLUMN information JSONB NOT NULL DEFAULT '{}'::jsonb;
