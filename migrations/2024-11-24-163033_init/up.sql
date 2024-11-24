-- Your SQL goes here
CREATE TYPE ENTITYKIND AS ENUM ('company', 'individual');
CREATE TYPE RELATIONSHIPKIND AS ENUM ('shareholder', 'officer');

CREATE TABLE "check"(
	"id" UUID NOT NULL PRIMARY KEY,
	"started_at" TIMESTAMP NOT NULL
);

CREATE TABLE "relationship"(
	"parent_id" UUID NOT NULL,
	"child_id" UUID NOT NULL,
	"kind" RELATIONSHIPKIND NOT NULL,
	PRIMARY KEY("parent_id", "child_id")
);

CREATE TABLE "entity"(
	"id" UUID NOT NULL PRIMARY KEY,
	"company_house_number" TEXT NOT NULL,
	"name" TEXT,
	"kind" ENTITYKIND NOT NULL,
	"country" TEXT,
	"postal_code" TEXT,
	"date_of_origin" TEXT,
	"is_root" BOOL NOT NULL
);

CREATE TABLE "check_entity_map"(
	"check_id" UUID NOT NULL,
	"entity_id" UUID NOT NULL,
	PRIMARY KEY("check_id", "entity_id")
);


