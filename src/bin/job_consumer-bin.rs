use dotenv::dotenv;
use Company_Investigation::worker::Worker;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();
    let mut worker = Worker::new().await.unwrap();
    worker.do_work().await.unwrap();
}
