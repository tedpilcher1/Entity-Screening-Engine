-- Your SQL goes here
ALTER TABLE "check" ADD COLUMN "kind" CHECKKIND NOT NULL;













CREATE TABLE "dormant_company"(
	"entity_id" UUID NOT NULL PRIMARY KEY,
	"dormant" BOOL NOT NULL
);

CREATE TABLE "monitored_entity"(
	"id" UUID NOT NULL PRIMARY KEY,
	"company_house_id" TEXT NOT NULL
);

CREATE TABLE "check_monitored_entity"(
	"check_id" UUID NOT NULL,
	"monitored_entity_id" UUID NOT NULL,
	PRIMARY KEY("check_id", "monitored_entity_id")
);

