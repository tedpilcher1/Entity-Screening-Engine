-- This file should undo anything in `up.sql`
ALTER TABLE "check" DROP COLUMN "kind";



DROP TYPE IF EXISTS "CHECKKIND";

