use futures::TryStreamExt;
use uuid::Uuid;

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

    pub async fn do_work(&mut self) -> Result<(), failure::Error> {
        while let Some(msg) = self.consumer.internal_consumer.try_next().await? {
            let job = match msg.deserialize() {
                Ok(data) => data,
                Err(e) => {
                    // log::error!("could not deserialize message: {:?}", e);
                    todo!()
                }
            };

            let job_result = match job {
                Job::RecursiveShareholders(job) => {
                    job.do_job(&mut self.database, &mut self.producer).await
                }
            };

            match job_result {
                // log + metrics
                Ok(_) => {}
                Err(_) => {}
            }

            // acknowledge message once processing completed
            match self.consumer.internal_consumer.ack(&msg).await {
                Ok(_) => {}
                Err(_) => {} // TODO: log and move on, potentially retry x times then give up
            }
        }
        Ok(())
    }
}
