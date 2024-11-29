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

use crate::company_house_response_types::{
    CompanyItem, Identification, OfficerListItem, OfficerListResponse, ShareholderList,
    ShareholderListItem,
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

#[derive(Debug, AsExpression, FromSqlRow, Default, Serialize, Deserialize)]
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

fn matchNumberToKind(
    person_number: Option<String>,
    identification: Option<Identification>,
) -> Option<(CompanyHouseNumber, Entitykind)> {
    match person_number {
        Some(person_number) => return Some((person_number, Entitykind::Individual)),
        None => {}
    }

    match identification {
        Some(identification) => {
            // let company_house_number = format!("{:0>8}", identification.registration_number.unwrap_or_else(f));
            let company_house_number = identification.registration_number;

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
    pub company_house_number: String,
    pub name: Option<String>,
    pub kind: Entitykind,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub date_of_origin: Option<String>,
    pub is_root: bool,
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
                    warn!("Failed to convert to entity."); // todo improve log
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
                    warn!("Failed to convert to entity."); // todo improve log
                }
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

        // TODO: this should be simplified
        let shareholder_identification = match shareholder.identification {
            Some(identification) => identification,
            None => return Err(()),
        };

        let company_house_number = match shareholder_identification.registration_number {
            Some(registration_numer) => registration_numer,
            None => return Err(()),
        };

        let company_house_number = format!("{:0>8}", company_house_number);

        let (country, postal_code) = match shareholder.address {
            Some(address) => (address.country, address.postal_code),
            None => (None, None),
        };

        let doi = Some("00/00/0000".to_string()); // TODO THIS PROPERLY

        let entity = Entity {
            id: Uuid::new_v4(),
            company_house_number,
            name: shareholder.name,
            kind: shareholder.kind.into(),
            country: country,
            postal_code: postal_code,
            date_of_origin: doi,
            is_root,
        };

        Ok(Self {
            entity,
            started_on: shareholder.notified_on,
            ended_on: shareholder.ceased_on,
        })
    }
}

impl TryFrom<(OfficerListItem, bool)> for EntityRelation {
    type Error = ();

    fn try_from(value: (OfficerListItem, bool)) -> Result<Self, Self::Error> {
        let officer = value.0;
        let is_root = value.1;

        // TODO: need method to take indentification.eregistration_num
        let (company_house_number, entity_kind) =
            match matchNumberToKind(officer.person_number, officer.identification) {
                Some((company_house_number, entity_kind)) => (company_house_number, entity_kind),
                None => return Err(()),
            };

        let (country, postal_code) = match officer.address {
            Some(address) => (address.country, address.postal_code),
            None => (None, None),
        };

        let doi = Some("00/00/0000".to_string()); // TODO THIS PROPERLY

        let entity = Entity {
            id: Uuid::new_v4(),
            company_house_number,
            name: officer.name,
            kind: entity_kind,
            country: country,
            postal_code: postal_code,
            date_of_origin: doi,
            is_root,
        };

        Ok(Self {
            entity,
            started_on: officer.appointed_on,
            ended_on: officer.resigned_on,
        })
    }
}

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
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::job)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Job {
    pub id: Uuid,
    pub enqueued_at: NaiveDateTime,
    pub completed_at: Option<NaiveDateTime>,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::check_job_map)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CheckJobMap {
    pub check_id: Uuid,
    pub job_id: Uuid,
}
