use ormx::Table;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Table)]
#[ormx(table = "labels", id = id, insertable)]
pub struct Label {
    #[ormx(default)]
    pub id: u64,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, FromRow, Table)]
#[ormx(table = "common_labels", id = id, insertable)]
pub struct CommonLabel {
    #[ormx(default)]
    pub id: u64,
    pub name: String,
    pub value: String,
}
