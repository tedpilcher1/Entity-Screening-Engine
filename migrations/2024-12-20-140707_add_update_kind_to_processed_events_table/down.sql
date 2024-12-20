-- This file should undo anything in `up.sql`

















ALTER TABLE "processed_update" DROP COLUMN "kind";


DROP TYPE IF EXISTS "UPDATEKIND";

