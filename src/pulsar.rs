use pulsar::{producer, proto, Consumer, DeserializeMessage, Error as PulsarError, Payload, Producer, Pulsar, SerializeMessage, SubType, TokioExecutor};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const PULSAR_ADDR: &str = "pulsar://localhost:6650";

#[derive(Serialize, Deserialize, Debug)]
pub struct CompanyId {
    company_id: Uuid,
    company_house_id: String,
}

pub struct PulsarClient {
    internal_client: Pulsar<TokioExecutor>,
}

impl PulsarClient {
    pub async fn new() -> Self {
        Self{
            internal_client: Pulsar::builder(PULSAR_ADDR, TokioExecutor)
            .build()
            .await
            .expect("Should be able to create new pulsar client builder"),
        }   
    }

    pub async fn create_producer(&self) -> PulsarProducer {
        PulsarProducer {
            id: Uuid::new_v4(),
            internal_producer: self.internal_client
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
            .expect("Should be able to create producer")
        }
    }

    pub async fn create_consumer(&self) -> PulsarConsumer {
        PulsarConsumer {
            id: Uuid::new_v4(),
            internal_consumer: self.internal_client.consumer()
            .with_topic("non-persistent://public/default/test")
            .with_consumer_name("consumer")
            .with_subscription_type(SubType::Shared)
            .with_subscription("test_subscription")
            .build()
            .await
            .expect("Should be able to create consumer")
        }
    }
}

pub struct PulsarProducer {
    id: Uuid,
    internal_producer: Producer<TokioExecutor>,
}

impl PulsarProducer {
    pub async fn produce_message(
        &mut self,
        company_id: Uuid,
        company_house_id: String,
    ) -> Result<(), failure::Error> {
        self.internal_producer
            .send(CompanyId {
                company_id,
                company_house_id,
            })
            .await?;
    
        Ok(())
    }
}

pub struct PulsarConsumer {
    id: Uuid,
    internal_consumer: Consumer<CompanyId, TokioExecutor>
}

impl SerializeMessage for CompanyId {
    fn serialize_message(input: Self) -> Result<producer::Message, PulsarError> {
        let payload = serde_json::to_vec(&input).map_err(|e| PulsarError::Custom(e.to_string()))?;
        Ok(producer::Message {
            payload,
            ..Default::default()
        })
    }
}

impl DeserializeMessage for CompanyId {
    type Output = Result<CompanyId, serde_json::Error>;

    fn deserialize_message(payload: &Payload) -> Self::Output {
        serde_json::from_slice(&payload.data)
    }
}