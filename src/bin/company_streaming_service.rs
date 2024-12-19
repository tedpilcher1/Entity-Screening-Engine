use dotenv::dotenv;
use Company_Investigation::workers::streaming_worker::{StreamingKind, StreamingWorker};

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let mut worker = StreamingWorker::new(StreamingKind::Company)
        .await
        .expect("Should be able to create company streaming worker");
    worker.do_work().await.unwrap();
}
