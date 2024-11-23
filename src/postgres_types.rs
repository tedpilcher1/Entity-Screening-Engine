use chrono::NaiveDateTime;
use diesel::prelude::{Insertable, Queryable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    company_house_response_types::{OfficerListItem, ShareholderListItem},
    model::{EntityKind, RelationshipKind},
    schema::{check, check_entity_map, entity, relationship},
};

#[derive(Debug, Queryable, Insertable, Default, Serialize, Deserialize)]
#[diesel(table_name = entity)]
pub struct Entity {
    pub id: Uuid,
    pub company_house_number: String,
    pub name: Option<String>,
    pub kind: EntityKind,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub date_of_origin: Option<String>,
    pub is_root: bool,
}

impl Entity {
    pub fn create_root() -> Self {
        Self {
            id: Uuid::new_v4(),
            is_root: true,
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

#[derive(Debug, Queryable, Insertable)]
#[diesel(table_name = check_entity_map)]
pub struct CheckEntityMap {
    pub check_id: Uuid,
    pub entity_id: Uuid,
}

#[derive(Debug, Queryable, Insertable)]
#[diesel(table_name = relationship)]
pub struct Relationship {
    pub parent_id: Uuid,
    pub child_id: Uuid,
    pub kind: RelationshipKind,
}

#[derive(Debug, Queryable, Insertable)]
#[diesel(table_name = check)]
pub struct Check {
    pub id: Uuid,
    pub started_at: NaiveDateTime,
}
