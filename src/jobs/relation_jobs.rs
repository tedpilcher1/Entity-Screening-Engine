use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::company_house::company_house_apis::{get_appointments, get_officers, get_shareholders};
// use crate::company_house_apis::{get_officers, get_shareholders};
use crate::jobs::jobs::JobKind;
use crate::models::{Entity, EntityRelation, Entitykind, Relationship, Relationshipkind};
use crate::postgres::Database;
use crate::pulsar::PulsarProducer;

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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum RelationJobKind {
    Shareholders,
    Officers,
    Appointments,
}

impl RelationJob {
    pub async fn do_work(
        &self,
        database: &mut Database,
        producer: &mut PulsarProducer,
    ) -> Result<(), failure::Error> {
        let entities: Vec<EntityRelation> = match self.relation_job_kind {
            RelationJobKind::Shareholders => {
                get_shareholders(&self.company_house_number).await?.into()
            }
            RelationJobKind::Officers => get_officers(&self.company_house_number).await?.into(),
            RelationJobKind::Appointments => {
                get_appointments(&self.company_house_number).await?.into()
            }
        };

        let relationship_kind = match self.relation_job_kind {
            RelationJobKind::Shareholders => Relationshipkind::Shareholder,
            RelationJobKind::Officers => Relationshipkind::Officer,
            RelationJobKind::Appointments => Relationshipkind::Officer, // not sure if this will work?
        };

        self.do_job(
            entities,
            relationship_kind,
            database,
            producer,
            self.relation_job_kind == RelationJobKind::Appointments,
        )
        .await
    }

    async fn do_job(
        &self,
        entity_relations: Vec<EntityRelation>,
        relationship_kind: Relationshipkind,
        database: &mut Database,
        producer: &mut PulsarProducer,
        reverse_relation: bool,
    ) -> Result<(), failure::Error> {
        for entity_relation in entity_relations {
            let parent_id = database.insert_entity(&entity_relation.entity, self.check_id)?;
            match database.insert_relationship(Relationship {
                parent_id,
                child_id: self.child_id,
                kind: relationship_kind,
                started_on: entity_relation.started_on,
                ended_on: entity_relation.ended_on,
            }) {
                Ok(_) => {
                    self.queue_further_jobs(database, producer, &entity_relation.entity)
                        .await?
                }
                // log error and continue
                Err(e) => println!(
                    "Inserting relation failed for {:?}, error: {:?}",
                    relationship_kind, e
                ),
            }
            self.queue_further_jobs(database, producer, &entity_relation.entity)
                .await?;
        }

        Ok(())
    }

    async fn queue_further_jobs(
        &self,
        database: &mut Database,
        producer: &mut PulsarProducer,
        entity: &Entity,
    ) -> Result<(), failure::Error> {
        match entity.kind {
            Entitykind::Company => {
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

                    producer
                        .enqueue_job(database, self.check_id, job_kind)
                        .await?;
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

                    producer
                        .enqueue_job(database, self.check_id, job_kind)
                        .await?;
                }
            }
            Entitykind::Individual => {
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

                    producer
                        .enqueue_job(database, self.check_id, job_kind)
                        .await?;
                }
            }
        }

        Ok(())
    }
}
