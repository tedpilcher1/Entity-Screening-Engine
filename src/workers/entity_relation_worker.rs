use pulsar::SubType;

use crate::{
    company_house::company_house_apis::CompanyHouseClient,
    jobs::jobs::{Job, JobKind},
    postgres::Database,
    pulsar::{PulsarClient, PulsarProducer},
};

use super::{
    risk_worker::RISK_TOPIC,
    worker::{Work, Worker},
};

pub const ENTITY_RELATION_TOPIC: &str = "non-persistent://public/default/entity-relation";
const SUBSCRIPTION: &str = "Entity-Relation-Sub";
const RATE_LIMIT_PER_MIN: u32 = 120;
const MAX_JOB_PER_CHECK: usize = 2000;
const SUB_TYPE: SubType = SubType::Exclusive;

pub struct EntityRelationWorker {
    pub database: Database,
    pub company_house_client: CompanyHouseClient,
    pub entity_relation_producer: PulsarProducer,
    pub risk_producer: PulsarProducer,
}

impl EntityRelationWorker {
    pub async fn new_worker() -> Result<Worker<EntityRelationWorker>, failure::Error> {
        let pulsar_client = PulsarClient::new().await;

        let entity_relation_worker = Self {
            database: Database::connect()?,
            company_house_client: CompanyHouseClient::new(),
            entity_relation_producer: pulsar_client
                .create_producer(
                    ENTITY_RELATION_TOPIC,
                    Some(RATE_LIMIT_PER_MIN),
                    Some(MAX_JOB_PER_CHECK),
                )
                .await,
            risk_producer: pulsar_client.create_producer(RISK_TOPIC, None, None).await,
        };
        Ok(Worker::new(
            vec![ENTITY_RELATION_TOPIC],
            SUBSCRIPTION,
            SUB_TYPE,
            entity_relation_worker,
        )
        .await?)
    }
}

impl Work for EntityRelationWorker {
    async fn work(&mut self, job: Job) -> Result<(), failure::Error> {
        let job_result = match job.job_kind {
            JobKind::RelationJob(relation_job) => relation_job.do_work(self).await,
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
