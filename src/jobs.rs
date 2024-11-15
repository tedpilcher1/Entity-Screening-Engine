use pulsar::{producer, DeserializeMessage, Error as PulsarError, SerializeMessage};
use serde::{Deserialize, Serialize};
use sqlx::database;
use uuid::Uuid;

use crate::{
    company_house_apis::get_company_shareholders, postgres::Database, pulsar::PulsarProducer,
};

#[derive(Serialize, Deserialize, Debug)]
pub enum Job {
    RecursiveShareholders(RecursiveShareholders),
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
    id: Uuid,
    parent_company_id: Uuid,
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

        let shareholders_list = get_company_shareholders(&self.parent_company_id).await?;

        for shareholder in shareholders_list.items {
            let shareholder_company_id = shareholder
                .identification
                .unwrap()
                .registration_number
                .unwrap();
            database.insert_company(&shareholder_company_id).await?;

            let job = Job::RecursiveShareholders(RecursiveShareholders {
                id: Uuid::new_v4(),
                parent_company_id: self.parent_company_id,
            });

            producer.produce_message(job).await?;
        }
        Ok(())
    }
}
