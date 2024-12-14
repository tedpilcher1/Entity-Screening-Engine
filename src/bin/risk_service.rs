use dotenv::dotenv;
use Company_Investigation::workers::risk_worker::RiskWorker;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();
    let mut worker = RiskWorker::new_worker()
        .await
        .expect("Should be able to create worker");
    worker.do_work().await;
}
