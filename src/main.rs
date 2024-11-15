use worker::Worker;

pub mod company_house_apis;
mod company_house_response_types;
mod jobs;
mod postgres;
mod pulsar;
pub mod types;
mod worker;

#[tokio::main]
async fn main() {
    let mut worker = Worker::new().await.unwrap();
    worker.do_work().await.unwrap();
}
