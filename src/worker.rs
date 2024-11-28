use futures::TryStreamExt;
use log::{info, warn};

use crate::{
    jobs::jobs::{Job, JobKind}, postgres::Database, pulsar::{PulsarClient, PulsarConsumer, PulsarProducer}
};

pub struct Worker {
    database: Database,
    producer: PulsarProducer,
    consumer: PulsarConsumer,
}

impl Worker {
    pub async fn new() -> Result<Self, failure::Error> {
        let pulsar_client = PulsarClient::new().await;

        Ok(Self {
            database: Database::connect()?,
            producer: pulsar_client.create_producer().await,
            consumer: pulsar_client.create_consumer().await,
        })
    }

    pub async fn run_job(&mut self, job: Job) -> Result<(), failure::Error> {
        match job.job_kind {
            JobKind::RelationJob(relation_job) => relation_job.do_work(&mut self.database, &mut self.producer).await?,
        };
        self.database.complete_job(job.id)?;
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

            match self.run_job(job).await {
                // TODO: log + metrics
                Ok(_) => {
                    info!("Job completed successfully")
                }
                Err(e) => {
                    println!("Job had and error, error: {:?}", e);
                    warn!("Job had and error, error: {:?}", e)
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
