-- Your SQL goes here










CREATE TABLE "positions"(
	"entity_id" UUID NOT NULL,
	"position_id" UUID NOT NULL,
	PRIMARY KEY("entity_id", "position_id")
);

CREATE TABLE "position"(
	"id" UUID NOT NULL PRIMARY KEY,
	"title" TEXT NOT NULL
);

