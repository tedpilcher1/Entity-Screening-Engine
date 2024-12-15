use pulsar::SubType;

use crate::{
    jobs::jobs::{Job, JobKind},
    open_sanctions::api::OpenSanctionsClient,
    postgres::Database,
};

use super::worker::{Work, Worker};

pub const RISK_TOPIC: &str = "non-persistent://public/default/risk";
const SUBSCRIPTION: &str = "Risk-Sub";
const SUB_TYPE: SubType = SubType::Shared;

pub struct RiskWorker {
    pub database: Database,
    pub open_sanctions_client: OpenSanctionsClient,
}

impl RiskWorker {
    pub async fn new_worker() -> Result<Worker<RiskWorker>, failure::Error> {
        let risk_worker = RiskWorker {
            database: Database::connect()?,
            open_sanctions_client: OpenSanctionsClient::new(),
        };
        Ok(Worker::new(RISK_TOPIC, SUBSCRIPTION, SUB_TYPE, risk_worker).await?)
    }
}

impl Work for RiskWorker {
    async fn work(&mut self, job: Job) -> Result<(), failure::Error> {
        let job_result = match job.job_kind {
            JobKind::RiskJob(risk_job) => risk_job.do_job(self).await,
            _ => unimplemented!(),
        };

        self.database.complete_job(job.id)?;
        if let Err(_) = job_result {
            self.database.update_job_with_error(&job.id)?
        }
        job_result?;
        Ok(())
    }
}
