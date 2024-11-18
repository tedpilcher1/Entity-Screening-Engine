use Company_Investigation::{
    jobs::{Job, RecursiveShareholders},
    postgres::Database,
    pulsar::PulsarClient,
};

async fn simulate_find_shareholders_endpoint() {
    let company_id = "02627406".to_string();
    let depth = 3;

    let pulsar_client = PulsarClient::new().await;
    let mut producer = pulsar_client.create_producer().await;

    let mut conn = Database::connect().await.unwrap();

    let _check_id = conn.insert_check().await.expect("should be able to create check");

    let parent_id = conn.insert_root_entity(&company_id).await.unwrap();

    let job = Job::RecursiveShareholders(RecursiveShareholders {
        parent_id,
        parent_company_id: company_id,
        remaining_depth: depth,
        get_officers: true,
    });

    producer.produce_message(job).await.unwrap();
}

#[tokio::main]
async fn main() {
    simulate_find_shareholders_endpoint().await
}
