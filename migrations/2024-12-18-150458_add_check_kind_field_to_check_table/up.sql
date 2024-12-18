-- Your SQL goes here
ALTER TABLE "check" ADD COLUMN "kind" CHECKKIND NOT NULL;













CREATE TABLE "dormant_company"(
	"entity_id" UUID NOT NULL PRIMARY KEY,
	"dormant" BOOL NOT NULL
);

