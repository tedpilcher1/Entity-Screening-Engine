-- Your SQL goes here













ALTER TABLE "monitoring_span" DROP COLUMN "started_at";
ALTER TABLE "monitoring_span" DROP COLUMN "ended_at";
ALTER TABLE "monitoring_span" ADD COLUMN "started_at" TIMESTAMP NOT NULL;
ALTER TABLE "monitoring_span" ADD COLUMN "ended_at" TIMESTAMP;




ALTER TABLE "processed_update" DROP COLUMN "processed_at";
ALTER TABLE "processed_update" ADD COLUMN "processed_at" TIMESTAMP NOT NULL;


ALTER TABLE "snapshot" DROP COLUMN "recieved_at";
ALTER TABLE "snapshot" ADD COLUMN "recieved_at" TIMESTAMP NOT NULL;

