use uuid::Uuid;

use crate::{postgres::Database, pulsar::{PulsarClient, PulsarConsumer, PulsarProducer}};

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

    pub fn do_work() {

        // wait for 


    }
}