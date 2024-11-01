use reqwest::{self, header, Client};
use std::collections::HashMap;

use crate::company_house_response_types::{CompanySearchResponse, OfficerListResponse};

const COMPANY_SEARCH_URL: &str = "https://api.company-information.service.gov.uk/search/companies";

pub async fn get_company(api_key: &String, name: &String) {
    let client = Client::new();

    let mut params = HashMap::new();
    params.insert("q", name);

    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Authorization",
        header::HeaderValue::from_str(&format!("{}", api_key)).unwrap(),
    );

    let response = client
        .get(COMPANY_SEARCH_URL)
        .headers(headers)
        .query(&params)
        .send()
        .await
        .unwrap();
    let company_search_response: CompanySearchResponse = response.json().await.unwrap();

    // some work required here to convert the search response into some generic company struct
}

pub async fn get_company_officers(api_key: &String, company_number: &String) -> OfficerListResponse {

    // TODO: urls should be lazy-loaded
    let url = format!("https://api.company-information.service.gov.uk/company/{}/officers", company_number);
    let client = Client::new();

    let mut params = HashMap::new();
    params.insert("q", "Google"); // TODO: stop this from being hardcoded

    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Authorization",
        header::HeaderValue::from_str(&format!("{}", api_key)).unwrap(),
    );

    let response = client
        .get(url)
        .headers(headers)
        .query(&params)
        .send()
        .await
        .unwrap();

    // TODO: Add validation etc

    let officer_search_response: OfficerListResponse = response.json().await.unwrap();
    officer_search_response
}

// TODO: Stop passing around api_key it's a bit dumb
pub async fn get_company_shareholders(api_key: &String, company_number: &String){

    let url = format!("https://api.company-information.service.gov.uk/company/{}/persons-with-significant-control", company_number);
    // TODO: create own client struct that implements fns in this file, has client as field
    // I'm actually not sure if this is a good idea? - should check what we do in bridge for inspiration! 
    let client = Client::new();

    // TODO: could stop duplication and do this once somewhere
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Authorization",
        header::HeaderValue::from_str(&format!("{}", api_key)).unwrap(),
    );
    
    let response = client
        .get(url)
        .headers(headers)
        .send()
        .await
        .unwrap();
}
