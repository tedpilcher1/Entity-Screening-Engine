use pulsar::SubType;

use crate::{
    jobs::jobs::{Job, JobKind},
    postgres::Database,
};

use super::{
    streaming_worker::{
        COMPANY_STREAMING_TOPIC, OFFICER_STREAMING_TOPIC, SHAREHOLDER_STREAMING_TOPIC,
    },
    worker::{Work, Worker},
};

const SUB: &str = "monitored-update-sub";
const SUB_TYPE: SubType = SubType::Shared;

pub struct MonitoredUpdateWorker {
    database: Database,
}

impl MonitoredUpdateWorker {
    pub async fn new_worker() -> Result<Worker<MonitoredUpdateWorker>, failure::Error> {
        let monitored_update_worker = Self {
            database: Database::connect()?,
        };
        Ok(Worker::new(
            vec![
                COMPANY_STREAMING_TOPIC,
                OFFICER_STREAMING_TOPIC,
                SHAREHOLDER_STREAMING_TOPIC,
            ],
            SUB,
            SUB_TYPE,
            monitored_update_worker,
        )
        .await?)
    }
}

impl Work for MonitoredUpdateWorker {
    async fn work(&mut self, job: Job) -> Result<(), failure::Error> {
        let job_result = match job.job_kind {
            JobKind::StreamingUpdateJob(update_job) => update_job.do_job(),
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
