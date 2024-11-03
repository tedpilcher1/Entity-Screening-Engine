use futures::TryStreamExt;
use pulsar::{Consumer, Producer, TokioExecutor};
use sqlx::{Connection, PgConnection};
use tokio::time::Interval;
use uuid::Uuid;

use crate::{postgres::init_db, pulsar::{produce_message, CompanyId}};

const DB_URL: &str = "postgres://localhost/postgres";

// TODO: Should not return Result, handle error with log message and gracefully terminate
// TODO: Refactor workers to use single function with enum for worker type
// For this, producer & consumer should be built internally, dependent on worker_type
pub async fn worker(mut consumer: Consumer<CompanyId, TokioExecutor>) -> Result<(), failure::Error> {
    let mut conn = PgConnection::connect(DB_URL).await.unwrap();
    init_db(&mut conn).await.unwrap();

    while let Some(msg) = consumer.try_next().await? {
        consumer.ack(&msg).await?;
        let company_id = match msg.deserialize() {
            Ok(data) => data,
            Err(e) => {
                log::error!("could not deserialize message: {:?}", e);
                break;
            }
        };

        println!("{:?}", company_id);
    }
    
    
    Ok(())
}

// TODO: Should not return Result, handle error with log message and gracefully terminate
pub async fn test_generate_messages_worker(mut producer: Producer<TokioExecutor>, mut interval: Interval) -> Result<(), failure::Error> {

    loop {
        interval.tick().await;
        let uuid = Uuid::new_v4();
        let company_house_id = "Test".to_string();
        produce_message(&mut producer, uuid.clone(), company_house_id.clone()).await?;
        println!("Produced new message: {:?} {:?}", uuid, company_house_id);
    }
}