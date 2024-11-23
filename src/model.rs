use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, IsNull, Output, ToSql};
use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Debug, Clone, Copy, AsExpression, FromSqlRow)]
#[diesel(sql_type = crate::schema::sql_types::RelationshipKind)]
pub enum RelationshipKind {
    Shareholder,
    Officer,
}

impl ToSql<crate::schema::sql_types::RelationshipKind, Pg> for RelationshipKind {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            RelationshipKind::Shareholder => out.write_all(b"shareholder")?,
            RelationshipKind::Officer => out.write_all(b"officer")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::RelationshipKind, Pg> for RelationshipKind {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"shareholder" => Ok(RelationshipKind::Shareholder),
            b"officer" => Ok(RelationshipKind::Officer),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Debug, AsExpression, FromSqlRow, Default, Serialize, Deserialize)]
#[diesel(sql_type = crate::schema::sql_types::EntityKind)]
pub enum EntityKind {
    #[default]
    Company,
    Individual,
}

impl ToSql<crate::schema::sql_types::EntityKind, Pg> for EntityKind {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            EntityKind::Company => out.write_all(b"company")?,
            EntityKind::Individual => out.write_all(b"individual")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::EntityKind, Pg> for EntityKind {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"company" => Ok(EntityKind::Company),
            b"individual" => Ok(EntityKind::Individual),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

// TODO: this needs proper mapping between returned kinds
impl From<Option<String>> for EntityKind {
    fn from(_kind: Option<String>) -> Self {
        Self::Company
        // should default to company
    }
}
