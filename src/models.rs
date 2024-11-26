use chrono::NaiveDateTime;
use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::prelude::{Insertable, Queryable};
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::Selectable;
use serde::{Deserialize, Serialize};
use std::io::Write;
use uuid::Uuid;

use crate::company_house_response_types::{CompanyItem, OfficerListItem, ShareholderListItem};

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

impl TryFrom<(ShareholderListItem, bool)> for Entity {
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

        Ok(Self {
            id: Uuid::new_v4(),
            company_house_number,
            name: shareholder.name,
            kind: shareholder.kind.into(),
            country: country,
            postal_code: postal_code,
            date_of_origin: doi,
            is_root,
        })
    }
}

impl TryFrom<(OfficerListItem, bool)> for Entity {
    type Error = ();

    fn try_from(value: (OfficerListItem, bool)) -> Result<Self, Self::Error> {
        let officer = value.0;
        let is_root = value.1;

        // TODO: this should be simplified
        let identification = match officer.identification {
            Some(identification) => identification,
            None => return Err(()),
        };

        let company_house_number = match identification.registration_number {
            Some(registration_numer) => registration_numer,
            None => return Err(()),
        };

        let company_house_number = format!("{:0>8}", company_house_number);

        let (country, postal_code) = match officer.address {
            Some(address) => (address.country, address.postal_code),
            None => (None, None),
        };

        let doi = Some("00/00/0000".to_string()); // TODO THIS PROPERLY

        Ok(Self {
            id: Uuid::new_v4(),
            company_house_number,
            name: officer.name,
            kind: officer.officer_role.into(),
            country: country,
            postal_code: postal_code,
            date_of_origin: doi,
            is_root,
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
