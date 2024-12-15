use log::warn;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::jobs::jobs::JobKind;
use crate::models::{Entity, EntityRelation, Entitykind, Relationship, Relationshipkind};
use crate::workers::entity_relation_worker::EntityRelationWorker;

use super::risk_jobs::{LocalRiskJob, RiskJob, RiskJobScope};

#[derive(Serialize, Deserialize, Debug)]
pub struct RelationJob {
    pub child_id: Uuid,
    pub check_id: Uuid,
    pub company_house_number: String,
    pub officer_id: Option<String>,
    pub remaining_depth: usize,
    pub relation_job_kind: RelationJobKind,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum RelationJobKind {
    Shareholders,
    Officers,
    Appointments,
}

impl RelationJob {
    pub async fn do_work(&self, worker: &mut EntityRelationWorker) -> Result<(), failure::Error> {
        let entities: Vec<EntityRelation> = match self.relation_job_kind {
            RelationJobKind::Shareholders => worker
                .company_house_client
                .get_shareholders(&self.company_house_number)
                .await?
                .into(),
            RelationJobKind::Officers => worker
                .company_house_client
                .get_officers(&self.company_house_number)
                .await?
                .into(),
            RelationJobKind::Appointments => worker
                .company_house_client
                .get_appointments(&self.officer_id)
                .await?
                .into(),
        };

        let relationship_kind = match self.relation_job_kind {
            RelationJobKind::Shareholders => Relationshipkind::Shareholder,
            RelationJobKind::Officers => Relationshipkind::Officer,
            RelationJobKind::Appointments => Relationshipkind::Officer,
        };

        self.do_job(
            entities,
            relationship_kind,
            self.relation_job_kind == RelationJobKind::Appointments,
            worker,
        )
        .await
    }

    async fn do_job(
        &self,
        entity_relations: Vec<EntityRelation>,
        relationship_kind: Relationshipkind,
        reverse_relation: bool,
        worker: &mut EntityRelationWorker,
    ) -> Result<(), failure::Error> {
        for entity_relation in entity_relations {
            let parent_id = worker
                .database
                .insert_entity(&entity_relation.entity, self.check_id)?;

            let insert_relationship_result = match reverse_relation {
                true => worker.database.insert_relationship(Relationship {
                    parent_id: self.child_id,
                    child_id: parent_id,
                    kind: relationship_kind,
                    started_on: entity_relation.started_on,
                    ended_on: entity_relation.ended_on,
                }),
                false => worker.database.insert_relationship(Relationship {
                    parent_id,
                    child_id: self.child_id,
                    kind: relationship_kind,
                    started_on: entity_relation.started_on,
                    ended_on: entity_relation.ended_on,
                }),
            };

            match insert_relationship_result {
                Ok(_) => {
                    self.queue_further_jobs(&entity_relation.entity, worker)
                        .await?
                }
                // log error and continue
                Err(e) => warn!(
                    "Inserting relation failed for {:?}, error: {:?}",
                    relationship_kind, e
                ),
            }
            self.queue_further_jobs(&entity_relation.entity, worker)
                .await?;
        }

        Ok(())
    }

    async fn queue_further_jobs(
        &self,
        entity: &Entity,
        worker: &mut EntityRelationWorker,
    ) -> Result<(), failure::Error> {
        match entity.kind {
            Entitykind::Company => {
                if self.remaining_depth > 0 {
                    let job_kind = JobKind::RelationJob(RelationJob {
                        child_id: entity.id,
                        check_id: self.check_id,
                        company_house_number: entity.company_house_number.clone(),
                        officer_id: entity.officer_id.clone(),
                        remaining_depth: self.remaining_depth - 1,
                        relation_job_kind: RelationJobKind::Shareholders,
                    });

                    worker
                        .entity_relation_producer
                        .enqueue_job(&mut worker.database, self.check_id, job_kind)
                        .await?;
                }
                if self.remaining_depth > 0 {
                    let job_kind = JobKind::RelationJob(RelationJob {
                        child_id: entity.id,
                        check_id: self.check_id,
                        company_house_number: entity.company_house_number.clone(),
                        officer_id: entity.officer_id.clone(),
                        remaining_depth: self.remaining_depth - 1,
                        relation_job_kind: RelationJobKind::Officers,
                    });

                    worker
                        .entity_relation_producer
                        .enqueue_job(&mut worker.database, self.check_id, job_kind)
                        .await?;
                }
            }
            Entitykind::Individual => {
                if self.remaining_depth > 0 {
                    let appointment_job = JobKind::RelationJob(RelationJob {
                        child_id: entity.id,
                        check_id: self.check_id,
                        company_house_number: entity.company_house_number.clone(),
                        officer_id: entity.officer_id.clone(),
                        remaining_depth: self.remaining_depth - 1,
                        relation_job_kind: RelationJobKind::Appointments,
                    });

                    worker
                        .entity_relation_producer
                        .enqueue_job(&mut worker.database, self.check_id, appointment_job)
                        .await?;
                }

                let flag_job = JobKind::RiskJob(RiskJob {
                    scope: RiskJobScope::Local(LocalRiskJob {
                        entity_id: entity.id,
                        kind: super::risk_jobs::LocalRiskJobKind::Flags,
                    }),
                });

                worker
                    .risk_producer
                    .enqueue_job(&mut worker.database, self.check_id, flag_job)
                    .await?;
            }
        }

        Ok(())
    }
}
