use dotenv::var;

mod company_house_apis;
mod company_house_response_types;

#[tokio::main]
async fn main() {
    let api_key = var("COMPANY_HOUSE_API_KEY_TEST").unwrap();

}
