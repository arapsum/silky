DROP INDEX IF EXISTS products_slug_unique_idx;

ALTER TABLE products
DROP CONSTRAINT IF EXISTS products_slug_not_empty;

ALTER TABLE products
DROP COLUMN IF EXISTS slug;
