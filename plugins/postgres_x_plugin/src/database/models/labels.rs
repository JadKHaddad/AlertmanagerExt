use sqlx::{FromRow, Type};

#[derive(Debug, Clone, FromRow, Type)]
/// Has to match every single field in the returned row in array_agg, otherwise using `as "labels!: Vec<Label>"` in [`sqlx::query_as!`]
/// will fail at runtime without emitting any warnings at compile time.
/// See function impl [`pull_definitions::Pull`] for [`crate::PostgresXPlugin`] for more details.
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
