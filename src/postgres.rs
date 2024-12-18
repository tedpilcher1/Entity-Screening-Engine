use chrono::{NaiveDate, NaiveDateTime, Utc};
use diesel::{insert_into, Connection, PgConnection};
use diesel::{prelude::*, update};
use uuid::Uuid;

use crate::models::{
    Check, CheckEntityMap, CheckJobMap, Checkkind, Dataset, Datasets, DormantCompany, Entity, Flag, Flagkind, Flags, Job, OutlierAge, Position, Positions, Relationship, Relationshipkind
};
use crate::schema::{
    check, check_entity_map, check_job_map, dataset, datasets, dormant_company, entity, flag,
    flags, job, outlier_age, position, positions, relationship,
};

pub struct Database {
    conn: PgConnection,
}

impl Database {
    pub fn connect() -> Result<Self, failure::Error> {
        let database_url = std::env::var("DATABASE_URL")?;
        let conn = PgConnection::establish(&database_url)?;
        Ok(Self { conn })
    }

    pub fn insert_check(&mut self, kind: Checkkind) -> Result<Uuid, failure::Error> {
        let id = Uuid::new_v4();

        insert_into(check::table)
            .values(&Check {
                id,
                started_at: Utc::now().naive_utc(),
                kind,
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
    ) -> Result<Vec<(Uuid, Option<NaiveDate>, Option<NaiveDate>)>, failure::Error> {
        let relations = entity::table
            .inner_join(relationship::table.on(relationship::parent_id.eq(entity::id)))
            .filter(relationship::child_id.eq(entity_id))
            .filter(relationship::kind.eq(relationship_kind))
            .select((
                relationship::parent_id,
                relationship::started_on,
                relationship::ended_on,
            ))
            .load::<(Uuid, Option<NaiveDate>, Option<NaiveDate>)>(&mut self.conn)?;

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

    pub fn add_job(&mut self, check_id: Uuid) -> Result<Uuid, failure::Error> {
        let id = Uuid::new_v4();

        self.conn.transaction(|conn| {
            insert_into(job::table)
                .values(Job {
                    id,
                    enqueued_at: Utc::now().naive_utc(),
                    completed_at: None,
                    has_error: false,
                })
                .execute(conn)?;

            insert_into(check_job_map::table)
                .values(CheckJobMap {
                    check_id,
                    job_id: id,
                })
                .execute(conn)?;

            diesel::result::QueryResult::Ok(())
        })?;

        Ok(id)
    }

    pub fn complete_job(&mut self, job_id: Uuid) -> Result<(), failure::Error> {
        update(job::table)
            .filter(job::id.eq(job_id))
            .set(job::completed_at.eq(Utc::now().naive_utc()))
            .execute(&mut self.conn)?;

        Ok(())
    }

    pub fn check_completed_at(
        &mut self,
        check_id: Uuid,
    ) -> Result<Option<NaiveDateTime>, failure::Error> {
        let incomplete_jobs = job::table
            .inner_join(check_job_map::table.on(check_job_map::job_id.eq(job::id)))
            .filter(check_job_map::check_id.eq(check_id))
            .filter(job::completed_at.is_null())
            .select(job::id)
            .load::<Uuid>(&mut self.conn)?;

        if incomplete_jobs.len() > 0 {
            return Ok(None);
        }

        let latest_completion = job::table
            .inner_join(check_job_map::table.on(check_job_map::job_id.eq(job::id)))
            .filter(check_job_map::check_id.eq(check_id))
            .select(diesel::dsl::max(job::completed_at))
            .first::<Option<NaiveDateTime>>(&mut self.conn)?;

        Ok(latest_completion)
    }

    pub fn get_checks(&mut self) -> Result<Vec<Check>, failure::Error> {
        Ok(check::table
            .select(check::all_columns)
            .load::<Check>(&mut self.conn)?)
    }

    pub fn get_root_entity(&mut self, check_id: &Uuid) -> Result<Entity, failure::Error> {
        Ok(entity::table
            .inner_join(check_entity_map::table.on(check_entity_map::entity_id.eq(entity::id)))
            .filter(check_entity_map::check_id.eq(check_id))
            .filter(entity::is_root.eq(true))
            .select(entity::all_columns)
            .first::<Entity>(&mut self.conn)?)
    }

    pub fn update_job_with_error(&mut self, job_id: &Uuid) -> Result<(), failure::Error> {
        update(job::table)
            .filter(job::id.eq(job_id))
            .set(job::has_error.eq(true))
            .execute(&mut self.conn)?;

        Ok(())
    }

    pub fn does_check_have_errored_job(&mut self, check_id: &Uuid) -> Result<bool, failure::Error> {
        let has_error = job::table
            .inner_join(check_job_map::table.on(check_job_map::job_id.eq(job::id)))
            .filter(check_job_map::check_id.eq(check_id))
            .select(job::has_error)
            .load::<bool>(&mut self.conn)?;

        Ok(has_error
            .into_iter()
            .find(|x| *x == true)
            .unwrap_or_else(|| false))
    }

    // TODO: this could be refactored into just jobs which could then be filtered by other fns
    pub fn get_num_of_jobs(&mut self, check_id: &Uuid) -> Result<usize, failure::Error> {
        let num_jobs = job::table
            .inner_join(check_job_map::table.on(check_job_map::job_id.eq(job::id)))
            .filter(check_job_map::check_id.eq(check_id))
            .select(job::id)
            .count()
            .first::<i64>(&mut self.conn)?;

        Ok(num_jobs as usize)
    }

    pub fn insert_positions(
        &mut self,
        entity_id: Uuid,
        titles: Vec<String>,
    ) -> Result<(), failure::Error> {
        self.conn.transaction(|conn| {
            for title in titles {
                let id = Uuid::new_v4();

                insert_into(position::table)
                    .values(Position { id, title })
                    .execute(conn)?;

                insert_into(positions::table)
                    .values(Positions {
                        entity_id,
                        position_id: id,
                    })
                    .execute(conn)?;
            }

            diesel::result::QueryResult::Ok(())
        })?;

        Ok(())
    }

    pub fn insert_flags(
        &mut self,
        entity_id: Uuid,
        flag_kinds: Vec<Flagkind>,
    ) -> Result<(), failure::Error> {
        self.conn.transaction(|conn| {
            for flag_kind in flag_kinds {
                let id = Uuid::new_v4();
                insert_into(flag::table)
                    .values(Flag {
                        id,
                        kind: flag_kind,
                    })
                    .execute(conn)?;

                insert_into(flags::table)
                    .values(Flags {
                        entity_id,
                        flag_id: id,
                    })
                    .execute(conn)?;
            }

            diesel::result::QueryResult::Ok(())
        })?;

        Ok(())
    }

    pub fn insert_datasets(
        &mut self,
        entity_id: Uuid,
        names: Vec<String>,
    ) -> Result<(), failure::Error> {
        self.conn.transaction(|conn| {
            for name in names {
                let id = Uuid::new_v4();
                insert_into(dataset::table)
                    .values(Dataset { id, name })
                    .execute(conn)?;

                insert_into(datasets::table)
                    .values(Datasets {
                        entity_id,
                        dataset_id: id,
                    })
                    .execute(conn)?;
            }

            diesel::result::QueryResult::Ok(())
        })?;

        Ok(())
    }

    pub fn get_flag_kinds_for_entity(
        &mut self,
        entity_id: &Uuid,
    ) -> Result<Vec<Flagkind>, failure::Error> {
        Ok(flag::table
            .inner_join(flags::table.on(flags::flag_id.eq(flag::id)))
            .filter(flags::entity_id.eq(entity_id))
            .select(flag::kind)
            .load::<Flagkind>(&mut self.conn)?)
    }

    pub fn get_flag_kinds_for_check(
        &mut self,
        check_id: &Uuid,
    ) -> Result<Vec<Flagkind>, failure::Error> {
        Ok(flag::table
            .inner_join(flags::table.on(flags::flag_id.eq(flag::id)))
            .inner_join(
                check_entity_map::table.on(check_entity_map::entity_id.eq(flags::entity_id)),
            )
            .filter(check_entity_map::check_id.eq(check_id))
            .select(flag::kind)
            .load::<Flagkind>(&mut self.conn)?)
    }

    pub fn get_positions(&mut self, entity_id: &Uuid) -> Result<Vec<String>, failure::Error> {
        Ok(position::table
            .inner_join(positions::table.on(positions::position_id.eq(position::id)))
            .filter(positions::entity_id.eq(entity_id))
            .select(position::title)
            .load::<String>(&mut self.conn)?)
    }

    pub fn get_datasets(&mut self, entity_id: &Uuid) -> Result<Vec<String>, failure::Error> {
        Ok(dataset::table
            .inner_join(datasets::table.on(datasets::dataset_id.eq(dataset::id)))
            .filter(datasets::entity_id.eq(entity_id))
            .select(dataset::name)
            .load::<String>(&mut self.conn)?)
    }

    pub fn insert_outlier_age(
        &mut self,
        entity_id: &Uuid,
        outlier: bool,
    ) -> Result<(), failure::Error> {
        insert_into(outlier_age::table)
            .values(OutlierAge {
                entity_id: *entity_id,
                outlier,
            })
            .execute(&mut self.conn)?;

        Ok(())
    }

    pub fn outlier_age(&mut self, entity_id: &Uuid) -> Result<bool, failure::Error> {
        let is_outlier = outlier_age::table
            .filter(outlier_age::entity_id.eq(*entity_id))
            .select(outlier_age::outlier)
            .first::<bool>(&mut self.conn)
            .optional()?;

        Ok(is_outlier.unwrap_or_default())
    }

    pub fn insert_dormant_company(
        &mut self,
        entity_id: &Uuid,
        dormant: bool,
    ) -> Result<(), failure::Error> {
        insert_into(dormant_company::table)
            .values(DormantCompany {
                entity_id: *entity_id,
                dormant,
            })
            .execute(&mut self.conn)?;

        Ok(())
    }

    pub fn company_dormant(&mut self, entity_id: &Uuid) -> Result<bool, failure::Error> {
        let is_dormant = dormant_company::table
            .filter(dormant_company::entity_id.eq(*entity_id))
            .select(dormant_company::dormant)
            .first::<bool>(&mut self.conn)
            .optional()?;

        Ok(is_dormant.unwrap_or_default())
    }
}
