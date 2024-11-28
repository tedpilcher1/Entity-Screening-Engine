use log::warn;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::pulsar::PulsarProducer;
use crate::postgres::Database;
use crate::company_house_apis::{get_shareholders, get_officers};
use crate::models::{Entity, Relationship, Relationshipkind};
use crate::jobs::jobs::JobKind;

#[derive(Serialize, Deserialize, Debug)]
pub struct RelationJob {
    pub child_id: Uuid,
    pub check_id: Uuid,
    pub company_house_number: String,
    pub remaining_shareholder_depth: usize,
    pub remaining_officer_depth: usize,
    pub remaining_appointment_depth: usize,
    pub relation_job_kind: RelationJobKind,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RelationJobKind {
    Shareholders,
    Officers,
    Appointments, // TODO
}

impl RelationJob {

    pub async fn do_work (&self, database: &mut Database, producer: &mut PulsarProducer) -> Result<(), failure::Error> {
        match self.relation_job_kind {
            RelationJobKind::Shareholders => self.shareholder_job(database, producer).await,
            RelationJobKind::Officers => self.officer_job(database, producer).await,
            RelationJobKind::Appointments => self.appointment_job().await,
        }
    }

    async fn shareholder_job(&self, database: &mut Database, producer: &mut PulsarProducer) -> Result<(), failure::Error> {
        let shareholders_list = get_shareholders(&self.company_house_number).await?;
        for shareholder in shareholders_list.items.unwrap_or_default() {
            let entity: Result<Entity, ()> = (shareholder, false).try_into();
            let entity = match entity {
                Ok(entity) => entity,
                Err(_) => {
                    warn!("Failed to convert to entity."); // todo improve log
                    continue;
                }
            };

            let parent_id = database.insert_entity(&entity, self.check_id)?;
            match database.insert_relationship(Relationship {
                parent_id,
                child_id: self.child_id,
                kind: Relationshipkind::Shareholder,
            }) {
                Ok(_) => self.queue_further_jobs(database, producer, &entity).await?,
                // log error and continue
                Err(e) => println!("Inserting relation failed for shareholder, error: {:?}", e),
            }
        }
        Ok(())
    }

    async fn officer_job(&self, database: &mut Database, producer: &mut PulsarProducer) -> Result<(), failure::Error> {

        let officers = get_officers(&self.company_house_number).await?;

        for officer in officers.items.unwrap_or_default() {
            let entity: Result<Entity, ()> = (officer, false).try_into();
            let entity = match entity {
                Ok(entity) => entity,
                Err(_) => {
                    warn!("Failed to convert to entity."); // todo improve log
                    continue;
                },
            };

            let parent_id = database.insert_entity(&entity, self.check_id)?;
            match database.insert_relationship(Relationship {
                parent_id,
                child_id: self.child_id,
                kind: Relationshipkind::Officer,
            }) {
                Ok(_) => self.queue_further_jobs(database, producer, &entity).await?,
                // log error and continue
                Err(e) => println!("Inserting relation failed for officer, error: {:?}", e),
            }
        }

        Ok(())
    }

    async fn appointment_job(&self) -> Result<(), failure::Error> {
        // TODO
        Ok(())
    }

    async fn queue_further_jobs(&self, database: &mut Database, producer: &mut PulsarProducer, entity: &Entity) -> Result<(), failure::Error> {
    
        if self.remaining_officer_depth > 0 {
            let job_kind = JobKind::RelationJob(RelationJob {
                child_id: entity.id,
                check_id: self.check_id,
                company_house_number: entity.company_house_number.clone(),
                remaining_shareholder_depth: self.remaining_shareholder_depth,
                remaining_officer_depth: self.remaining_officer_depth - 1,
                remaining_appointment_depth: self.remaining_appointment_depth,
                relation_job_kind: RelationJobKind::Shareholders,
            });

            producer.enqueue_job(database, self.check_id, job_kind).await?;
        }
        if self.remaining_shareholder_depth > 0 {
            let job_kind = JobKind::RelationJob(RelationJob {
                child_id: entity.id,
                check_id: self.check_id,
                company_house_number: entity.company_house_number.clone(),
                remaining_shareholder_depth: self.remaining_shareholder_depth - 1,
                remaining_officer_depth: self.remaining_officer_depth,
                remaining_appointment_depth: self.remaining_appointment_depth,
                relation_job_kind: RelationJobKind::Officers,
            });

            producer.enqueue_job(database, self.check_id, job_kind).await?;
        }
        if self.remaining_appointment_depth > 0 {
            let job_kind = JobKind::RelationJob(RelationJob {
                child_id: entity.id,
                check_id: self.check_id,
                company_house_number: entity.company_house_number.clone(),
                remaining_shareholder_depth: self.remaining_shareholder_depth,
                remaining_officer_depth: self.remaining_officer_depth,
                remaining_appointment_depth: self.remaining_appointment_depth - 1,
                relation_job_kind: RelationJobKind::Appointments,
            });

            producer.enqueue_job(database, self.check_id, job_kind).await?;
        }

        Ok(())
    }
}