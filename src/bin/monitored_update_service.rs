use dotenv::dotenv;
use Company_Investigation::workers::monitored_update_worker::MonitoredUpdateWorker;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let mut worker = MonitoredUpdateWorker::new_worker()
        .await
        .expect("Should be able to create monitored update worker");
    worker.do_work().await;
}
