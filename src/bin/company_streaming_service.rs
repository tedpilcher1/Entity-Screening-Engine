use dotenv::dotenv;
use Company_Investigation::company_house::company_house_streaming_client::CompanyHouseStreamingClient;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let company_streaming_client = CompanyHouseStreamingClient::new();
    company_streaming_client.connect_to_company_stream().await.unwrap();
}