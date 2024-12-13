use futures::TryStreamExt;
use log::{info, warn};

use crate::{
    company_house::company_house_apis::CompanyHouseClient,
    jobs::jobs::{Job, JobKind},
    postgres::Database,
    pulsar::{PulsarClient, PulsarConsumer, PulsarProducer},
};

pub struct EntityRelationWorker {
    database: Database,
    producer: PulsarProducer,
    consumer: PulsarConsumer,
    company_house_client: CompanyHouseClient,
}

impl EntityRelationWorker {
    pub async fn new() -> Result<Self, failure::Error> {
        let pulsar_client = PulsarClient::new().await;

        Ok(Self {
            database: Database::connect()?,
            producer: pulsar_client.create_producer().await,
            consumer: pulsar_client.create_consumer().await,
            company_house_client: CompanyHouseClient::new(),
        })
    }

    pub async fn run_job(&mut self, job: Job) -> Result<(), failure::Error> {
        let job_result = match job.job_kind {
            JobKind::RelationJob(relation_job) => {
                relation_job
                    .do_work(
                        &mut self.database,
                        &mut self.producer,
                        &self.company_house_client,
                    )
                    .await
            }
        };

        // record job as completed and then throw error
        self.database.complete_job(job.id)?;
        if let Err(_) = job_result {
            self.database.update_job_with_error(&job.id)?
        }
        job_result?;
        Ok(())
    }

    // TODO: negative ack
    pub async fn do_work(&mut self) {
        // TODO: remove .expect(...) below
        while let Some(msg) = self
            .consumer
            .internal_consumer
            .try_next()
            .await
            .expect("Should be able to wait for new message.")
        {
            let job = match msg.deserialize() {
                Ok(data) => data,
                Err(e) => {
                    warn!("Couldn't deseralize job, error: {:?}.", e);
                    continue;
                }
            };

            let job_id = job.id;
            match self.run_job(job).await {
                // TODO: log + metrics
                Ok(_) => {
                    self.consumer.ack(&msg).await;
                    println!("Job completed successfully, id: {:?}", job_id);
                    info!("Job completed successfully, id: {:?}", job_id)
                }
                Err(e) => {            
                    // self.consumer.nack(&msg).await;
                    println!("Job had an error, id: {:?}, error:{:?}", job_id, e);
                    warn!("Job had an error, id: {:?}, error: {:?}", job_id, e)
                }
            }

            match self.consumer.internal_consumer.ack(&msg).await {
                Ok(_) => {}
                Err(e) => {
                    warn!("Couldn't acknowledge message, error: {:?}", e)
                }
            }
        }
    }
}
