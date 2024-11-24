use dotenv::dotenv;
use uuid::Uuid;
use Company_Investigation::{
    jobs::{Job, Shareholders},
    models::Entity,
    postgres::Database,
    pulsar::PulsarClient,
};

async fn simulate_find_shareholders_endpoint() {
    let company_id = "02627406".to_string();
    let depth = 3;

    let pulsar_client = PulsarClient::new().await;
    let mut producer = pulsar_client.create_producer().await;

    let mut conn = Database::connect().expect("Should be able to connect to db");
    let check_id = conn.insert_check().expect("Should be able to insert check");
    let parent_id = conn
        .insert_entity(
            &Entity {
                id: Uuid::new_v4(),
                is_root: true,
                ..Default::default()
            },
            check_id,
        )
        .expect("Should be able to insert root entity");

    let job = Job::Shareholders(Shareholders {
        parent_id,
        check_id,
        parent_company_number: company_id,
        remaining_shareholder_depth: 5,
        remaining_officers_depth: 5,
    });

    producer.produce_message(job).await.unwrap();
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();
    simulate_find_shareholders_endpoint().await
}
