use pulsar::{producer, proto, Consumer, DeserializeMessage, Error as PulsarError, Payload, Producer, Pulsar, SerializeMessage, SubType, TokioExecutor};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const PULSAR_ADDR: &str = "pulsar://localhost:6650";

#[derive(Serialize, Deserialize, Debug)]
pub struct CompanyId {
    company_id: Uuid,
    company_house_id: String,
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

pub async fn produce_message(
    producer: &mut Producer<TokioExecutor>,
    company_id: Uuid,
    company_house_id: String,
) -> Result<(), failure::Error> {
    producer
        .send(CompanyId {
            company_id,
            company_house_id,
        })
        .await?;

    Ok(())
}

pub async fn get_pulsar_client() -> Pulsar<TokioExecutor> {
    Pulsar::builder(PULSAR_ADDR, TokioExecutor)
        .build()
        .await
        .expect("Should be able to create new pulsar client builder")
}

pub async fn get_producer(pulsar: &Pulsar<TokioExecutor>) -> Producer<TokioExecutor> {
    pulsar
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

pub async fn get_consumer(pulsar: &Pulsar<TokioExecutor>) -> Consumer<CompanyId, TokioExecutor> {
    pulsar
        .consumer()
        .with_topic("non-persistent://public/default/test")
        .with_consumer_name("consumer")
        .with_subscription_type(SubType::Shared)
        .with_subscription("test_subscription")
        .build()
        .await
        .expect("Should be able to create consumer")
}