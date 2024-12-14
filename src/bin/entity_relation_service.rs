use dotenv::dotenv;
use Company_Investigation::workers::entity_relation_worker::EntityRelationWorker;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();
    let mut worker = EntityRelationWorker::new_worker()
        .await
        .expect("Should be able to create worker");
    worker.do_work().await;
}
