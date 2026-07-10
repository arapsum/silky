CREATE TABLE tags (
    id SERIAL PRIMARY KEY,
    pid UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    name VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT tags_name_check CHECK (
        char_length(btrim(name)) BETWEEN 1 AND 64
    )
);

CREATE UNIQUE INDEX tags_name_unique ON tags (LOWER(name));

CREATE TRIGGER tags_updated_at_trigger
BEFORE UPDATE ON tags
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE TABLE product_tags (
    product_id INTEGER NOT NULL REFERENCES products (id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (product_id, tag_id)
);

CREATE INDEX product_tags_tag_id_idx ON product_tags (tag_id);
