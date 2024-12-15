// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "entitykind"))]
    pub struct Entitykind;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "flagkind"))]
    pub struct Flagkind;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "relationshipkind"))]
    pub struct Relationshipkind;
}

diesel::table! {
    check (id) {
        id -> Uuid,
        started_at -> Timestamp,
    }
}

diesel::table! {
    check_entity_map (check_id, entity_id) {
        check_id -> Uuid,
        entity_id -> Uuid,
    }
}

diesel::table! {
    check_job_map (check_id, job_id) {
        check_id -> Uuid,
        job_id -> Uuid,
    }
}

diesel::table! {
    dataset (id) {
        id -> Uuid,
        name -> Text,
    }
}

diesel::table! {
    datasets (entity_id, dataset_id) {
        entity_id -> Uuid,
        dataset_id -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Entitykind;

    entity (id) {
        id -> Uuid,
        company_house_number -> Text,
        name -> Nullable<Text>,
        kind -> Entitykind,
        country -> Nullable<Text>,
        postal_code -> Nullable<Text>,
        date_of_origin -> Nullable<Text>,
        is_root -> Bool,
        officer_id -> Nullable<Text>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Flagkind;

    flag (id) {
        id -> Uuid,
        kind -> Flagkind,
    }
}

diesel::table! {
    flags (entity_id, flag_id) {
        entity_id -> Uuid,
        flag_id -> Uuid,
    }
}

diesel::table! {
    job (id) {
        id -> Uuid,
        enqueued_at -> Timestamp,
        completed_at -> Nullable<Timestamp>,
        has_error -> Bool,
    }
}

diesel::table! {
    position (id) {
        id -> Uuid,
        title -> Text,
    }
}

diesel::table! {
    positions (entity_id, position_id) {
        entity_id -> Uuid,
        position_id -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Relationshipkind;

    relationship (parent_id, child_id) {
        parent_id -> Uuid,
        child_id -> Uuid,
        kind -> Relationshipkind,
        started_on -> Nullable<Date>,
        ended_on -> Nullable<Date>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    check,
    check_entity_map,
    check_job_map,
    dataset,
    datasets,
    entity,
    flag,
    flags,
    job,
    position,
    positions,
    relationship,
);
