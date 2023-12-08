use ormx::Table;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Table)]
#[ormx(table = "`groups`", id = id, insertable)]
pub struct Group {
    #[ormx(default)]
    pub id: u64,
    pub group_key: String,
    pub receiver: String,
    pub status: String,
    pub external_url: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct GroupLabel {
    pub group_id: u64,
    pub label_id: u64,
}

#[derive(Debug, Clone, FromRow)]
pub struct GroupCommonLabel {
    pub group_id: u64,
    pub common_label_id: u64,
}

#[derive(Debug, Clone, FromRow)]
pub struct GroupCommonAnnotation {
    pub group_id: i32,
    pub common_annotation_id: i32,
}
