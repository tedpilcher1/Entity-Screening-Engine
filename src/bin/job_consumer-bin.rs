use Company_Investigation::worker::Worker;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let mut worker = Worker::new().await.unwrap();
    worker.do_work().await.unwrap();
}
