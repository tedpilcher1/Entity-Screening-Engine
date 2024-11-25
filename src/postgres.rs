use chrono::Utc;
use diesel::prelude::*;
use diesel::{insert_into, Connection, PgConnection};
use uuid::Uuid;

use crate::models::{Check, CheckEntityMap, Entity, Job, Relationship, Relationshipkind};
use crate::schema::{check, check_entity_map, entity, relationship, job};

pub struct Database {
    conn: PgConnection,
}

impl Database {
    pub fn connect() -> Result<Self, failure::Error> {
        let database_url = std::env::var("DATABASE_URL")?;
        let conn = PgConnection::establish(&database_url)?;
        Ok(Self { conn })
    }

    pub fn insert_check(&mut self) -> Result<Uuid, failure::Error> {
        let id = Uuid::new_v4();

        insert_into(check::table)
            .values(&Check {
                id,
                started_at: Utc::now().naive_utc(),
            })
            .execute(&mut self.conn)?;

        Ok(id)
    }

    pub fn insert_entity(
        &mut self,
        entity: &Entity,
        check_id: Uuid,
    ) -> Result<Uuid, failure::Error> {
        match self.get_existing_entity_id(&check_id, &entity.company_house_number)? {
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

    pub fn insert_relationship(
        &mut self,
        relationship: Relationship,
    ) -> Result<(), failure::Error> {
        insert_into(relationship::table)
            .values(&relationship)
            .execute(&mut self.conn)?;

        Ok(())
    }

    pub fn get_relations(
        &mut self,
        entity_id: Uuid,
        relationship_kind: Relationshipkind,
    ) -> Result<Vec<Entity>, failure::Error> {
        let relations = entity::table
            .inner_join(relationship::table.on(relationship::parent_id.eq(entity::id)))
            .filter(relationship::child_id.eq(entity_id))
            .filter(relationship::kind.eq(relationship_kind))
            .select(entity::all_columns)
            .load::<Entity>(&mut self.conn)?;

        Ok(relations)
    }

    fn get_existing_entity_id(
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

    pub fn get_entity(&mut self, entity_id: Uuid) -> Result<Entity, failure::Error> {
        let entity = entity::table
            .filter(entity::id.eq(entity_id))
            .select(entity::all_columns)
            .first::<Entity>(&mut self.conn)
            .optional()?
            .ok_or_else(|| failure::format_err!("Entity not found with id: {}", entity_id))?;

        Ok(entity)
    }

    pub fn get_entities(&mut self, check_id: Uuid) -> Result<Vec<Entity>, failure::Error> {
        let entities = entity::table
            .inner_join(check_entity_map::table.on(check_entity_map::entity_id.eq(entity::id)))
            .filter(check_entity_map::check_id.eq(check_id))
            .select(entity::all_columns)
            .load::<Entity>(&mut self.conn)?;

        Ok(entities)
    }

    pub fn get_check(&mut self, check_id: Uuid) -> Result<Check, failure::Error> {
        let check = check::table
            .filter(check::id.eq(check_id))
            .first::<Check>(&mut self.conn)
            .optional()?
            .ok_or_else(|| failure::format_err!("Check not found with id: {}", check_id))?;

        Ok(check)
    }

    pub fn add_job(&mut self) -> Result<Uuid, failure::Error> {
        let id = Uuid::new_v4();
        insert_into(job::table)
            .values(Job {
                id,
                enqueued_at: Utc::now().naive_utc(),
                completed_at: None,
            }
            )
            .execute(&mut self.conn)?;

        Ok(id)
    }
}
