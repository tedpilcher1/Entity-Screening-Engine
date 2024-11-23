use pulsar::{producer, DeserializeMessage, Error as PulsarError, SerializeMessage};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    company_house_apis::{get_company_officers, get_company_shareholders},
    model::RelationshipKind,
    postgres::Database,
    postgres_types::{Entity, Relationship},
    pulsar::PulsarProducer,
};

#[derive(Serialize, Deserialize, Debug)]
pub enum Job {
    RecursiveShareholders(RecursiveShareholders),
    Officers(Officers),
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
pub struct RecursiveShareholders {
    pub parent_id: Uuid,
    pub check_id: Uuid,
    pub parent_company_number: String,
    pub remaining_depth: i32,
    pub get_officers: bool,
}

impl RecursiveShareholders {
    pub async fn do_job(
        &self,
        database: &mut Database,
        producer: &mut PulsarProducer,
    ) -> Result<(), failure::Error> {
        // get shareholders for parent_company_id
        // for each shareholder
        // -- store in db
        // -- produce message

        let shareholders_list = get_company_shareholders(&self.parent_company_number).await?;
        for shareholder in shareholders_list.items.unwrap_or_default() {
            let entity: Result<Entity, ()> = (shareholder, false).try_into();
            let entity = match entity {
                Ok(entity) => entity,
                Err(_) => return Ok(()),
            };

            let child_id = database.insert_entity(&entity, self.check_id)?;
            database.insert_relationship(Relationship {
                parent_id: entity.id,
                child_id,
                kind: RelationshipKind::Shareholder,
            })?;

            if self.get_officers {
                let job = Job::Officers(Officers {
                    entity_id: child_id,
                    check_id: self.check_id,
                    company_house_number: entity.company_house_number.clone(),
                });

                producer.produce_message(job).await?;
            }

            if self.remaining_depth > 0 {
                let job = Job::RecursiveShareholders(RecursiveShareholders {
                    parent_id: child_id,
                    check_id: self.check_id,
                    parent_company_number: entity.company_house_number,
                    remaining_depth: self.remaining_depth - 1,
                    get_officers: self.get_officers,
                });

                producer.produce_message(job).await?;
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Officers {
    pub entity_id: Uuid,
    pub check_id: Uuid,
    pub company_house_number: String,
}

impl Officers {
    // TODO: officers returned can be companies, hence shouldn't just assume individual
    pub async fn do_job(&self, database: &mut Database) -> Result<(), failure::Error> {
        let officers = get_company_officers(&self.company_house_number).await?;

        for officer in officers.items.unwrap_or_default() {
            let entity: Result<Entity, ()> = (officer, false).try_into();
            let entity = match entity {
                Ok(entity) => entity,
                Err(_) => return Ok(()),
            };

            let child_id = database.insert_entity(&entity, self.check_id)?;
            database.insert_relationship(Relationship {
                parent_id: entity.id,
                child_id,
                kind: RelationshipKind::Officer,
            })?;
        }

        Ok(())
    }
}
