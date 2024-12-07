// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "entitykind"))]
    pub struct Entitykind;

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
    job (id) {
        id -> Uuid,
        enqueued_at -> Timestamp,
        completed_at -> Nullable<Timestamp>,
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
    entity,
    job,
    relationship,
);
