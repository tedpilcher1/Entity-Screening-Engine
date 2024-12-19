use chrono::{NaiveDate, NaiveDateTime};
use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::prelude::{Insertable, Queryable};
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::Selectable;
use log::warn;
use serde::{Deserialize, Serialize};
use std::io::Write;
use uuid::Uuid;

use crate::company_house::company_house_streaming_types::CompanyData;
use crate::company_house::company_house_types::{
    AppointmentListItem, AppointmentsResponse, CompanyItem, Identification, OfficerListItem,
    OfficerListResponse, ShareholderList, ShareholderListItem,
};

type CompanyHouseNumber = String;

#[derive(Debug, Clone, Copy, AsExpression, FromSqlRow)]
#[diesel(sql_type = crate::schema::sql_types::Relationshipkind)]
pub enum Relationshipkind {
    Shareholder,
    Officer,
}

impl ToSql<crate::schema::sql_types::Relationshipkind, Pg> for Relationshipkind {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            Relationshipkind::Shareholder => out.write_all(b"shareholder")?,
            Relationshipkind::Officer => out.write_all(b"officer")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::Relationshipkind, Pg> for Relationshipkind {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"shareholder" => Ok(Relationshipkind::Shareholder),
            b"officer" => Ok(Relationshipkind::Officer),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Debug, AsExpression, FromSqlRow, Default, Serialize, Deserialize, PartialEq)]
#[diesel(sql_type = crate::schema::sql_types::Entitykind)]
pub enum Entitykind {
    #[default]
    Company,
    Individual,
}

impl ToSql<crate::schema::sql_types::Entitykind, Pg> for Entitykind {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            Entitykind::Company => out.write_all(b"company")?,
            Entitykind::Individual => out.write_all(b"individual")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::Entitykind, Pg> for Entitykind {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"company" => Ok(Entitykind::Company),
            b"individual" => Ok(Entitykind::Individual),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

// TODO: this could be better
impl From<Option<String>> for Entitykind {
    fn from(kind: Option<String>) -> Self {
        match kind.unwrap_or_default().as_str() {
            "individual-person-with-significant-control" => Self::Individual,
            "corporate-entity-person-with-significant-control" => Self::Company,
            "legal-person-with-significant-control" => Self::Individual,
            "super-secure-person-with-significant-control" => Self::Individual,
            "individual-beneficial-owner" => Self::Individual,
            "corporate-entity-beneficial-owner" => Self::Company,
            "legal-person-beneficial-owner" => Self::Individual,
            "super-secure-beneficial-owner" => Self::Company,
            _ => Self::Company,
        }
    }
}

fn match_number_to_kind(
    person_number: &Option<String>,
    identification: &Option<Identification>,
) -> Option<(CompanyHouseNumber, Entitykind)> {
    match person_number {
        Some(person_number) => return Some((person_number.clone(), Entitykind::Individual)),
        None => {}
    }

    match identification {
        Some(identification) => {
            // let company_house_number = format!("{:0>8}", identification.registration_number.unwrap_or_else(f));
            let company_house_number = identification.registration_number.clone();

            match company_house_number {
                Some(company_house_number) => {
                    return Some((company_house_number, Entitykind::Company))
                }
                None => return None,
            }
        }
        None => return None,
    }
}

#[derive(Queryable, Selectable, Default, Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::entity)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Entity {
    pub id: Uuid,
    // this will be either company_house_number or person_number
    // TODO: rename to entity_number
    pub company_house_number: String,
    pub name: Option<String>,
    pub kind: Entitykind,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub date_of_origin: Option<String>,
    pub is_root: bool,
    pub officer_id: Option<String>,
}

impl Entity {
    pub fn create_root(company_house_number: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            is_root: true,
            company_house_number,
            ..Default::default()
        }
    }
}

impl From<CompanyData> for Entity {
    fn from(company_data: CompanyData) -> Self {
        let (country, postal_code) = match company_data.registered_office_address {
            Some(officer_address) => (officer_address.country, officer_address.postal_code),
            None => (None, None),
        };

        Self {
            id: Uuid::new_v4(),
            company_house_number: company_data.company_number,
            name: company_data.company_name,
            kind: Entitykind::Company,
            country,
            postal_code,
            date_of_origin: company_data.date_of_creation,
            is_root: false,
            officer_id: None,
        }
    }
}

