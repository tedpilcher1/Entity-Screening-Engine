use pulsar::{producer, DeserializeMessage, Error as PulsarError, SerializeMessage};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    company_house_apis::{get_company_officers, get_company_shareholders},
    postgres::Database,
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
    pub parent_company_id: String,
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

            let padded_company_house_number = format!("{:0>8}", shareholder_registration_number);

            let (country, postal_code) = match shareholder.address {
                Some(address) => (address.country, address.postal_code),
                None => (None, None),
            };

            let child_id = database
                .insert_company(
                    &padded_company_house_number,
                    shareholder.name,
                    shareholder.kind,
                    country,
                    postal_code,
                )
                .await?;
            database
                .insert_shareholder(self.parent_id, child_id)
                .await?;

            if self.get_officers {
                let job = Job::Officers(Officers {
                    company_id: child_id,
                    company_house_number: shareholder_registration_number.clone(),
                });

                producer.produce_message(job).await?;
            }

            if self.remaining_depth > 0 {
                let job = Job::RecursiveShareholders(RecursiveShareholders {
                    parent_id: child_id,
                    parent_company_id: shareholder_registration_number,
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
    pub company_id: Uuid,
    pub company_house_number: String,
}

impl Officers {
    // TODO: officers returned can be companies, hence shouldn't just assume individual
    pub async fn do_job(&self, database: &mut Database) -> Result<(), failure::Error> {
        let officers = get_company_officers(&self.company_house_number).await?;

        for officer in officers.items.unwrap_or_default() {
            let officer_identification = match officer.identification {
                Some(identification) => identification,
                None => return Ok(()),
            };

            let officer_company_house_number = match officer_identification.registration_number {
                Some(company_house_number) => company_house_number,
                None => return Ok(()),
            };

            let (country, postal_code) = match officer.address {
                Some(address) => (address.country, address.postal_code),
                None => (None, None),
            };

            let dob = Some("00/00/0000".to_string()); // TODO THIS PROPERLY

            let individual_id = database
                .insert_individual(
                    officer_company_house_number,
                    officer.name,
                    officer.nationality,
                    country,
                    postal_code,
                    dob,
                )
                .await?;
            database
                .insert_officer(self.company_id, individual_id, officer.officer_role)
                .await?;
        }

        Ok(())
    }
}
