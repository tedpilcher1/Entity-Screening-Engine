-- Your SQL goes here
CREATE TYPE FLAGKIND AS ENUM (
    'crime',
    'fraud',
    'cybercrime',
    'financial_crime',
    'environment_violations',
    'theft',
    'war_crimes',
    'criminal_leadership',
    'terrorism',
    'trafficking',
    'drug_trafficking',
    'human_trafficking',
    'wanted',
    'offshore',
    'shell_company',
    'public_listed_company',
    'disqualified',
    'government',
    'national_government',
    'state_government',
    'municipal_government',
    'state_owned_enterprise',
    'intergovernmental_org',
    'head_of_government',
    'civil_service',
    'executive_branch_of_government',
    'legislative_branch_of_government',
    'judicial_branch_of_government',
    'security_services',
    'central_banking_and_fin_integrity',
    'financial_services',
    'bank',
    'fund',
    'financial_advisor',
    'regulator_action',
    'regulator_warning',
    'politician',
    'non_pep',
    'close_associate',
    'judge',
    'civil_servant',
    'diplomat',
    'lawyer',
    'accountant',
    'spy',
    'oligarch',
    'journalist',
    'activist',
    'lobbyist',
    'political_party',
    'union',
    'religion',
    'military',
    'frozen_asset',
    'sanctioned_entity',
    'sanction_linked_entity',
    'counter_sanctioned_entity',
    'export_controlled',
    'trade_risk',
    'debarred_entity',
    'person_of_interest'
);

CREATE TABLE "datasets"(
	"entity_id" UUID NOT NULL,
	"dataset_id" UUID NOT NULL,
	PRIMARY KEY("entity_id", "dataset_id")
);

CREATE TABLE "flags"(
	"entity_id" UUID NOT NULL,
	"flag_id" UUID NOT NULL,
	PRIMARY KEY("entity_id", "flag_id")
);

CREATE TABLE "dataset"(
	"id" UUID NOT NULL PRIMARY KEY,
	"name" TEXT NOT NULL
);

CREATE TABLE "flag"(
	"id" UUID NOT NULL PRIMARY KEY,
	"kind" FLAGKIND NOT NULL
);

