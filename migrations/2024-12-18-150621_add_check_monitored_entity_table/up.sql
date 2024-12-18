-- Your SQL goes here
CREATE TABLE "check_monitored_entity"(
	"check_id" UUID NOT NULL,
	"monitored_entity_id" UUID NOT NULL,
	PRIMARY KEY("check_id", "monitored_entity_id")
);