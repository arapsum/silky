DROP INDEX IF EXISTS pictures_media_asset_id_idx;
DROP INDEX IF EXISTS categories_media_asset_id_idx;
DROP INDEX IF EXISTS users_media_asset_id_idx;

ALTER TABLE pictures DROP COLUMN IF EXISTS media_asset_id;
ALTER TABLE categories DROP COLUMN IF EXISTS media_asset_id;
ALTER TABLE users DROP COLUMN IF EXISTS media_asset_id;

DROP TRIGGER IF EXISTS media_assets_updated_at_trigger ON media_assets;
DROP INDEX IF EXISTS media_assets_folder_idx;
DROP INDEX IF EXISTS media_assets_status_purge_after_idx;
DROP INDEX IF EXISTS media_assets_checksum_unique;
DROP TABLE IF EXISTS media_assets;
