use sqlx::{FromRow, Type};

#[derive(Debug, Clone, FromRow, Type)]
pub struct Label {
    pub id: i32,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct CommonLabel {
    pub id: i32,
    pub name: String,
    pub value: String,
}
