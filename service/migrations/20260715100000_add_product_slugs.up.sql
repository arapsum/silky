ALTER TABLE products
ADD COLUMN slug TEXT;

WITH normalized_products AS (
    SELECT
        id,
        COALESCE(
            NULLIF(
                TRIM(BOTH '-' FROM REGEXP_REPLACE(LOWER(TRIM(name)), '[^a-z0-9]+', '-', 'g')),
                ''
            ),
            'product'
        ) AS base_slug
    FROM products
),
ranked_products AS (
    SELECT
        id,
        base_slug,
        ROW_NUMBER() OVER (PARTITION BY base_slug ORDER BY id) AS slug_position
    FROM normalized_products
)
UPDATE products
SET slug = CASE
    WHEN ranked_products.slug_position = 1 THEN ranked_products.base_slug
    ELSE ranked_products.base_slug || '-' || ranked_products.slug_position
END
FROM ranked_products
WHERE products.id = ranked_products.id;

ALTER TABLE products
ALTER COLUMN slug SET NOT NULL;

ALTER TABLE products
ADD CONSTRAINT products_slug_not_empty CHECK (char_length(slug) > 0);

CREATE UNIQUE INDEX products_slug_unique_idx
ON products (slug);
