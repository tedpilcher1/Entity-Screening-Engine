use ::pulsar::producer;
use pulsar::{get_consumer, get_producer, get_pulsar_client};
use tokio::{signal, time::{self, Interval}};
use workers::{test_generate_messages_worker, worker};

pub mod company_house_apis;
mod company_house_response_types;
mod jobs;
mod postgres;
mod pulsar;
pub mod types;
mod workers;

const NUM_WORKERS: usize = 2;
const TEST_PRODUCER_WORKERS: usize = 1;

#[tokio::main]
async fn main() {
    let pulsar_client = get_pulsar_client().await;
    let mut workers = Vec::new();

    for _ in 0..NUM_WORKERS {
        let consumer = get_consumer(&pulsar_client).await;
        let handle = tokio::spawn(async move {
            worker(consumer).await.unwrap();
        });
        workers.push(handle);
    }

    for _ in 0..TEST_PRODUCER_WORKERS {
        let producer = get_producer(&pulsar_client).await;
        let handle = tokio::spawn(async move {
            test_generate_messages_worker(producer, time::interval(time::Duration::from_secs(1))).await.unwrap();
        });
        workers.push(handle);
    }

    signal::ctrl_c()
        .await
        .expect("Should be able to listen for kill signal");

    for worker in workers {
        let _ = worker.abort();
    }
}
