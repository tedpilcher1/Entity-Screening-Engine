use sqlx::PgConnection;
use uuid::Uuid;

use crate::{postgres::connect, pulsar::{PulsarClient, PulsarConsumer, PulsarProducer}};

pub struct Worker {
    id: Uuid,
    conn: PgConnection,
    producer: PulsarProducer,
    consumer: PulsarConsumer,
}

impl Worker {
    pub async fn new() -> Self {

        let pulsar_client = PulsarClient::new().await;

        Self {
            id: Uuid::new_v4(),
            conn: connect().await,
            producer: pulsar_client.create_producer().await,
            consumer: pulsar_client.create_consumer().await,
        }
    } 

    pub fn do_work() {

        // wait for 


    }
}