-- Your SQL goes here










CREATE TYPE UPDATEKIND AS ENUM ('company', 'officer', 'shareholder');







ALTER TABLE "processed_update" ADD COLUMN "kind" UPDATEKIND NOT NULL;



