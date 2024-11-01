use company_house_apis::get_company_officers;
use dotenv::var;

mod company_house_response_types;
pub mod company_house_apis;
pub mod types;

#[tokio::main]
async fn main() {
    let api_key = var("COMPANY_HOUSE_API_KEY_TEST").unwrap();

    let company_number: String = "03977902".to_string();
    let officer_search_response = get_company_officers(&api_key, &company_number).await;

    // need method to determine if OfficerItem is company or individual

    // for each company officer
    for item in officer_search_response.items.unwrap() {

        println!("{:?}", item)
    }
}
