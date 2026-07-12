CREATE TABLE addresses (
    id SERIAL PRIMARY KEY,
    pid UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),

    customer_id INTEGER NOT NULL REFERENCES users (id) ON DELETE RESTRICT,
    address_type TEXT NOT NULL DEFAULT 'shipping' CHECK (address_type IN ('billing', 'shipping', 'other')),
    label VARCHAR(120),
    recipient_name VARCHAR(255) NOT NULL CHECK (char_length(btrim(recipient_name)) > 0),
    company VARCHAR(255),
    line_one VARCHAR(255) NOT NULL CHECK (char_length(btrim(line_one)) > 0),
    line_two VARCHAR(255),
    city VARCHAR(120) NOT NULL CHECK (char_length(btrim(city)) > 0),
    region VARCHAR(120),
    postal_code VARCHAR(32),
    country_code VARCHAR(2) NOT NULL CHECK (country_code ~ '^[A-Z]{2}$'),
    email CITEXT,
    phone VARCHAR(32),
    is_default BOOLEAN NOT NULL DEFAULT false,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

CREATE TRIGGER addresses_updated_at_trigger
BEFORE UPDATE ON addresses
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE INDEX addresses_customer_idx
ON addresses (customer_id, created_at DESC)
WHERE deleted_at IS NULL;

CREATE UNIQUE INDEX one_default_address_per_type
ON addresses (customer_id, address_type)
WHERE is_default AND deleted_at IS NULL;

CREATE TABLE orders (
    id SERIAL PRIMARY KEY,
    pid UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    order_number BIGSERIAL NOT NULL UNIQUE,

    customer_id INTEGER NOT NULL REFERENCES users (id) ON DELETE RESTRICT,
    customer_name VARCHAR(255) NOT NULL CHECK (char_length(btrim(customer_name)) > 0),
    customer_email CITEXT NOT NULL,
    billing_address_id INTEGER REFERENCES addresses (id) ON DELETE SET NULL,
    shipping_address_id INTEGER REFERENCES addresses (id) ON DELETE SET NULL,

    -- Immutable copies protect historical orders when a customer edits an address.
    billing_address_snapshot JSONB NOT NULL DEFAULT '{}'::jsonb
        CHECK (jsonb_typeof(billing_address_snapshot) = 'object'),
    shipping_address_snapshot JSONB NOT NULL DEFAULT '{}'::jsonb
        CHECK (jsonb_typeof(shipping_address_snapshot) = 'object'),

    status TEXT NOT NULL DEFAULT 'pending' CHECK (
        status IN ('pending', 'confirmed', 'processing', 'completed', 'cancelled', 'refunded')
    ),
    payment_status TEXT NOT NULL DEFAULT 'pending' CHECK (
        payment_status IN ('pending', 'authorized', 'paid', 'failed', 'refunded')
    ),
    fulfillment_status TEXT NOT NULL DEFAULT 'unfulfilled' CHECK (
        fulfillment_status IN ('unfulfilled', 'partial', 'fulfilled', 'cancelled')
    ),
    currency VARCHAR(3) NOT NULL CHECK (currency ~ '^[A-Z]{3}$'),

    subtotal NUMERIC(12, 2) NOT NULL CHECK (subtotal >= 0),
    discount_total NUMERIC(12, 2) NOT NULL DEFAULT 0 CHECK (discount_total >= 0),
    shipping_total NUMERIC(12, 2) NOT NULL DEFAULT 0 CHECK (shipping_total >= 0),
    tax_total NUMERIC(12, 2) NOT NULL DEFAULT 0 CHECK (tax_total >= 0),
    grand_total NUMERIC(12, 2) NOT NULL CHECK (
        grand_total >= 0
        AND grand_total = subtotal - discount_total + shipping_total + tax_total
    ),

    customer_note TEXT,
    staff_note TEXT,
    placed_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER orders_updated_at_trigger
BEFORE UPDATE ON orders
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE INDEX orders_customer_created_idx
ON orders (customer_id, created_at DESC);
CREATE INDEX orders_status_created_idx
ON orders (status, created_at DESC);
CREATE INDEX orders_payment_status_idx
ON orders (payment_status);
CREATE INDEX orders_fulfillment_status_idx
ON orders (fulfillment_status);

CREATE TABLE order_items (
    id SERIAL PRIMARY KEY,
    pid UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),

    order_id INTEGER NOT NULL REFERENCES orders (id) ON DELETE CASCADE,
    product_id INTEGER REFERENCES products (id) ON DELETE SET NULL,
    variant_id INTEGER REFERENCES product_variants (id) ON DELETE SET NULL,

    -- Product identifiers and commercial values are snapshots, not live catalogue data.
    product_pid UUID NOT NULL,
    variant_pid UUID NOT NULL,
    product_name VARCHAR(255) NOT NULL,
    sku VARCHAR(64) NOT NULL,
    selected_options JSONB NOT NULL DEFAULT '{}'::jsonb
        CHECK (jsonb_typeof(selected_options) = 'object'),

    quantity INTEGER NOT NULL CHECK (quantity > 0),
    unit_price NUMERIC(12, 2) NOT NULL CHECK (unit_price >= 0),
    discount_total NUMERIC(12, 2) NOT NULL DEFAULT 0 CHECK (discount_total >= 0),
    tax_total NUMERIC(12, 2) NOT NULL DEFAULT 0 CHECK (tax_total >= 0),
    line_total NUMERIC(12, 2) NOT NULL CHECK (
        line_total >= 0
        AND line_total = unit_price * quantity - discount_total + tax_total
    ),

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER order_items_updated_at_trigger
BEFORE UPDATE ON order_items
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE INDEX order_items_order_idx
ON order_items (order_id, id);
CREATE INDEX order_items_product_idx
ON order_items (product_id)
WHERE product_id IS NOT NULL;
CREATE INDEX order_items_variant_idx
ON order_items (variant_id)
WHERE variant_id IS NOT NULL;
