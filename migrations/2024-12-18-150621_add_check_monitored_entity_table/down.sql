-- This file should undo anything in `up.sql`
ALTER TABLE "check" DROP COLUMN "kind";













DROP TABLE IF EXISTS "check_monitored_entity";
DROP TABLE IF EXISTS "dormant_company";
