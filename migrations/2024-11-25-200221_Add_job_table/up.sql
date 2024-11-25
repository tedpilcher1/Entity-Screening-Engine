-- Your SQL goes here




CREATE TABLE "job"(
	"id" UUID NOT NULL PRIMARY KEY,
	"enqueued_at" TIMESTAMP NOT NULL,
	"completed_at" TIMESTAMP
);

