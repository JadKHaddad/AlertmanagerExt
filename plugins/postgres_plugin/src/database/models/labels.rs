use crate::database::schema::{common_labels, labels};
use diesel::prelude::*;

#[derive(Insertable)]
#[diesel(table_name = labels)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableLabel<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug, Clone)]
#[diesel(table_name = labels)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Label {
    pub id: i32,
    pub name: String,
    pub value: String,
}

#[derive(Insertable)]
#[diesel(table_name = common_labels)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableCommonLabel<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug, Clone)]
#[diesel(table_name = common_labels)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CommonLabel {
    pub id: i32,
    pub name: String,
    pub value: String,
}
