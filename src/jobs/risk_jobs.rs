use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::{Entity, Entitykind, FlagStringList},
    workers::risk_worker::RiskWorker,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct RiskJob {
    pub scope: RiskJobScope,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RiskJobScope {
    Global(GlobalRiskJob),
    Local(LocalRiskJob),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GlobalRiskJob {
    CircularRelations,
    ShellDetection,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LocalRiskJob {
    pub entity_id: Uuid,
    pub kind: LocalRiskJobKind,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum LocalRiskJobKind {
    Flags,
}

impl RiskJob {
    pub async fn do_job(&self, worker: &mut RiskWorker) -> Result<(), failure::Error> {
        match &self.scope {
            RiskJobScope::Global(global_risk_job) => self.do_global_job(global_risk_job, worker),
            RiskJobScope::Local(local_risk_job) => self.do_local_job(local_risk_job, worker).await,
        }
    }

    fn do_global_job(
        &self,
        job: &GlobalRiskJob,
        worker: &mut RiskWorker,
    ) -> Result<(), failure::Error> {
        todo!()
    }

    async fn do_local_job(
        &self,
        job: &LocalRiskJob,
        worker: &mut RiskWorker,
    ) -> Result<(), failure::Error> {
        let entity = worker.database.get_entity(job.entity_id)?;
        match job.kind {
            LocalRiskJobKind::Flags => self.do_flags_job(entity, worker).await,
        }
    }

    async fn do_flags_job(
        &self,
        entity: Entity,
        worker: &mut RiskWorker,
    ) -> Result<(), failure::Error> {
        if entity.kind != Entitykind::Individual {
            return Ok(());
        }

        if let Some(name) = entity.name {
            let search_result = worker.open_sanctions_client.get_flags(name).await?;

            if let Some(os_entity) = search_result.results.first() {
                for (key, value) in os_entity.properties.to_owned().into_iter() {
                    if key == "topics" {
                        worker
                            .database
                            .insert_flags(entity.id, FlagStringList(value).into())?
                    } else if key == "position" {
                        worker.database.insert_positions(entity.id, value)?
                    }
                }
                worker
                    .database
                    .insert_datasets(entity.id, os_entity.datasets.to_owned())?;
            }
        }
        Ok(())
    }
}
