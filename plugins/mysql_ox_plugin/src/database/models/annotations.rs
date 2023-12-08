use sqlx::{FromRow, Type};

#[derive(Debug, Clone, FromRow, Type)]
pub struct Annotation {
    pub id: i32,
    pub name: String,
    pub value: String,
}
