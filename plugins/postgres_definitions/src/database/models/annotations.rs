use crate::database::schema::{annotations, common_annotations};
use diesel::prelude::*;
use sqlx::{FromRow, Type};

#[derive(Insertable)]
#[diesel(table_name = annotations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableAnnotation<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug, Clone, FromRow, Type)]
#[diesel(table_name = annotations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Annotation {
    pub id: i32,
    pub name: String,
    pub value: String,
}

#[derive(Insertable)]
#[diesel(table_name = common_annotations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableCommonAnnotation<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug, Clone)]
#[diesel(table_name = common_annotations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CommonAnnotation {
    pub id: i32,
    pub name: String,
    pub value: String,
}
