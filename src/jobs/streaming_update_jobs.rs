use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    company_house::company_house_streaming_types::{CompanyData, Event},
    models::Entity,
    workers::monitored_update_worker::MonitoredUpdateWorker,
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
    pub fn do_job(self, worker: &mut MonitoredUpdateWorker) -> Result<(), failure::Error> {
        let check_ids = self.get_check_ids_monitoring_entity(worker)?;

        if check_ids.len() > 0 {
            let entity: Entity = match self.kind {
                UpdateKind::Company(company_data) => company_data.into(),
                UpdateKind::Officer => todo!(),
                UpdateKind::Shareholder => todo!(),
            };

            worker.database.insert_entity_snapshot(&entity, check_ids)?;
        }

        worker
            .database
            .insert_processed_update(self.event.timepoint)?;

        // TODO: create message on notification topic

        Ok(())
    }

    fn get_check_ids_monitoring_entity(
        &self,
        worker: &mut MonitoredUpdateWorker,
    ) -> Result<Vec<Uuid>, failure::Error> {
        let company_house_id = match &self.kind {
            UpdateKind::Company(company_data) => &company_data.company_number,
            UpdateKind::Officer => todo!(),
            UpdateKind::Shareholder => todo!(),
        };
        worker
            .database
            .get_monitored_entity_check_ids(company_house_id)
    }
}
