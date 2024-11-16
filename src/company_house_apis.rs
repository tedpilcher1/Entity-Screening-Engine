use lazy_static::lazy_static;
use reqwest::{self, header, Client};
use std::{collections::HashMap, env};
use uuid::Uuid;

use crate::company_house_response_types::{
    CompanySearchResponse, OfficerListResponse, ShareholderList,
};

const COMPANY_SEARCH_URL: &str = "https://api.company-information.service.gov.uk/search/companies";
const COMPANY_SHAREHOLDERS_URL: &str =
    "https://api.company-information.service.gov.uk/company/{}/persons-with-significant-control";
const COMPANY_OFFICERS_URL: &str =
    "https://api.company-information.service.gov.uk/company/{}/officers";

lazy_static! {
    static ref API_KEY: String = env::var("API_KEY").expect("API KEY should be set");
}

pub async fn get_company(name: &String) {
    let client = Client::new();

    let mut params = HashMap::new();
    params.insert("q", name);

    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Authorization",
        header::HeaderValue::from_str(&format!("{}", API_KEY.as_str())).unwrap(),
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
    todo!()
}

pub async fn get_company_officers(company_number: &String) -> OfficerListResponse {
    // TODO: urls should be lazy-loaded
    let url = format!("{} {}", COMPANY_OFFICERS_URL, company_number);
    let client = Client::new();

    let mut params = HashMap::new();
    params.insert("q", "Google"); // TODO: stop this from being hardcoded

    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Authorization",
        header::HeaderValue::from_str(&format!("{}", API_KEY.as_str())).unwrap(),
    );

    let response = client
        .get(url)
        .headers(headers)
        .query(&params)
        .send()
        .await
        .unwrap();

    let officer_search_response: OfficerListResponse = response.json().await.unwrap();
    officer_search_response
}

pub async fn get_company_shareholders(
    company_number: &String,
) -> Result<ShareholderList, failure::Error> {
    let url = format!("https://api.company-information.service.gov.uk/company/{}/persons-with-significant-control", company_number);

    // TODO: create own client struct that implements fns in this file, has client as field
    // I'm actually not sure if this is a good idea? - should check what we do in bridge for inspiration!
    let client = Client::new();

    // TODO: could stop duplication and do this once somewhere
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Authorization",
        header::HeaderValue::from_str(&format!("{}", API_KEY.as_str()))?,
    );

    let response = client.get(url).headers(headers).send().await?;

    let shareholder_list: ShareholderList = response.json().await?;
    Ok(shareholder_list)
}
