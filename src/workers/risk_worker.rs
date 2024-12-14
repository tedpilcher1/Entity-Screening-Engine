use pulsar::SubType;

use crate::{jobs::jobs::Job, postgres::Database, pulsar::PulsarProducer};

use super::worker::{Work, Worker};

pub const RISK_TOPIC: &str = "non-persistent://public/default/risk";
const SUBSCRIPTION: &str = "Risk-Sub";
const SUB_TYPE: SubType = SubType::Shared;

pub struct RiskWorker {}

impl RiskWorker {
    pub async fn new_worker() -> Result<Worker<RiskWorker>, failure::Error> {
        let risk_worker = RiskWorker {};
        Ok(Worker::new(RISK_TOPIC, SUBSCRIPTION, None, None, SUB_TYPE, risk_worker).await?)
    }
}

impl Work for RiskWorker {
    async fn run_job(
        &mut self,
        job: Job,
        database: &mut Database,
        producer: &mut PulsarProducer,
    ) -> Result<(), failure::Error> {
        // let job_result = match job.job_kind {
        //     _ => unimplemented!(),
        // };

        // database.complete_job(job.id)?;
        // if let Err(_) = job_result {
        //     database.update_job_with_error(&job.id)?
        // }
        // job_result?;
        Ok(())
    }
}
