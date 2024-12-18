-- Your SQL goes here
CREATE TYPE CHECKKIND AS ENUM ('entity_relation', 'monitored_entity');

ALTER TABLE "check" ADD COLUMN "kind" CHECKKIND NOT NULL;