pub struct EntityRelation {
    pub entity: Entity,
    pub started_on: Option<NaiveDate>,
    pub ended_on: Option<NaiveDate>,
}

impl From<OfficerListResponse> for Vec<EntityRelation> {
    fn from(officers: OfficerListResponse) -> Self {
        let mut entity_relations: Vec<EntityRelation> = Vec::new();
        for officer in officers.items.unwrap_or_default() {
            let entity: Result<EntityRelation, ()> = (officer, false).try_into();
            match entity {
                Ok(entity) => entity_relations.push(entity),
                Err(_) => {
                    warn!("Failed to convert officer into an entity."); // todo improve log
                }
            };
        }
        entity_relations
    }
}

impl From<ShareholderList> for Vec<EntityRelation> {
    fn from(shareholders: ShareholderList) -> Self {
        let mut entity_relations: Vec<EntityRelation> = Vec::new();
        for shareholder in shareholders.items.unwrap_or_default() {
            let entity: Result<EntityRelation, ()> = (shareholder, false).try_into();
            match entity {
                Ok(entity) => entity_relations.push(entity),
                Err(_) => {
                    warn!("Failed to convert shareholder into an entity."); // todo improve log
                }
            }
        }
        entity_relations
    }
}

impl From<AppointmentsResponse> for Vec<EntityRelation> {
    fn from(appointments: AppointmentsResponse) -> Self {
        let mut entity_relations: Vec<EntityRelation> = Vec::new();
        for appointment in appointments.items.unwrap_or_default() {
            let entity: Result<EntityRelation, ()> = appointment.try_into();
            match entity {
                Ok(entity) => entity_relations.push(entity),
                Err(_) => warn!("Failed to convert appointment into an entity"),
            }
        }
        entity_relations
    }
}

impl TryFrom<(ShareholderListItem, bool)> for EntityRelation {
    type Error = ();

    fn try_from(value: (ShareholderListItem, bool)) -> Result<Self, Self::Error> {
        let shareholder = value.0;
        let is_root = value.1;
        let shareholder_identification = match shareholder.identification {
            Some(identification) => identification,
            None => return Err(()),
        };
        let company_house_number = match shareholder_identification.registration_number {
            Some(registration_numer) => registration_numer,
            None => return Err(()),
        };
        let (country, postal_code) = match shareholder.address {
            Some(address) => (address.country, address.postal_code),
            None => (None, None),
        };

        let date_of_origin = match shareholder.date_of_birth {
            Some(dob) => {
                let day = dob.day.unwrap_or_else(|| 0);
                let month = dob.month.unwrap_or_else(|| 0);

                if let Some(year) = dob.year {
                    Some(format!(
                        "{}/{}/{}",
                        day.to_string(),
                        month.to_string(),
                        year.to_string()
                    ))
                } else {
                    None
                }
            }
            None => None,
        };

        let entity = Entity {
            id: Uuid::new_v4(),
            company_house_number,
            officer_id: None,
            name: shareholder.name,
            kind: shareholder.kind.into(),
            country: country,
            postal_code: postal_code,
            date_of_origin,
            is_root,
        };

        Ok(Self {
            entity,
            started_on: shareholder.notified_on,
            ended_on: shareholder.ceased_on,
        })
    }
}

fn extract_officer_id(officer_item: OfficerListItem) -> Result<String, ()> {
    officer_item
        .links
        .as_ref()
        .and_then(|links| links.officer.as_ref())
        .and_then(|officer| officer.appointments.as_ref())
        .and_then(|appointments| {
            appointments
                .split('/')
                .filter(|s| !s.is_empty())
                .nth(1)
                .map(String::from)
        })
        .ok_or(())
}

impl TryFrom<(OfficerListItem, bool)> for EntityRelation {
    type Error = ();

