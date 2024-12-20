-- This file should undo anything in `up.sql`













ALTER TABLE "monitoring_span" DROP COLUMN "started_at";
ALTER TABLE "monitoring_span" DROP COLUMN "ended_at";
ALTER TABLE "monitoring_span" ADD COLUMN "started_at" DATE NOT NULL;
ALTER TABLE "monitoring_span" ADD COLUMN "ended_at" DATE;




ALTER TABLE "processed_update" DROP COLUMN "processed_at";
ALTER TABLE "processed_update" ADD COLUMN "processed_at" DATE NOT NULL;


ALTER TABLE "snapshot" DROP COLUMN "recieved_at";
ALTER TABLE "snapshot" ADD COLUMN "recieved_at" DATE NOT NULL;

