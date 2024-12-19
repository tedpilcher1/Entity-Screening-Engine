use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::company_house::company_house_streaming_types::{
    CompanyData, CompanyStreamingResponse, Event,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct StreamingUpdateJob {
    pub event: Event,
    pub kind: UpdateKind,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum UpdateKind {
    Company(CompanyData),
    Officer,
    Shareholder,
}

impl StreamingUpdateJob {
    pub fn do_job(&self) -> Result<(), failure::Error> {
        // first check if entity is monitored, if not stop processing job

        // add
        let entity_id = match &self.kind {
            UpdateKind::Company(company_streaming_response) => {
                self.handle_company_update(company_streaming_response)?
            }
            UpdateKind::Officer => unimplemented!(),
            UpdateKind::Shareholder => unimplemented!(),
        };

        // once update successfully handled, insert new snapshot row (timstamp, entity_id)

        Ok(())
    }

    fn handle_company_update(
        &self,
        company_streaming_response: &CompanyData,
    ) -> Result<Uuid, failure::Error> {
        // store in db information we care about

        // update latest timestamp
        // Need to be careful about this, could cause locking contention
        //  if multiple workers are all trying to update same row

        // produce message

        unimplemented!()
    }

    fn handle_officer_update(
        &self,
        company_streaming_response: &CompanyStreamingResponse,
    ) -> Result<(), failure::Error> {
        Ok(())
    }

    fn handle_shareholder_update(
        &self,
        company_streaming_response: &CompanyStreamingResponse,
    ) -> Result<(), failure::Error> {
        Ok(())
    }
}
