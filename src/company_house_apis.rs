use std::collections::HashMap;
use reqwest::{self, header, Client};

use crate::company_house_response_types::CompanySearchResponse;

const COMPANY_SEARCH_URL: &str = "https://api.company-information.service.gov.uk/search/companies";
const COMPANY_OFFICERS_URL: &str = "https://api.company-information.service.gov.uk/search/officers";

pub async fn get_company_by_name(api_key: &String, name: &String) {

    let client = Client::new();

    let mut params = HashMap::new();
    params.insert("q", name);
    
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Authorization",
        header::HeaderValue::from_str(&format!("{}", api_key)).unwrap(),
    );

    let response = client.get(COMPANY_SEARCH_URL).headers(headers).query(&params).send().await.unwrap();
    let company_search_response: CompanySearchResponse = response.json().await.unwrap();
    
    // some work required here to convert the search response into some generic company struct
}


async fn get_company(api_key: &String){
    let client = Client::new();

    let mut params = HashMap::new();
    params.insert("q", "Google");
    
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Authorization",
        header::HeaderValue::from_str(&format!("{}", api_key)).unwrap(),
    );

    let response = client.get(COMPANY_OFFICERS_URL).headers(headers).query(&params).send().await.unwrap();

    // todo
}


