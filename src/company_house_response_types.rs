use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CompanySearchResponse {
    pub etag: Option<String>,
    pub items: Option<Vec<CompanyItem>>,
    pub items_per_page: Option<i32>,
    pub kind: Option<String>,
    pub start_index: Option<i32>,
    pub total_results: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompanyItem {
    pub address: Address,
    pub address_snippet: String,
    pub company_number: String,
    pub company_status: String,
    pub company_type: String,
    pub date_of_cessation: Option<NaiveDate>,
    pub date_of_creation: NaiveDate,
    pub description: Option<String>,
    pub description_identifier: Option<Vec<String>>,
    pub kind: String,
    pub links: Option<Links>,
    pub matches: Option<Matches>,
    pub snippet: Option<String>,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Address {
    pub address_line_1: Option<String>,
    pub address_line_2: Option<String>,
    pub care_of: Option<String>,
    pub country: Option<String>,
    pub locality: Option<String>,
    pub po_box: Option<String>,
    pub postal_code: Option<String>,
    pub region: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Links {
    pub _self: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Matches {
    pub address_snippet: Option<Vec<i32>>,
    pub snippet: Option<Vec<i32>>,
    pub title: Option<Vec<i32>>,
} 

#[derive(Debug, Serialize, Deserialize)]
pub struct OfficerSearchResponse {
    pub etag: Option<String>,
    pub items: Option<Vec<CompanyItem>>,
    pub kind: Option<String>,
    pub start_index: Option<i32>,
    pub total_results: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OfficerItem {
    pub address: Address,
    pub address_snippet: String,
    pub appointment_count: i32,
    pub date_of_birth: Option<DateOfBirth>,
    pub description: Option<String>,
    pub description_identifier: Option<Vec<String>>,
    pub kind: String,
    pub links: Option<Links>,
    pub matches: Option<Matches>,
    pub snippet: Option<String>,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DateOfBirth {
    pub month: i32,
    pub year: i32,
}