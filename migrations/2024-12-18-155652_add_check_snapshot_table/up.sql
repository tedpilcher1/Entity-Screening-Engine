-- Your SQL goes here

















CREATE TABLE "check_snapshot"(
	"check_id" UUID NOT NULL,
	"snapshot_id" UUID NOT NULL,
	PRIMARY KEY("check_id", "snapshot_id")
);

