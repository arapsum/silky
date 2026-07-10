DROP INDEX IF EXISTS product_tags_tag_id_idx;
DROP TABLE IF EXISTS product_tags;

DROP TRIGGER IF EXISTS tags_updated_at_trigger ON tags;
DROP INDEX IF EXISTS tags_name_unique;
DROP TABLE IF EXISTS tags;
