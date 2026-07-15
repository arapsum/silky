ALTER TABLE order_items
DROP COLUMN image_url,
DROP COLUMN product_slug;

DROP INDEX orders_customer_checkout_key_idx;

ALTER TABLE orders
DROP COLUMN checkout_key;
