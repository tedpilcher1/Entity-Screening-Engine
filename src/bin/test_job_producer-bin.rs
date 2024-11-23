use uuid::Uuid;
use Company_Investigation::{
    jobs::{Job, RecursiveShareholders},
    postgres::Database,
    postgres_types::Entity,
    pulsar::PulsarClient,
};

async fn simulate_find_shareholders_endpoint() {
    let company_id = "02627406".to_string();
    let depth = 3;

    let pulsar_client = PulsarClient::new().await;
    let mut producer = pulsar_client.create_producer().await;

    let mut conn = Database::connect().expect("Should be able to connect to db");

    // let check_id = conn
    //     .insert_check()
    //     .expect("should be able to create check");

    // let parent_id = conn
    //     .insert_root_entity(&company_id, &check_id)
    //     .await
    //     .unwrap();
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

    let job = Job::RecursiveShareholders(RecursiveShareholders {
        parent_id,
        check_id,
        parent_company_number: company_id,
        remaining_depth: depth,
        get_officers: true,
    });

    producer.produce_message(job).await.unwrap();
}

#[tokio::main]
async fn main() {
    simulate_find_shareholders_endpoint().await
}
