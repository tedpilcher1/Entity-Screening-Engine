use futures::TryStreamExt;
use uuid::Uuid;

use crate::{jobs::{Job, TestJob}, postgres::Database, pulsar::{PulsarClient, PulsarConsumer, PulsarProducer}};

pub struct Worker {
    id: Uuid,
    database: Database,
    producer: PulsarProducer,
    consumer: PulsarConsumer,
}

impl Worker {
    pub async fn new() -> Result<Self, failure::Error> {

        let pulsar_client = PulsarClient::new().await;

        Ok(Self {
            id: Uuid::new_v4(),
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
               Job::TestJob(test_job) => test_job.do_job(),
            };

            match job_result { // log + metrics
                Ok(_) => todo!(), 
                Err(_) => todo!(), 
            }
        }

        Ok(())
    }
}