use pulsar::{
    producer, proto, Consumer, DeserializeMessage, Error as PulsarError, Payload, Producer, Pulsar,
    SerializeMessage, SubType, TokioExecutor,
};
use uuid::Uuid;

use crate::jobs::Job;

const PULSAR_ADDR: &str = "pulsar://localhost:6650";
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
        PulsarProducer {
            id: Uuid::new_v4(),
            internal_producer: self
                .internal_client
                .producer()
                .with_topic("non-persistent://public/default/test")
                .with_name("producer")
                .with_options(producer::ProducerOptions {
                    schema: Some(proto::Schema {
                        r#type: proto::schema::Type::String as i32,
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
        PulsarConsumer {
            id: Uuid::new_v4(),
            internal_consumer: self
                .internal_client
                .consumer()
                .with_topic("non-persistent://public/default/test")
                .with_consumer_name("consumer")
                .with_subscription_type(SubType::Shared)
                .with_subscription("test_subscription")
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
    pub async fn produce_message(&mut self) -> Result<(), failure::Error> {
        // self.internal_producer
        //     .send(CompanyId {
        //         company_id,
        //         company_house_id,
        //     })
        //     .await?;

        Ok(())
    }
}

pub struct PulsarConsumer {
    pub id: Uuid,
    pub internal_consumer: Consumer<Job, TokioExecutor>,
}
