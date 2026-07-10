CREATE TABLE media_assets (
    id SERIAL PRIMARY KEY,
    pid UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),

    provider VARCHAR(32) NOT NULL DEFAULT 'cloudinary',
    resource_type VARCHAR(32) NOT NULL DEFAULT 'image',
    public_id TEXT NOT NULL UNIQUE,
    asset_id TEXT UNIQUE,
    secure_url TEXT NOT NULL,
    folder TEXT NOT NULL,
    format VARCHAR(32),
    bytes BIGINT,
    width INTEGER,
    height INTEGER,
    checksum VARCHAR(128),
    status VARCHAR(24) NOT NULL DEFAULT 'active'
        CHECK (status IN ('pending', 'active', 'quarantined', 'deleted')),
    created_by INTEGER REFERENCES users (id) ON DELETE SET NULL,
    orphaned_at TIMESTAMPTZ,
    purge_after TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX media_assets_checksum_unique
ON media_assets (checksum)
WHERE checksum IS NOT NULL AND status <> 'deleted';

CREATE INDEX media_assets_status_purge_after_idx
ON media_assets (status, purge_after);

CREATE INDEX media_assets_folder_idx
ON media_assets (folder);

CREATE TRIGGER media_assets_updated_at_trigger
BEFORE UPDATE ON media_assets
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

ALTER TABLE users
ADD COLUMN media_asset_id INTEGER REFERENCES media_assets (id) ON DELETE SET NULL;

ALTER TABLE categories
ADD COLUMN media_asset_id INTEGER REFERENCES media_assets (id) ON DELETE SET NULL;

ALTER TABLE pictures
ADD COLUMN media_asset_id INTEGER REFERENCES media_assets (id) ON DELETE SET NULL;

CREATE INDEX users_media_asset_id_idx ON users (media_asset_id);
CREATE INDEX categories_media_asset_id_idx ON categories (media_asset_id);
CREATE INDEX pictures_media_asset_id_idx ON pictures (media_asset_id);
