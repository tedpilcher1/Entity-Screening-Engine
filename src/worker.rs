use futures::TryStreamExt;
use log::{info, warn};

use crate::{
    jobs::Job,
    postgres::Database,
    pulsar::{PulsarClient, PulsarConsumer, PulsarProducer},
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
            database: Database::connect().await?,
            producer: pulsar_client.create_producer().await,
            consumer: pulsar_client.create_consumer().await,
        })
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

            let job_result = match job {
                Job::RecursiveShareholders(job) => {
                    job.do_job(&mut self.database, &mut self.producer).await
                }
                Job::Officers(job) => job.do_job(&mut self.database).await,
            };

            match job_result {
                // TODO: log + metrics
                Ok(_) => {
                    info!("Job completed successfully")
                }
                Err(e) => {
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
