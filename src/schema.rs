// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "checkkind"))]
    pub struct Checkkind;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "entitykind"))]
    pub struct Entitykind;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "flagkind"))]
    pub struct Flagkind;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "relationshipkind"))]
    pub struct Relationshipkind;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "updatekind"))]
    pub struct Updatekind;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Checkkind;

    check (id) {
        id -> Uuid,
        started_at -> Timestamp,
        kind -> Checkkind,
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
    check_monitored_entity (check_id, monitored_entity_id) {
        check_id -> Uuid,
        monitored_entity_id -> Uuid,
    }
}

diesel::table! {
    check_snapshot (check_id, snapshot_id) {
        check_id -> Uuid,
        snapshot_id -> Uuid,
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
    dormant_company (entity_id) {
        entity_id -> Uuid,
        dormant -> Bool,
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
    monitored_entity (id) {
        id -> Uuid,
        company_house_id -> Text,
        monitoring_span_id -> Uuid,
    }
}

diesel::table! {
    monitoring_span (id) {
        id -> Uuid,
        started_at -> Timestamp,
        ended_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    outlier_age (entity_id) {
        entity_id -> Uuid,
        outlier -> Bool,
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
    use super::sql_types::Updatekind;

    processed_update (id) {
        id -> Uuid,
        timepoint -> Int4,
        kind -> Updatekind,
        processed_at -> Timestamp,
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

diesel::table! {
    snapshot (id) {
        id -> Uuid,
        entity_id -> Uuid,
        recieved_at -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    check,
    check_entity_map,
    check_job_map,
    check_monitored_entity,
    check_snapshot,
    dataset,
    datasets,
    dormant_company,
    entity,
    flag,
    flags,
    job,
    monitored_entity,
    monitoring_span,
    outlier_age,
    position,
    positions,
    processed_update,
    relationship,
    snapshot,
);
