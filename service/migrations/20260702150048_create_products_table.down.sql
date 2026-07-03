-- Add down migration script here

DROP TRIGGER IF EXISTS pictures_updated_at_trigger ON pictures;
DROP TRIGGER IF EXISTS product_variants_updated_at_trigger ON product_variants;
DROP TRIGGER IF EXISTS attribute_values_updated_at_trigger ON attribute_values;
DROP TRIGGER IF EXISTS attributes_updated_at_trigger ON attributes;
DROP TRIGGER IF EXISTS products_updated_at_trigger ON products;

DROP INDEX IF EXISTS one_default_variant_per_product;

DROP TABLE IF EXISTS pictures;
DROP TABLE IF EXISTS variant_attribute_values;
DROP TABLE IF EXISTS product_variants;
DROP TABLE IF EXISTS product_options;
DROP TABLE IF EXISTS attribute_values;
DROP TABLE IF EXISTS attributes;
DROP TABLE IF EXISTS products;
