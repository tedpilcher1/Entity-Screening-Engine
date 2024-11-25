-- Your SQL goes here





CREATE TABLE "check_job_map"(
	"check_id" UUID NOT NULL,
	"job_id" UUID NOT NULL,
	PRIMARY KEY("check_id", "job_id")
);

