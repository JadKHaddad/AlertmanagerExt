use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Annotation {
    pub id: i32,
    pub name: String,
    pub value: String,
}
