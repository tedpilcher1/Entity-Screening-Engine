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
    pub day: Option<i32>,
    pub month: Option<i32>,
    pub year: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OfficerListResponse {
    pub active_count: Option<i32>,
    pub etag: Option<String>,
    pub items: Option<Vec<OfficerListItem>>,
    pub items_per_page: Option<i32>,
    pub kind: Option<String>,
    pub links: Links,
    pub start_index: Option<i32>,
    pub total_results: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OfficerListItem {
    pub address: Option<Address>,
    pub appointed_before: Option<String>,
    pub appointed_on: Option<NaiveDate>,
    pub contact_details: Option<ContactDetails>,
    pub date_of_birth: Option<DateOfBirth>,
    pub etag: Option<String>,
    pub former_names: Option<Vec<FormerNames>>,
    pub identification: Option<Identification>,
    pub is_pre_1992_appointment: Option<bool>,
    pub links: Option<OfficerLinks>,
    pub name: Option<String>,
    pub nationality: Option<String>,
    pub occupation: Option<String>,
    pub officer_role: Option<String>,
    pub person_number: Option<String>,
    pub principal_office_address: Option<PrincipalOfficerAddress>,
    pub resigned_on: Option<NaiveDate>,
    pub responsibilities: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContactDetails {
    contact_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FormerNames {
    forenames: Option<String>,
    surname: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Identification {
    pub identification_type: Option<String>,
    pub legal_authority: Option<String>,
    pub legal_form: Option<String>,
    pub place_registered: Option<String>,
    pub registration_number: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OfficerLinks {
    pub officer: Option<Officer>,
    pub self_: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Officer {
    pub appointments: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrincipalOfficerAddress {
    pub address_line_1: Option<String>,
    pub address_line_2: Option<String>,
    pub care_of: Option<String>,
    pub country: Option<String>,
    pub locality: Option<String>,
    pub po_box: Option<String>,
    pub postal_code: Option<String>,
    pub premises: Option<String>,
    pub region: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShareholderList {
    pub active_count: Option<i32>,
    pub creased_count: Option<i32>,
    pub items: Option<Vec<ShareholderListItem>>,
    pub items_per_page: Option<i32>,
    pub links: Option<ShareholderListLinks>,
    pub start_index: Option<i32>,
    pub total_result: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShareholderListItem {
    pub address: Option<PrincipalOfficerAddress>,
    pub ceased: Option<bool>,
    pub ceased_on: Option<NaiveDate>,
    pub country_of_residence: Option<String>,
    pub date_of_birth: Option<DateOfBirth>,
    pub description: Option<String>,
    pub etag: Option<String>,
    pub identification: Option<ShareholderID>,
    pub is_sanctioned_bool: Option<bool>,
    pub kind: Option<String>,
    pub links: Option<ShareholderLinks>,
    pub name: Option<String>,
    pub name_elements: Option<NameElements>,
    pub nationality: Option<String>,
    pub nature_of_control: Option<Vec<String>>,
    pub notified_on: Option<NaiveDate>,
    pub principal_office_address: Option<PrincipalOfficerAddress>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShareholderID {
    pub country_registered: Option<String>,
    pub legal_authority: Option<String>,
    pub legal_form: Option<String>,
    pub place_registered: Option<String>,
    pub registration_number: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShareholderLinks {
    pub self_: Option<String>,
    pub statement: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NameElements {
    pub forename: Option<String>,
    pub middle_name: Option<String>,
    pub surname: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShareholderListLinks {
    persons_with_significant_control_list: Option<String>,
    self_: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppointmentsResponse {
    pub date_of_birth: Option<SimpleDateOfBirth>,
    pub etag: Option<String>,
    pub is_corporate_officer: Option<bool>,
    pub items: Option<Vec<AppointmentListItem>>,
    pub items_per_page: Option<i32>,
    pub kind: Option<String>,
    pub links: Option<Links>,
    pub name: Option<String>,
    pub start_index: Option<i32>,
    pub total_results: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppointmentListItem {
    pub address: Option<Address>,
    pub appointed_before: Option<String>,
    pub appointed_on: Option<NaiveDate>,
    pub appointed_to: Option<AppointedTo>,
    pub contact_details: Option<ContactDetails>,
    pub country_of_residence: Option<String>,
    pub former_names: Option<Vec<FormerName>>,
    pub identification: Option<Identification>,
    pub is_pre_1992_appointment: Option<bool>,
    pub links: Option<Links>,
    pub name: Option<String>,
    pub name_elements: Option<NameElements>,
    pub nationality: Option<String>,
    pub occupation: Option<String>,
    pub officer_role: Option<String>,
    pub principal_office_address: Option<Address>,
    pub resigned_on: Option<NaiveDate>,
    pub responsibilities: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SimpleDateOfBirth {
    pub month: Option<i32>,
    pub year: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppointedTo {
    pub company_name: Option<String>,
    pub company_number: Option<String>,
    pub company_status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FormerName {
    pub forenames: Option<String>,
    pub surname: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilingHistoryResponse {
    pub etag: Option<String>,
    pub filing_history_status: Option<String>,
    pub items: Vec<FilingHistoryItem>,
    pub items_per_page: Option<i32>,
    pub kind: Option<String>,
    pub start_index: Option<i32>,
    pub total_count: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilingHistoryItem {
    pub annotations: Option<Vec<Annotation>>,
    pub associated_filings: Option<Vec<AssociatedFiling>>,
    pub barcode: Option<String>,
    pub category: Option<String>,
    pub date: Option<NaiveDate>,
    pub description: Option<String>,
    pub links: Option<ItemLinks>,
    pub pages: Option<i32>,
    pub paper_filed: Option<bool>,
    pub resolutions: Option<Vec<Resolution>>,
    pub subcategory: Option<String>,
    pub transaction_id: Option<String>,
    pub r#type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Annotation {
    pub annotation: Option<String>,
    pub date: Option<NaiveDate>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssociatedFiling {
    pub date: Option<NaiveDate>,
    pub description: Option<String>,
    pub r#type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ItemLinks {
    pub document_metadata: Option<String>,
    pub self_: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Resolution {
    pub category: Option<String>,
    pub description: Option<String>,
    pub document_id: Option<String>,
    pub receive_date: Option<NaiveDate>,
    pub subcategory: Option<String>,
    pub r#type: Option<String>,
}