    fn try_from(value: (OfficerListItem, bool)) -> Result<Self, Self::Error> {
        let officer = value.0;
        let is_root = value.1;

        // TODO: need method to take indentification.eregistration_num
        let (company_house_number, entity_kind) =
            match match_number_to_kind(&officer.person_number, &officer.identification) {
                Some((company_house_number, entity_kind)) => (company_house_number, entity_kind),
                None => return Err(()),
            };

        let (country, postal_code) = match officer.address {
            Some(ref address) => (address.country.clone(), address.postal_code.clone()),
            None => (None, None),
        };

        let doi = Some("00/00/0000".to_string()); // TODO THIS PROPERLY

        let name = officer.name.clone();
        let started_on = officer.appointed_on.clone();
        let ended_on = officer.resigned_on.clone();
        let officer_id = extract_officer_id(officer);

        let entity = Entity {
            id: Uuid::new_v4(),
            company_house_number,
            officer_id: officer_id.ok(),
            name,
            kind: entity_kind,
            country: country,
            postal_code: postal_code,
            date_of_origin: doi,
            is_root,
        };

        Ok(Self {
            entity,
            started_on,
            ended_on,
        })
    }
}

impl TryFrom<AppointmentListItem> for EntityRelation {
    type Error = ();

    fn try_from(appointment: AppointmentListItem) -> Result<Self, Self::Error> {
        let (company_house_number, name) = match appointment.appointed_to {
            Some(appointed_to) => (appointed_to.company_number, appointed_to.company_name),
            None => return Err(()),
        };

        let company_house_number = match company_house_number {
            Some(number) => number,
            None => return Err(()),
        };

        let entity = Entity {
            id: Uuid::new_v4(),
            company_house_number,
            officer_id: None,
            name,
            kind: Entitykind::Company,
            country: None,
            postal_code: None,
            date_of_origin: None,
            is_root: false,
        };

        Ok(Self {
            entity,
            started_on: appointment.appointed_on,
            ended_on: appointment.resigned_on,
        })
    }
}

// TODO
impl TryFrom<CompanyItem> for Entity {
    type Error = ();

    fn try_from(company: CompanyItem) -> Result<Self, Self::Error> {
        todo!()
    }
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::check_entity_map)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CheckEntityMap {
    pub check_id: Uuid,
    pub entity_id: Uuid,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::relationship)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Relationship {
    pub parent_id: Uuid,
    pub child_id: Uuid,
    pub kind: Relationshipkind,
    pub started_on: Option<NaiveDate>,
    pub ended_on: Option<NaiveDate>,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::check)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Check {
    pub id: Uuid,
    pub started_at: NaiveDateTime,
    pub kind: Checkkind,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::job)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Job {
    pub id: Uuid,
    pub enqueued_at: NaiveDateTime,
    pub completed_at: Option<NaiveDateTime>,
    pub has_error: bool,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::check_job_map)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CheckJobMap {
    pub check_id: Uuid,
    pub job_id: Uuid,
}

#[derive(Debug, AsExpression, FromSqlRow, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[diesel(sql_type = crate::schema::sql_types::Flagkind)]
pub enum Flagkind {
    #[default]
    Crime,
    Fraud,
    Cybercrime,
    FinancialCrime,
    EnvironmentViolations,
    Theft,
    WarCrimes,
    CriminalLeaderShip,
    Terrorism,
    Trafficking,
    DrugTrafficking,
    HumanTrafficking,
    Wanted,
    Offshore,
    ShellCompany,
    PublicListedCompany,
    Disqualified,
    Government,
    NationalGovernment,
    StateGovernment,
    MunicipalGovernment,
    StateOwnedEnterprise,
    IntergovernmentalOrg,
    HeadOfGovernment,
    CivilService,
    ExecutiveBranchOfGovernment,
    LegislativeBranchOfGovernment,
    JudicialBranchOfGovernment,
    SecurityServices,
    CentralBankingAndFinIntegrity,
    FinancialServices,
    Bank,
    Fund,
    FinancialAdvisor,
    RegulatorAction,
    RegulatorWarning,
    Politician,
    NonPep,
    CloseAsociate,
    Judge,
    CivilServant,
    Diplomat,
    Lawyer,
    Accountant,
    Spy,
    Oligarch,
    Journalist,
    Activist,
    Lobbyist,
    PoliticalParty,
    Union,
    Religion,
    Military,
    FrozenAsset,
    SanctionedEntity,
    SanctionLinkedEntity,
    CounterSanctionedEntity,
    ExportControlled,
    TradeRisk,
    DEbarredEntity,
    PersonOfInterest,
}

