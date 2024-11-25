use pulsar::{producer, proto, Consumer, Producer, Pulsar, SubType, TokioExecutor};
use uuid::Uuid;

use crate::{jobs::{Job, JobKind}, postgres::Database};

const PULSAR_ADDR: &str = "pulsar://localhost:6650";
const TOPIC: &str = "non-persistent://public/default/testing";

pub struct PulsarClient {
    pub internal_client: Pulsar<TokioExecutor>,
}

impl PulsarClient {
    pub async fn new() -> Self {
        Self {
            internal_client: Pulsar::builder(PULSAR_ADDR, TokioExecutor)
                .build()
                .await
                .expect("Should be able to create new pulsar client builder"),
        }
    }

    pub async fn create_producer(&self) -> PulsarProducer {
        let id = Uuid::new_v4();
        PulsarProducer {
            id,
            internal_producer: self
                .internal_client
                .producer()
                .with_topic(TOPIC)
                .with_name("PRODUCER_".to_owned() + &id.to_string())
                .with_options(producer::ProducerOptions {
                    schema: Some(proto::Schema {
                        r#type: proto::schema::Type::String as i32, // Or appropriate type for Job
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .build()
                .await
                .expect("Should be able to create producer"),
        }
    }

    pub async fn create_consumer(&self) -> PulsarConsumer {
        let id = Uuid::new_v4();
        PulsarConsumer {
            id,
            internal_consumer: self
                .internal_client
                .consumer()
                .with_topic(TOPIC)
                .with_consumer_name("CONSUMER_".to_owned() + &id.to_string())
                .with_subscription_type(SubType::Exclusive) // exclusive for current testing
                // .with_subscription("SUB_".to_owned() + &id.to_string())
                .with_subscription("test-sub")
                .build()
                .await
                .expect("Should be able to create consumer"),
        }
    }
}

pub struct PulsarProducer {
    pub id: Uuid,
    pub internal_producer: Producer<TokioExecutor>,
}

impl PulsarProducer {
    async fn produce_message(&mut self, job: Job) -> Result<(), failure::Error> {
        self.internal_producer.send(job).await?;
        Ok(())
    }

    pub async fn enqueue_job(&mut self, database: &mut Database, job_kind: JobKind) -> Result<(), failure::Error> {
        let job_id = database.add_job()?;
        self.produce_message(Job {
            id: job_id,
            job_kind,
        }).await?;
        Ok(())
    }
}

pub struct PulsarConsumer {
    pub id: Uuid,
    pub internal_consumer: Consumer<Job, TokioExecutor>,
}
