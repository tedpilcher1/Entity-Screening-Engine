use futures::TryStreamExt;
use log::{info, warn};
use pulsar::SubType;

use crate::{
    jobs::jobs::Job,
    pulsar::{PulsarClient, PulsarConsumer},
};

pub trait Work {
    fn work(
        &mut self,
        job: Job,
    ) -> impl std::future::Future<Output = Result<(), failure::Error>> + Send;
}

pub struct Worker<T: Work> {
    pub consumer: PulsarConsumer,
    pub internal_worker: T,
}

impl<T: Work> Worker<T> {
    pub async fn new(
        topics: Vec<&str>,
        sub: &str,
        sub_type: SubType,
        internal_worker: T,
    ) -> Result<Self, failure::Error> {
        let pulsar_client = PulsarClient::new().await;

        Ok(Self {
            consumer: pulsar_client.create_consumer(topics, sub_type, sub).await,
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
            match self.internal_worker.work(job).await {
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