impl ToSql<crate::schema::sql_types::Flagkind, Pg> for Flagkind {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            Flagkind::Crime => out.write_all(b"crime")?,
            Flagkind::Fraud => out.write_all(b"fraud")?,
            Flagkind::Cybercrime => out.write_all(b"cybercrime")?,
            Flagkind::FinancialCrime => out.write_all(b"financial_crime")?,
            Flagkind::EnvironmentViolations => out.write_all(b"environment_violations")?,
            Flagkind::Theft => out.write_all(b"theft")?,
            Flagkind::WarCrimes => out.write_all(b"war_crimes")?,
            Flagkind::CriminalLeaderShip => out.write_all(b"criminal_leadership")?,
            Flagkind::Terrorism => out.write_all(b"terrorism")?,
            Flagkind::Trafficking => out.write_all(b"trafficking")?,
            Flagkind::DrugTrafficking => out.write_all(b"drug_trafficking")?,
            Flagkind::HumanTrafficking => out.write_all(b"human_trafficking")?,
            Flagkind::Wanted => out.write_all(b"wanted")?,
            Flagkind::Offshore => out.write_all(b"offshore")?,
            Flagkind::ShellCompany => out.write_all(b"shell_company")?,
            Flagkind::PublicListedCompany => out.write_all(b"public_listed_company")?,
            Flagkind::Disqualified => out.write_all(b"disqualified")?,
            Flagkind::Government => out.write_all(b"government")?,
            Flagkind::NationalGovernment => out.write_all(b"national_government")?,
            Flagkind::StateGovernment => out.write_all(b"state_government")?,
            Flagkind::MunicipalGovernment => out.write_all(b"municipal_government")?,
            Flagkind::StateOwnedEnterprise => out.write_all(b"state_owned_enterprise")?,
            Flagkind::IntergovernmentalOrg => out.write_all(b"intergovernmental_org")?,
            Flagkind::HeadOfGovernment => out.write_all(b"head_of_government")?,
            Flagkind::CivilService => out.write_all(b"civil_service")?,
            Flagkind::ExecutiveBranchOfGovernment => {
                out.write_all(b"executive_branch_of_government")?
            }
            Flagkind::LegislativeBranchOfGovernment => {
                out.write_all(b"legislative_branch_of_government")?
            }
            Flagkind::JudicialBranchOfGovernment => {
                out.write_all(b"judicial_branch_of_government")?
            }
            Flagkind::SecurityServices => out.write_all(b"security_services")?,
            Flagkind::CentralBankingAndFinIntegrity => {
                out.write_all(b"central_banking_and_fin_integrity")?
            }
            Flagkind::FinancialServices => out.write_all(b"financial_services")?,
            Flagkind::Bank => out.write_all(b"bank")?,
            Flagkind::Fund => out.write_all(b"fund")?,
            Flagkind::FinancialAdvisor => out.write_all(b"financial_advisor")?,
            Flagkind::RegulatorAction => out.write_all(b"regulator_action")?,
            Flagkind::RegulatorWarning => out.write_all(b"regulator_warning")?,
            Flagkind::Politician => out.write_all(b"politician")?,
            Flagkind::NonPep => out.write_all(b"non_pep")?,
            Flagkind::CloseAsociate => out.write_all(b"close_associate")?,
            Flagkind::Judge => out.write_all(b"judge")?,
            Flagkind::CivilServant => out.write_all(b"civil_servant")?,
            Flagkind::Diplomat => out.write_all(b"diplomat")?, // Fixed typo
            Flagkind::Lawyer => out.write_all(b"lawyer")?,
            Flagkind::Accountant => out.write_all(b"accountant")?,
            Flagkind::Spy => out.write_all(b"spy")?,
            Flagkind::Oligarch => out.write_all(b"oligarch")?,
            Flagkind::Journalist => out.write_all(b"journalist")?,
            Flagkind::Activist => out.write_all(b"activist")?,
            Flagkind::Lobbyist => out.write_all(b"lobbyist")?,
            Flagkind::PoliticalParty => out.write_all(b"political_party")?,
            Flagkind::Union => out.write_all(b"union")?,
            Flagkind::Religion => out.write_all(b"religion")?,
            Flagkind::Military => out.write_all(b"military")?,
            Flagkind::FrozenAsset => out.write_all(b"frozen_asset")?,
            Flagkind::SanctionedEntity => out.write_all(b"sanctioned_entity")?,
            Flagkind::SanctionLinkedEntity => out.write_all(b"sanction_linked_entity")?,
            Flagkind::CounterSanctionedEntity => out.write_all(b"counter_sanctioned_entity")?,
            Flagkind::ExportControlled => out.write_all(b"export_controlled")?,
            Flagkind::TradeRisk => out.write_all(b"trade_risk")?,
            Flagkind::DEbarredEntity => out.write_all(b"debarred_entity")?,
            Flagkind::PersonOfInterest => out.write_all(b"person_of_interest")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::Flagkind, Pg> for Flagkind {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"crime" => Ok(Flagkind::Crime),
            b"fraud" => Ok(Flagkind::Fraud),
            b"cybercrime" => Ok(Flagkind::Cybercrime),
            b"financial_crime" => Ok(Flagkind::FinancialCrime),
            b"environment_violations" => Ok(Flagkind::EnvironmentViolations),
            b"theft" => Ok(Flagkind::Theft),
            b"war_crimes" => Ok(Flagkind::WarCrimes),
            b"criminal_leadership" => Ok(Flagkind::CriminalLeaderShip),
            b"terrorism" => Ok(Flagkind::Terrorism),
            b"trafficking" => Ok(Flagkind::Trafficking),
            b"drug_trafficking" => Ok(Flagkind::DrugTrafficking),
            b"human_trafficking" => Ok(Flagkind::HumanTrafficking),
            b"wanted" => Ok(Flagkind::Wanted),
            b"offshore" => Ok(Flagkind::Offshore),
            b"shell_company" => Ok(Flagkind::ShellCompany),
            b"public_listed_company" => Ok(Flagkind::PublicListedCompany),
            b"disqualified" => Ok(Flagkind::Disqualified),
            b"government" => Ok(Flagkind::Government),
            b"national_government" => Ok(Flagkind::NationalGovernment),
            b"state_government" => Ok(Flagkind::StateGovernment),
            b"municipal_government" => Ok(Flagkind::MunicipalGovernment),
            b"state_owned_enterprise" => Ok(Flagkind::StateOwnedEnterprise),
            b"intergovernmental_org" => Ok(Flagkind::IntergovernmentalOrg),
            b"head_of_government" => Ok(Flagkind::HeadOfGovernment),
            b"civil_service" => Ok(Flagkind::CivilService),
            b"executive_branch_of_government" => Ok(Flagkind::ExecutiveBranchOfGovernment),
            b"legislative_branch_of_government" => Ok(Flagkind::LegislativeBranchOfGovernment),
            b"judicial_branch_of_government" => Ok(Flagkind::JudicialBranchOfGovernment),
            b"security_services" => Ok(Flagkind::SecurityServices),
            b"central_banking_and_fin_integrity" => Ok(Flagkind::CentralBankingAndFinIntegrity),
            b"financial_services" => Ok(Flagkind::FinancialServices),
            b"bank" => Ok(Flagkind::Bank),
            b"fund" => Ok(Flagkind::Fund),
            b"financial_advisor" => Ok(Flagkind::FinancialAdvisor),
            b"regulator_action" => Ok(Flagkind::RegulatorAction),
            b"regulator_warning" => Ok(Flagkind::RegulatorWarning),
            b"politician" => Ok(Flagkind::Politician),
            b"non_pep" => Ok(Flagkind::NonPep),
            b"close_associate" => Ok(Flagkind::CloseAsociate),
            b"judge" => Ok(Flagkind::Judge),
            b"civil_servant" => Ok(Flagkind::CivilServant),
            b"diplomat" => Ok(Flagkind::Diplomat),
            b"lawyer" => Ok(Flagkind::Lawyer),
            b"accountant" => Ok(Flagkind::Accountant),
            b"spy" => Ok(Flagkind::Spy),
            b"oligarch" => Ok(Flagkind::Oligarch),
            b"journalist" => Ok(Flagkind::Journalist),
            b"activist" => Ok(Flagkind::Activist),
            b"lobbyist" => Ok(Flagkind::Lobbyist),
            b"political_party" => Ok(Flagkind::PoliticalParty),
            b"union" => Ok(Flagkind::Union),
            b"religion" => Ok(Flagkind::Religion),
            b"military" => Ok(Flagkind::Military),
            b"frozen_asset" => Ok(Flagkind::FrozenAsset),
            b"sanctioned_entity" => Ok(Flagkind::SanctionedEntity),
            b"sanction_linked_entity" => Ok(Flagkind::SanctionLinkedEntity),
            b"counter_sanctioned_entity" => Ok(Flagkind::CounterSanctionedEntity),
            b"export_controlled" => Ok(Flagkind::ExportControlled),
            b"trade_risk" => Ok(Flagkind::TradeRisk),
            b"debarred_entity" => Ok(Flagkind::DEbarredEntity),
            b"person_of_interest" => Ok(Flagkind::PersonOfInterest),

            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl TryFrom<&str> for Flagkind {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "crime" => Ok(Flagkind::Crime),
            "crime.fraud" => Ok(Flagkind::Fraud),
            "crime.cyber" => Ok(Flagkind::Cybercrime),
            "crime.fin" => Ok(Flagkind::FinancialCrime),
            "crime.env" => Ok(Flagkind::EnvironmentViolations),
            "crime.theft" => Ok(Flagkind::Theft),
            "crime.war" => Ok(Flagkind::WarCrimes),
            "crime.boss" => Ok(Flagkind::CriminalLeaderShip),
            "crime.terror" => Ok(Flagkind::Terrorism),
            "crime.traffick" => Ok(Flagkind::Trafficking),
            "crime.traffick.drug" => Ok(Flagkind::DrugTrafficking),
            "crime.traffick.human" => Ok(Flagkind::HumanTrafficking),
            "wanted" => Ok(Flagkind::Wanted),
            "corp.offshore" => Ok(Flagkind::Offshore),
            "corp.shell" => Ok(Flagkind::ShellCompany),
            "corp.public" => Ok(Flagkind::PublicListedCompany),
            "corp.disqual" => Ok(Flagkind::Disqualified),
            "gov" => Ok(Flagkind::Government),
            "gov.national" => Ok(Flagkind::NationalGovernment),
            "gov.state" => Ok(Flagkind::StateGovernment),
            "gov.muni" => Ok(Flagkind::MunicipalGovernment),
            "gov.soe" => Ok(Flagkind::StateOwnedEnterprise),
            "gov.igo" => Ok(Flagkind::IntergovernmentalOrg),
            "gov.head" => Ok(Flagkind::HeadOfGovernment),
            "gov.admin" => Ok(Flagkind::CivilService),
            "gov.executive" => Ok(Flagkind::ExecutiveBranchOfGovernment),
            "gov.legislative" => Ok(Flagkind::LegislativeBranchOfGovernment),
            "gov.judicial" => Ok(Flagkind::JudicialBranchOfGovernment),
            "gov.security" => Ok(Flagkind::SecurityServices),
            "gov.financial" => Ok(Flagkind::CentralBankingAndFinIntegrity),
            "fin" => Ok(Flagkind::FinancialServices),
            "fin.bank" => Ok(Flagkind::Bank),
            "fin.fund" => Ok(Flagkind::Fund),
            "fin.adivsor" => Ok(Flagkind::FinancialAdvisor),
            "reg.action" => Ok(Flagkind::RegulatorAction),
            "reg.warn" => Ok(Flagkind::RegulatorWarning),
            "role.pep" => Ok(Flagkind::Politician),
            "role.pol" => Ok(Flagkind::NonPep),
            "role.rca" => Ok(Flagkind::CloseAsociate),
            "role.judge" => Ok(Flagkind::Judge),
            "role.civil" => Ok(Flagkind::CivilServant),
            "role.diplo" => Ok(Flagkind::Diplomat),
            "role.lawyer" => Ok(Flagkind::Lawyer),
            "role.acct" => Ok(Flagkind::Accountant),
            "role.spy" => Ok(Flagkind::Spy),
            "role.oligarch" => Ok(Flagkind::Oligarch),
            "role.journo" => Ok(Flagkind::Journalist),
            "role.act" => Ok(Flagkind::Activist),
            "role.lobby" => Ok(Flagkind::Lobbyist),
            "pol.party" => Ok(Flagkind::PoliticalParty),
            "pol.union" => Ok(Flagkind::Union),
            "rel" => Ok(Flagkind::Religion),
            "mil" => Ok(Flagkind::Military),
            "asset.frozen" => Ok(Flagkind::FrozenAsset),
            "sanction" => Ok(Flagkind::SanctionedEntity),
            "sanction.linked" => Ok(Flagkind::SanctionLinkedEntity),
            "sanction.counter" => Ok(Flagkind::CounterSanctionedEntity),
            "export.control" => Ok(Flagkind::ExportControlled),
            "export.risk" => Ok(Flagkind::TradeRisk),
            "debarment" => Ok(Flagkind::DEbarredEntity),
            "poi" => Ok(Flagkind::PersonOfInterest),
            _ => Err(()),
        }
    }
}

