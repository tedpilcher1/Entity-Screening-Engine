use diesel::{allow_tables_to_appear_in_same_query, joinable};

pub mod sql_types {
    use diesel::{query_builder::QueryId, sql_types::SqlType};

    #[derive(SqlType, QueryId)]
    #[diesel(postgres_type(name = "RelationshipKind"))]
    pub struct RelationshipKind;
    #[derive(SqlType, QueryId)]
    #[diesel(postgres_type(name = "EntityKind"))]
    pub struct EntityKind;
}

diesel::table! {
    use diesel::sql_types::*;
    check(id) {
        id -> Uuid,
        started_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    check_entity_map(check_id, entity_id) {
        check_id -> Uuid,
        entity_id -> Uuid,
    }
}

joinable!(check_entity_map -> check(check_id));
allow_tables_to_appear_in_same_query!(check_entity_map, check);

joinable!(check_entity_map -> entity(entity_id));
allow_tables_to_appear_in_same_query!(check_entity_map, entity);

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::EntityKind;

    entity(id) {
        id -> Uuid,
        company_house_number -> Text,
        name -> Nullable<Text>,
        kind -> Nullable<EntityKind>,
        country -> Nullable<Text>,
        postal_code -> Nullable<Text>,
        date_of_origin -> Nullable<Text>,
        is_root -> Bool,
    }
}

// Parent is x of child
// i.e. parent is officer of child
// i.e. parent is shareholder of child
diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::RelationshipKind;

    relationship(parent_id, child_id) {
        parent_id -> Uuid,
        child_id -> Uuid,
        kind -> RelationshipKind,
    }
}
joinable!(relationship -> entity(parent_id));
allow_tables_to_appear_in_same_query!(entity, relationship);