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

#[derive(Debug, Clone, FromRow, Table)]
#[ormx(table = "groups_labels", id = id, insertable)]
pub struct GroupLabel {
    #[ormx(default)]
    pub id: u64,
    pub group_id: u64,
    pub label_id: u64,
}

#[derive(Debug, Clone, FromRow, Table)]
#[ormx(table = "groups_common_labels", id = id, insertable)]
pub struct GroupCommonLabel {
    #[ormx(default)]
    pub id: u64,
    pub group_id: u64,
    pub common_label_id: u64,
}

#[derive(Debug, Clone, FromRow, Table)]
#[ormx(table = "groups_common_annotations", id = id, insertable)]
pub struct GroupCommonAnnotation {
    #[ormx(default)]
    pub id: u64,
    pub group_id: u64,
    pub common_annotation_id: u64,
}
