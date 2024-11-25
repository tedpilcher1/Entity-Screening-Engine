use pulsar::{producer, DeserializeMessage, Error as PulsarError, SerializeMessage};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{Entity, Relationship};
use crate::{
    company_house_apis::{get_officers, get_shareholders},
    models::Relationshipkind,
    postgres::Database,
    pulsar::PulsarProducer,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Job {
    pub id: Uuid,
    pub job_kind: JobKind,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum JobKind {
    Shareholders(Shareholders),
    Officers(Officers),
    // Companies(Companies),
}

impl SerializeMessage for Job {
    fn serialize_message(input: Self) -> Result<producer::Message, PulsarError> {
        let payload = serde_json::to_vec(&input).map_err(|e| PulsarError::Custom(e.to_string()))?;
        Ok(producer::Message {
            payload,
            ..Default::default()
        })
    }
}

impl DeserializeMessage for Job {
    type Output = Result<Job, serde_json::Error>;

    fn deserialize_message(payload: &pulsar::Payload) -> Self::Output {
        serde_json::from_slice(&payload.data)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Shareholders {
    pub parent_id: Uuid,
    pub check_id: Uuid,
    pub parent_company_number: String,
    pub remaining_shareholder_depth: usize,
    pub remaining_officers_depth: usize,
}

impl Shareholders {
    pub async fn do_job(
        &self,
        database: &mut Database,
        producer: &mut PulsarProducer,
    ) -> Result<(), failure::Error> {
        let shareholders_list = get_shareholders(&self.parent_company_number).await?;
        for shareholder in shareholders_list.items.unwrap_or_default() {
            let entity: Result<Entity, ()> = (shareholder, false).try_into();
            let entity = match entity {
                Ok(entity) => entity,
                Err(_) => return Ok(()),
            };

            let parent_id = database.insert_entity(&entity, self.check_id)?;
            database.insert_relationship(Relationship {
                parent_id,
                child_id: entity.id,
                kind: Relationshipkind::Shareholder,
            })?;

            queue_relation_jobs(
                producer,
                database,
                self.remaining_officers_depth,
                self.remaining_shareholder_depth - 1,
                parent_id,
                self.check_id,
                entity.company_house_number,
            )
            .await?;
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Officers {
    pub entity_id: Uuid, // todo rename to child_id
    pub check_id: Uuid,
    pub company_house_number: String,
    pub remaining_shareholder_depth: usize,
    pub remaining_officers_depth: usize,
}

impl Officers {
    pub async fn do_job(
        &self,
        database: &mut Database,
        producer: &mut PulsarProducer,
    ) -> Result<(), failure::Error> {
        let officers = get_officers(&self.company_house_number).await?;

        for officer in officers.items.unwrap_or_default() {
            let entity: Result<Entity, ()> = (officer, false).try_into();
            let entity = match entity {
                Ok(entity) => entity,
                Err(_) => return Ok(()),
            };

            let parent_id = database.insert_entity(&entity, self.check_id)?;
            database.insert_relationship(Relationship {
                parent_id,
                child_id: self.entity_id,
                kind: Relationshipkind::Officer,
            })?;

            queue_relation_jobs(
                producer,
                database,
                self.remaining_officers_depth - 1,
                self.remaining_shareholder_depth,
                parent_id,
                self.check_id,
                entity.company_house_number,
            )
            .await?;
        }

        Ok(())
    }
}

async fn queue_relation_jobs(
    producer: &mut PulsarProducer,
    database: &mut Database,
    remaining_officers_depth: usize,
    remaining_shareholder_depth: usize,
    entity_id: Uuid,
    check_id: Uuid,
    company_house_number: String,
) -> Result<(), failure::Error> {
    if remaining_officers_depth > 0 {
        let job_kind = JobKind::Officers(Officers {
            entity_id,
            check_id,
            company_house_number: company_house_number.clone(),
            remaining_officers_depth,
            remaining_shareholder_depth,
        });

        producer.enqueue_job(database, check_id, job_kind).await?;
    }

    if remaining_shareholder_depth > 0 {
        let job_kind = JobKind::Shareholders(Shareholders {
            parent_id: entity_id,
            check_id,
            parent_company_number: company_house_number,
            remaining_officers_depth,
            remaining_shareholder_depth,
        });

        producer.enqueue_job(database, check_id, job_kind).await?;
    }
    Ok(())
}

// #[derive(Serialize, Deserialize, Debug)]
// pub struct Companies {
//     pub compan_name: String,
//     pub check_id: Uuid,
// }

// impl Companies {
//     pub async fn do_job(&self, database: &mut Database) -> Result<(), failure::Error> {

//         // using company house api, get potential companies
//         let companies = get_company(self.compan_name).await?;

//         // store in db

//         // TODO: do some further processing to understand what is most likely match
//         // could use additional data, e.g. country, postal code, individual or company etc,
//     }
// }