pub struct FlagStringList(pub Vec<String>);

impl From<FlagStringList> for Vec<Flagkind> {
    fn from(list: FlagStringList) -> Self {
        list.0
            .into_iter()
            .filter_map(|value| (*value).try_into().ok())
            .collect()
    }
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::flag)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Flag {
    pub id: Uuid,
    pub kind: Flagkind,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::flags)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Flags {
    pub entity_id: Uuid,
    pub flag_id: Uuid,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::dataset)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Dataset {
    pub id: Uuid,
    pub name: String,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::datasets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Datasets {
    pub entity_id: Uuid,
    pub dataset_id: Uuid,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::position)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Position {
    pub id: Uuid,
    pub title: String,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::positions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Positions {
    pub entity_id: Uuid,
    pub position_id: Uuid,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::outlier_age)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OutlierAge {
    pub entity_id: Uuid,
    pub outlier: bool,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::dormant_company)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DormantCompany {
    pub entity_id: Uuid,
    pub dormant: bool,
}

#[derive(Debug, AsExpression, FromSqlRow, Default, Serialize, Deserialize, PartialEq)]
#[diesel(sql_type = crate::schema::sql_types::Checkkind)]
pub enum Checkkind {
    #[default]
    EntityRelation,
    MonitoredEntity,
}

impl ToSql<crate::schema::sql_types::Checkkind, Pg> for Checkkind {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            Checkkind::EntityRelation => out.write_all(b"entity_relation")?,
            Checkkind::MonitoredEntity => out.write_all(b"monitored_entity")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::Checkkind, Pg> for Checkkind {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"entity_relation" => Ok(Checkkind::EntityRelation),
            b"monitored_entity" => Ok(Checkkind::MonitoredEntity),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::check_monitored_entity)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CheckMonitoredEntity {
    pub check_id: Uuid,
    pub monitored_entity_id: Uuid,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::monitored_entity)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MonitoredEntity {
    pub id: Uuid,
    pub company_house_id: String,
    pub monitoring_span_id: Uuid,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::snapshot)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Snapshot {
    pub id: Uuid,
    pub recieved_at: NaiveDate,
    pub entity_id: Uuid,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::check_snapshot)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CheckSnapshot {
    pub check_id: Uuid,
    pub snapshot_id: Uuid,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::processed_update)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ProcessedUpdate {
    pub id: Uuid,
    pub processed_at: NaiveDate,
    pub timepoint: i32,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::monitoring_span)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MonitoringSpan {
    pub id: Uuid,
    pub started_at: NaiveDate,
    pub ended_at: Option<NaiveDate>,
}
