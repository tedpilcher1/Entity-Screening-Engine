use Company_Investigation::{
    jobs::{Job, RecursiveShareholders}, postgres::Database, pulsar::PulsarClient
};


async fn simulate_find_shareholders_endpoint() {

    let company_id = "02627406".to_string();
    let depth = 3;

    let pulsar_client = PulsarClient::new().await;
    let mut producer = pulsar_client.create_producer().await;

    let mut conn = Database::connect().await.unwrap();

    let parent_id = conn.insert_company(&company_id, None, None, None, None).await.unwrap();

    let job = Job::RecursiveShareholders(RecursiveShareholders {
        parent_id,
        parent_company_id: company_id,
        remaining_depth: depth,
    });

    producer.produce_message(job).await.unwrap();
}

#[tokio::main]
async fn main() {
    simulate_find_shareholders_endpoint().await
}
