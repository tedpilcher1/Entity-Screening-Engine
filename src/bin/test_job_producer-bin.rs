use dotenv::dotenv;
use uuid::Uuid;
use Company_Investigation::{
    jobs::{jobs::JobKind, relation_jobs::{RelationJob, RelationJobKind}}, models::Entity, postgres::Database, pulsar::PulsarClient
};

async fn simulate_find_shareholders_endpoint() {
    let company_house_number = "04366849".to_string();

    let pulsar_client = PulsarClient::new().await;
    let mut producer = pulsar_client.create_producer().await;

    let company_house_number = format!("{:0>8}", company_house_number);

    let mut conn = Database::connect().expect("Should be able to connect to db");
    let check_id = conn.insert_check().expect("Should be able to insert check");
    let child_id = conn
        .insert_entity(
            &Entity {
                id: Uuid::new_v4(),
                is_root: true,
                company_house_number: company_house_number.clone(),
                ..Default::default()
            },
            check_id,
        )
        .expect("Should be able to insert root entity");

    let job_kind = JobKind::RelationJob(RelationJob {
        child_id,
        check_id,
        company_house_number,
        remaining_shareholder_depth: 5,
        remaining_officer_depth: 5,
        remaining_appointment_depth: 5,
        relation_job_kind: RelationJobKind::Officers,
    });

    producer
        .enqueue_job(&mut conn, check_id, job_kind)
        .await
        .unwrap();
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();
    simulate_find_shareholders_endpoint().await
}
