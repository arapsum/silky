CREATE TABLE category_attributes (
    category_id INTEGER NOT NULL REFERENCES categories (id) ON DELETE CASCADE,
    attribute_id INTEGER NOT NULL REFERENCES attributes (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (category_id, attribute_id)
);

CREATE INDEX category_attributes_attribute_id_idx ON category_attributes (attribute_id);
