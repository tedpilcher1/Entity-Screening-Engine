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
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Relationshipkind;

    relationship (parent_id, child_id) {
        parent_id -> Uuid,
        child_id -> Uuid,
        kind -> Relationshipkind,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    check,
    check_entity_map,
    entity,
    relationship,
);
