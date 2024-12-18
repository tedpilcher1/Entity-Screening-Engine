use dotenv::dotenv;
use Company_Investigation::workers::company_streaming_worker::{
    StreamingWorker, StreamingWorkerKind,
};

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let worker = StreamingWorker::new(StreamingWorkerKind::Company)
        .await
        .expect("Should be able to create company streaming worker");
    worker.do_work().await.unwrap();
}
