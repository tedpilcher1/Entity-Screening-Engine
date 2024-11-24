-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS "check";
DROP TABLE IF EXISTS "relationship";
DROP TABLE IF EXISTS "entity";
DROP TABLE IF EXISTS "check_entity_map";
DROP TYPE IF EXISTS "ENTITYKIND";
DROP TYPE IF EXISTS "RELATIONSHIPKIND";