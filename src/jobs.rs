use pulsar::{producer, DeserializeMessage, Error as PulsarError, SerializeMessage};
use serde::{Deserialize, Serialize};
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
    pub parent_id: Uuid,
    pub parent_company_id: String,
    pub remaining_depth: i32,
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

        // TODO: fix unwrap - but this should really never fail
        for shareholder in shareholders_list.items.unwrap_or_default() {
            let shareholder_identification = match shareholder.identification {
                Some(identification) => identification,
                None => return Ok(()), // graceful finish
            };

            let shareholder_registration_number =
                match shareholder_identification.registration_number {
                    Some(registration_numer) => registration_numer,
                    None => return Ok(()),
                };

            let (country, postal_code) = match shareholder.address {
                Some(address) => (address.country, address.postal_code),
                None => (None, None),
            };

            let child_id = database
                .insert_company(
                    &shareholder_registration_number,
                    shareholder.name,
                    shareholder.kind,
                    country,
                    postal_code,
                )
                .await?;
            database
                .insert_shareholder(self.parent_id, child_id)
                .await?;

            if self.remaining_depth > 0 {
                let job = Job::RecursiveShareholders(RecursiveShareholders {
                    parent_id: child_id,
                    parent_company_id: shareholder_registration_number,
                    remaining_depth: self.remaining_depth - 1,
                });

                producer.produce_message(job).await?;
            }
        }
        Ok(())
    }
}
