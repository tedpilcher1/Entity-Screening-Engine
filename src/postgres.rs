use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use diesel::{insert_into, Connection, PgConnection};
use uuid::Uuid;

use crate::model::{EntityKind, RelationshipKind};
use crate::schema::{check, check_entity_map, entity, relationship};

#[derive(Debug, Queryable, Insertable)]
#[diesel(table_name = entity)]
pub struct Entity {
    id: Uuid,
    company_house_number: String,
    name: Option<String>,
    kind: Option<EntityKind>,
    country: Option<String>,
    postal_code: Option<String>,
    date_of_origin: Option<String>,
    is_root: bool,
}

#[derive(Debug, Queryable, Insertable)]
#[diesel(table_name = check_entity_map)]
pub struct CheckEntityMap {
    check_id: Uuid,
    entity_id: Uuid,
}

#[derive(Debug, Queryable, Insertable)]
#[diesel(table_name = relationship)]
pub struct Relationship {
    parent_id: Uuid,
    child_id: Uuid,
    kind: RelationshipKind,
}

#[derive(Debug, Queryable, Insertable)]
#[diesel(table_name = check)]
pub struct Check {
    id: Uuid,
    started_at: NaiveDateTime,
}

pub struct Database {
    conn: PgConnection,
}

impl Database {
    pub async fn connect() -> Result<Self, failure::Error> {
        let database_url = std::env::var("DATABASE_URL")?;
        let conn = PgConnection::establish(&database_url)?;
        Ok(Self { conn })
    }

    pub async fn insert_check(&mut self) -> Result<Uuid, failure::Error> {
        let id = Uuid::new_v4();

        insert_into(check::table)
            .values(&Check {
                id,
                started_at: Utc::now().naive_utc(),
            })
            .execute(&mut self.conn)?;

        Ok(id)
    }

    pub async fn insert_entity(
        &mut self,
        entity: &Entity,
        check_id: Uuid,
    ) -> Result<Uuid, failure::Error> {
        match self
            .get_existing_entity_id(&check_id, &entity.company_house_number)
            .await?
        {
            Some(id) => return Ok(id),
            None => {}
        }

        self.conn.transaction(|conn| {
            insert_into(entity::table).values(entity).execute(conn)?;

            insert_into(check_entity_map::table)
                .values(&CheckEntityMap {
                    check_id,
                    entity_id: entity.id,
                })
                .execute(conn)?;

            diesel::result::QueryResult::Ok(())
        })?;

        Ok(entity.id)
    }

    pub async fn insert_relationship(
        &mut self,
        relationship: &Relationship,
    ) -> Result<(), failure::Error> {
        insert_into(relationship::table)
            .values(relationship)
            .execute(&mut self.conn)?;

        Ok(())
    }

    pub async fn get_relations(
        &mut self,
        entity_id: Uuid,
        relationship_kind: RelationshipKind,
    ) -> Result<Vec<Entity>, failure::Error> {
        let relations = entity::table
            .inner_join(relationship::table.on(relationship::parent_id.eq(entity::id)))
            .filter(relationship::child_id.eq(entity_id))
            .filter(relationship::kind.eq(relationship_kind))
            .select(entity::all_columns)
            .load::<Entity>(&mut self.conn)?;

        Ok(relations)
    }

    pub async fn get_existing_entity_id(
        &mut self,
        check_id: &Uuid,
        company_house_number: &String,
    ) -> Result<Option<Uuid>, failure::Error> {
        let entity_id = entity::table
            .inner_join(check_entity_map::table.on(check_entity_map::entity_id.eq(entity::id)))
            .filter(check_entity_map::check_id.eq(check_id))
            .filter(entity::company_house_number.eq(company_house_number))
            .select(entity::id)
            .first::<Uuid>(&mut self.conn)
            .optional()?;

        Ok(entity_id)
    }
}
