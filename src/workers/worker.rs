use futures::TryStreamExt;
use log::{info, warn};
use pulsar::SubType;

use crate::{
    jobs::jobs::Job,
    postgres::Database,
    pulsar::{PulsarClient, PulsarConsumer, PulsarProducer},
};

pub trait Work {
    fn run_job(
        &mut self,
        job: Job,
        database: &mut Database,
        producer: &mut PulsarProducer,
    ) -> impl std::future::Future<Output = Result<(), failure::Error>> + Send;
}

pub struct Worker<T: Work> {
    pub database: Database,
    pub producer: PulsarProducer,
    pub consumer: PulsarConsumer,
    pub internal_worker: T,
}

impl<T: Work> Worker<T> {
    pub async fn new(
        topic: &str,
        sub: &str,
        rate_limit_per_min: Option<u32>,
        max_jobs_per_check: Option<usize>,
        sub_type: SubType,
        internal_worker: T,
    ) -> Result<Self, failure::Error> {
        let pulsar_client = PulsarClient::new().await;

        Ok(Self {
            database: Database::connect()?,
            producer: pulsar_client
                .create_producer(topic, rate_limit_per_min, max_jobs_per_check)
                .await,
            consumer: pulsar_client.create_consumer(topic, sub_type, sub).await,
            internal_worker,
        })
    }
}

impl<T: Work> Worker<T> {
    pub async fn do_work(&mut self) {
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
            match self
                .internal_worker
                .run_job(job, &mut self.database, &mut self.producer)
                .await
            {
                Ok(_) => {
                    self.consumer.ack(&msg).await;
                    println!("Job completed successfully, id: {:?}", job_id);
                    info!("Job completed successfully, id: {:?}", job_id)
                }
                Err(e) => {
                    self.consumer.nack(&msg).await;
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
