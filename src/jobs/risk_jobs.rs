use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{models::Entity, workers::{risk_worker::RiskWorker, worker::Worker}};

#[derive(Serialize, Deserialize, Debug)]
pub struct RiskJob{
    scope: RiskJobScope,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RiskJobScope {
    Global(GlobalRiskJob),
    Local(LocalRiskJob),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GlobalRiskJob {
    CircularOwnership,
    ShellDetection,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LocalRiskJob {
    entity_id: Uuid,
    kind: LocalRiskJobKind,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum LocalRiskJobKind{
    Sanctions,
    CriminalWatchlists,
    Peps,
}

impl RiskJob {
    pub fn do_job(&self, worker: &mut Worker<RiskWorker>) -> Result<(), failure::Error> {
        match &self.scope {
            RiskJobScope::Global(global_risk_job) => self.do_global_job(global_risk_job, worker),
            RiskJobScope::Local(local_risk_job) => self.do_local_job(local_risk_job, worker),
        }
    }

    fn do_global_job(&self, job: &GlobalRiskJob, worker: &mut Worker<RiskWorker>) -> Result<(), failure::Error> {
        todo!()
    }

    fn do_local_job(&self, job: &LocalRiskJob, worker: &mut Worker<RiskWorker>) -> Result<(), failure::Error> {
        
        let entity = worker.database.get_entity(job.entity_id)?;        
        match job.kind {
            LocalRiskJobKind::Sanctions => self.do_sanctions_job(entity),
            LocalRiskJobKind::CriminalWatchlists => self.do_criminal_watchlists_job(entity),
            LocalRiskJobKind::Peps => self.do_peps_job(entity),
        }
    }

    fn do_sanctions_job(&self, entity: Entity) -> Result<(), failure::Error>{
        todo!()
    }

    fn do_criminal_watchlists_job(&self, entity: Entity) -> Result<(), failure::Error>{
        todo!()
    }

    fn do_peps_job(&self, entity: Entity) -> Result<(), failure::Error>{
        todo!()
    }
}