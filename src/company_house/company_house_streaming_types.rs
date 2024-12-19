use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompanyStreamingResponse {
    pub data: Option<CompanyData>,
    pub event: Option<Event>,
    pub resource_id: Option<String>,
    pub resource_kind: Option<String>,
    pub resource_uri: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompanyData {
    pub accounts: Option<Accounts>,
    pub annual_return: Option<AnnualReturn>,
    pub branch_company_details: Option<BranchCompanyDetails>,
    pub can_file: Option<bool>,
    pub company_name: Option<String>,
    pub company_number: String,
    pub company_status: Option<String>,
    pub company_status_detail: Option<String>,
    pub confirmation_statement: Option<ConfirmationStatement>,
    pub corporate_annotation: Option<Vec<CorporateAnnotation>>,
    pub date_of_cessation: Option<String>,
    pub date_of_creation: Option<String>,
    pub etag: Option<String>,
    pub external_registration_number: Option<String>,
    pub foreign_company_details: Option<ForeignCompanyDetails>,
    pub has_been_liquidated: Option<bool>,
    pub has_charges: Option<bool>,
    pub has_insolvency_history: Option<bool>,
    pub is_community_interest_company: Option<bool>,
    pub jurisdiction: Option<String>,
    pub last_full_members_list_date: Option<String>,
    pub links: Option<Links>,
    pub partial_data_available: Option<String>,
    pub previous_company_names: Option<Vec<PreviousCompanyName>>,
    pub registered_office_address: Option<Address>,
    pub registered_office_is_in_dispute: Option<bool>,
    pub service_address: Option<Address>,
    pub sic_codes: Option<Vec<String>>,
    pub subtype: Option<String>,
    // pub super_secure_managing_officer_count: Option<i64>,
    pub r#type: Option<String>,
    pub undeliverable_registered_office_address: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Accounts {
    pub accounting_reference_date: Option<AccountingReferenceDate>,
    pub last_accounts: Option<LastAccounts>,
    pub next_accounts: Option<NextAccounts>,
    pub next_due: Option<String>,
    pub next_made_up_to: Option<String>,
    pub overdue: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountingReferenceDate {
    // pub day: Option<i64>,
    // pub month: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LastAccounts {
    pub made_up_to: Option<String>,
    pub period_end_on: Option<String>,
    pub period_start_on: Option<String>,
    pub r#type: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NextAccounts {
    pub due_on: Option<String>,
    pub overdue: Option<bool>,
    pub period_end_on: Option<String>,
    pub period_start_on: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnnualReturn {
    pub last_made_up_to: Option<String>,
    pub next_due: Option<String>,
    pub next_made_up_to: Option<String>,
    pub overdue: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BranchCompanyDetails {
    pub business_activity: Option<String>,
    pub parent_company_name: Option<String>,
    pub parent_company_number: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfirmationStatement {
    pub last_made_up_to: Option<String>,
    pub next_due: Option<String>,
    pub next_made_up_to: Option<String>,
    pub overdue: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CorporateAnnotation {
    pub created_on: Option<String>,
    pub description: Option<String>,
    pub r#type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ForeignCompanyDetails {
    pub accounting_requirement: Option<AccountingRequirement>,
    pub accounts: Option<ForeignCompanyAccounts>,
    pub business_activity: Option<String>,
    pub company_type: Option<String>,
    pub governed_by: Option<String>,
    pub is_a_credit_finance_institution: Option<bool>,
    pub originating_registry: Option<OriginatingRegistry>,
    pub registration_number: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountingRequirement {
    pub foreign_account_type: Option<String>,
    pub terms_of_account_publication: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ForeignCompanyAccounts {
    pub account_period_from: Option<AccountPeriod>,
    pub account_period_to: Option<AccountPeriod>,
    pub must_file_within: Option<MustFileWithin>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountPeriod {
    // pub day: Option<i64>,
    // pub month: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MustFileWithin {
    // pub months: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OriginatingRegistry {
    pub country: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Links {
    pub charges: Option<String>,
    pub exemptions: Option<String>,
    pub filing_history: Option<String>,
    pub insolvency: Option<String>,
    pub officers: Option<String>,
    pub overseas: Option<String>,
    pub persons_with_significant_control: Option<String>,
    pub persons_with_significant_control_statements: Option<String>,
    pub registers: Option<String>,
    pub self_: Option<String>,
    pub uk_establishments: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PreviousCompanyName {
    pub ceased_on: Option<String>,
    pub effective_from: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Address {
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    pub fields_changed: Option<Vec<String>>,
    pub published_at: Option<String>,
    pub timepoint: i32,
    pub r#type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OfficerStreamingResponse {
    pub data: Option<OfficerData>,
    pub event: Option<Event>,
    pub resource_id: Option<String>,
    pub resource_kind: Option<String>,
    pub resource_uri: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OfficerData {
    pub address: Option<Address>,
    pub appointed_before: Option<String>,
    pub appointed_on: Option<String>, // date
    pub contact_details: Option<ContactDetails>,
    pub country_of_residence: Option<String>,
    pub date_of_birth: Option<DateOfBirth>,
    pub etag: Option<String>,
    pub former_names: Option<Vec<Name>>,
    pub identification: Option<Identification>,
    pub is_pre_1992_appointment: Option<bool>,
    pub links: Option<OfficerLink>,
    pub name: Option<String>,
    pub nationality: Option<String>,
    pub occupation: Option<String>,
    pub officer_role: Option<String>,
    pub person_number: Option<String>,
    pub principal_office_address: Option<Address>,
    pub resigned_on: Option<String>, // date
    pub responsibilities: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContactDetails {
    pub contact_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DateOfBirth {
    pub day: Option<i32>,
    pub month: Option<i32>,
    pub year: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Name {
    pub forenames: Option<String>,
    pub surname: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Identification {
    pub identification_type: Option<String>,
    pub legal_authority: Option<String>,
    pub legal_form: Option<String>,
    pub place_registered: Option<String>,
    pub registration_number: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OfficerLink {
    pub officer: Option<LinkedOfficer>,
    pub self_link: Option<String>, // renamed from "self" as it's a reserved keyword
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LinkedOfficer {
    pub appointments: Option<String>,
}
