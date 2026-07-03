-- Add up migration script here
CREATE TABLE products (
    id SERIAL PRIMARY KEY,
    pid UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),

    category_id INTEGER NOT NULL REFERENCES categories (id) ON DELETE CASCADE,

    name VARCHAR(255) NOT NULL UNIQUE CHECK (char_length(name) > 0),
    description TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    deleted_at TIMESTAMPTZ
);

CREATE TRIGGER products_updated_at_trigger
BEFORE UPDATE ON products
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();


CREATE TABLE attributes (
    id SERIAL PRIMARY KEY,
    pid UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),

    name VARCHAR(255) NOT NULL UNIQUE CHECK (char_length(name) > 0), -- i.e,  'colour', 'size', 'material'

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);


CREATE TRIGGER attributes_updated_at_trigger
BEFORE UPDATE ON attributes
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE TABLE attribute_values (
    id SERIAL PRIMARY KEY,
    pid UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),

    attribute_id INTEGER NOT NULL REFERENCES attributes (id) ON DELETE CASCADE,

    value TEXT NOT NULL, -- i.e, 'Marine Blue', 'xl', '60% Cotton 40% Polyester'

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE (attribute_id, value),
    UNIQUE (id, attribute_id)
);


CREATE TRIGGER attribute_values_updated_at_trigger
BEFORE UPDATE ON attribute_values
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

--  Which attribute a product varies by, e.g. this product uses colour + size
CREATE TABLE product_options (
    id SERIAL PRIMARY KEY,
    pid UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),

    product_id INTEGER NOT NULL REFERENCES products (id) ON DELETE CASCADE,
    attribute_id INTEGER NOT NULL REFERENCES attributes (id) ON DELETE CASCADE,

    display_order INTEGER,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE (product_id, attribute_id)
);

CREATE TABLE product_variants (
    id SERIAL PRIMARY KEY,
    pid UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),

    product_id INTEGER NOT NULL REFERENCES products (id) ON DELETE CASCADE,

    sku VARCHAR(64) NOT NULL UNIQUE,
    price NUMERIC(10, 2) NOT NULL CHECK (price >= 0),

    stock_quantity INTEGER NOT NULL DEFAULT 0 CHECK (stock_quantity >= 0),
    is_default BOOLEAN NOT NULL DEFAULT false,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

CREATE TRIGGER product_variants_updated_at_trigger
BEFORE UPDATE ON product_variants
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE UNIQUE INDEX one_default_variant_per_product
ON product_variants (product_id)
WHERE is_default AND deleted_at IS null;


CREATE TABLE variant_attribute_values (
    id SERIAL PRIMARY KEY,
    pid UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),

    variant_id INTEGER NOT NULL REFERENCES
    product_variants (id) ON DELETE CASCADE,
    attribute_id INTEGER NOT NULL,
    attribute_value_id INTEGER NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    FOREIGN KEY (attribute_value_id, attribute_id)
    REFERENCES attribute_values (id, attribute_id) ON DELETE CASCADE,

    -- a variant can't have two values for the same attribute
    UNIQUE (variant_id, attribute_id)
);


CREATE TABLE pictures (
    id SERIAL PRIMARY KEY,
    pid UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),

    product_id INTEGER NOT NULL REFERENCES products (id) ON DELETE CASCADE,
    variant_id INTEGER REFERENCES product_variants (id) ON DELETE CASCADE,

    image_link TEXT NOT NULL,
    display_order INTEGER,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER pictures_updated_at_trigger
BEFORE UPDATE ON pictures
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();
