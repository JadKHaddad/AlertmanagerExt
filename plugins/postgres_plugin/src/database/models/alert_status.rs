use crate::database::schema::sql_types::AlertStatus;
use diesel::backend::Backend;
use diesel::{
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    query_builder::bind_collector::RawBytesBindCollector,
    serialize::{self, Output, ToSql},
    sql_types::VarChar,
};
use models::Status as AlermanagerPushStatus;
use std::io::Write;

#[derive(Clone, Debug, FromSqlRow, AsExpression)]
#[diesel(sql_type = AlertStatus)]
pub enum AlertStatusModel {
    Resolved,
    Firing,
}

impl From<&AlermanagerPushStatus> for AlertStatusModel {
    fn from(status: &AlermanagerPushStatus) -> Self {
        match status {
            AlermanagerPushStatus::Resolved => AlertStatusModel::Resolved,
            AlermanagerPushStatus::Firing => AlertStatusModel::Firing,
        }
    }
}

impl<DB> ToSql<AlertStatus, DB> for AlertStatusModel
where
    for<'c> DB: Backend<BindCollector<'c> = RawBytesBindCollector<DB>>,
{
    fn to_sql<'a>(&'a self, out: &mut Output<'a, '_, DB>) -> serialize::Result {
        match self {
            AlertStatusModel::Resolved => out.write_all(b"resolved")?,
            AlertStatusModel::Firing => out.write_all(b"firing")?,
        }
        Ok(serialize::IsNull::No)
    }
}

impl<DB> FromSql<AlertStatus, DB> for AlertStatusModel
where
    DB: Backend,
    *const str: FromSql<VarChar, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        let s = <String as FromSql<VarChar, DB>>::from_sql(bytes)?;
        match s.as_str() {
            "resolved" => Ok(AlertStatusModel::Resolved),
            "firing" => Ok(AlertStatusModel::Firing),
            _ => Err(format!("Unrecognized enum variant: {}", s).into()),
        }
    }
}
