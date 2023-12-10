use ormx::Table;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Table)]
#[ormx(table = "annotations", id = id, insertable)]
pub struct Annotation {
    #[ormx(default)]
    pub id: u64,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, FromRow, Table)]
#[ormx(table = "common_annotations", id = id, insertable)]
pub struct CommonAnnotation {
    #[ormx(default)]
    pub id: u64,
    pub name: String,
    pub value: String,
}
