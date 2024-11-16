use std::time::Duration;

use tokio::time::sleep;
use uuid::Uuid;
use Company_Investigation::{
    jobs::{Job, RecursiveShareholders},
    pulsar::PulsarClient,
};

#[tokio::main]
async fn main() {
    let pulsar_client = PulsarClient::new().await;
    let mut producer = pulsar_client.create_producer().await;

    loop {
        let job = Job::RecursiveShareholders(RecursiveShareholders {
            parent_id: Uuid::new_v4(),
            parent_company_id: "02627406".to_string(),
            remaining_depth: 3,
        });

        producer.produce_message(job).await.unwrap();
        sleep(Duration::from_secs(20)).await;
        break;
    }
}
