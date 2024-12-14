use pulsar::SubType;

use crate::{
    company_house::company_house_apis::CompanyHouseClient,
    jobs::jobs::{Job, JobKind},
    postgres::Database,
    pulsar::PulsarProducer,
};

use super::worker::{Work, Worker};

pub const ENTITY_RELATION_TOPIC: &str = "non-persistent://public/default/entity-relation";
const SUBSCRIPTION: &str = "Entity-Relation-Sub";
const RATE_LIMIT_PER_MIN: u32 = 120;
const MAX_JOB_PER_CHECK: usize = 2000;
const SUB_TYPE: SubType = SubType::Exclusive;

pub struct EntityRelationWorker {
    company_house_client: CompanyHouseClient,
}

impl EntityRelationWorker {
    pub async fn new_worker() -> Result<Worker<EntityRelationWorker>, failure::Error> {
        let entity_relation_worker = Self {
            company_house_client: CompanyHouseClient::new(),
        };
        Ok(Worker::new(
            ENTITY_RELATION_TOPIC,
            SUBSCRIPTION,
            Some(RATE_LIMIT_PER_MIN),
            Some(MAX_JOB_PER_CHECK),
            SUB_TYPE,
            entity_relation_worker,
        )
        .await?)
    }
}

impl Work for EntityRelationWorker {
    async fn run_job(
        &mut self,
        job: Job,
        database: &mut Database,
        producer: &mut PulsarProducer,
    ) -> Result<(), failure::Error> {
        let job_result = match job.job_kind {
            JobKind::RelationJob(relation_job) => {
                relation_job
                    .do_work(database, producer, &self.company_house_client)
                    .await
            }
            _ => unimplemented!(),
        };

        database.complete_job(job.id)?;
        if let Err(_) = job_result {
            database.update_job_with_error(&job.id)?
        }
        job_result?;
        Ok(())
    }
}
