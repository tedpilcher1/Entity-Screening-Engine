use futures::TryStreamExt;

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
    pub async fn do_work(&mut self) -> Result<(), failure::Error> {
        while let Some(msg) = self.consumer.internal_consumer.try_next().await? {
            let job = match msg.deserialize() {
                Ok(data) => data,
                Err(e) => {
                    // TODO log error
                    println!("ERROR deseralizing");
                    todo!()
                }
            };

            println!("{:?}", job);

            let job_result = match job {
                Job::RecursiveShareholders(job) => {
                    job.do_job(&mut self.database, &mut self.producer).await
                }
                Job::Officers(job) => job.do_job(&mut self.database).await,
            };

            match job_result {
                // TODO: log + metrics
                Ok(_) => {}
                Err(e) => {println!("{:?}", e)}
            }

            match self.consumer.internal_consumer.ack(&msg).await {
                Ok(_) => {}
                Err(_) => {} // TODO: log and move on, potentially retry x times then give up
            }
        }
        Ok(())
    }
}
