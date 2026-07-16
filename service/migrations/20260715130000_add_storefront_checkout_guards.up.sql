ALTER TABLE orders
ADD COLUMN checkout_key UUID;

CREATE UNIQUE INDEX orders_customer_checkout_key_idx
ON orders (customer_id, checkout_key)
WHERE checkout_key IS NOT NULL;

ALTER TABLE order_items
ADD COLUMN product_slug TEXT,
ADD COLUMN image_url TEXT;
